use std::fmt::Debug;

use anyhow::anyhow;
use cynic::http::ReqwestExt;
use cynic::schema::QueryRoot;
use cynic::{Id, Operation, QueryFragment, QueryVariables};
use log::trace;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::gql::query::{Homes, WebsocketSubscriptionUrl};
use crate::gql::subscription::SubscriptionVariables;
use crate::gql::subscription::{LiveMeasurementSubscription, TestMeasurementSubscription};
use crate::subscription::Subscription;

mod gql;
pub mod subscription;

const API_URL: &str = "https://api.tibber.com/v1-beta/gql/";
const DEMO_TOKEN: &str = "5K4MVS-OjfWhK_4yrjOlFe1F6kJXPVf7eQYggo8ebAE";

pub struct TibberClient {
    token: String,
}

impl TibberClient {
    async fn fetch_data<T: QueryFragment + Debug + 'static, V: QueryVariables + Serialize>(
        &self,
        variables: V,
    ) -> Result<T, TibberClientError>
    where
        for<'de> T: Deserialize<'de>,
        <T as QueryFragment>::SchemaType: QueryRoot,
    {
        let client = Client::builder()
            .user_agent(Self::get_user_agent())
            .default_headers(
                std::iter::once((
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {}", self.token))
                        .map_err(|e| {
                            <reqwest::header::InvalidHeaderValue as Into<anyhow::Error>>::into(e)
                        })?,
                ))
                .collect(),
            )
            .build()
            .map_err(|e| TibberClientError::RequestFailed(e.into()))?;

        let response_body = client
            .post(API_URL)
            .run_graphql(Operation::query(variables))
            .await
            .map_err(|e| TibberClientError::RequestFailed(e.into()))?;

        trace!("Got query response: {:?}", response_body);

        let Some(response_data) = response_body.data else {
            return Err(TibberClientError::RequestFailed(anyhow!(
                "Got empty response!"
            )));
        };
        Ok(response_data)
    }

    async fn get_subscription_url(&self) -> Result<String, TibberClientError> {
        let r = self.fetch_data::<WebsocketSubscriptionUrl, _>(()).await?;
        r.viewer
            .websocket_subscription_url
            .ok_or(anyhow!("API response malformed!").into())
    }

    fn get_user_agent() -> String {
        format!(
            "tibber-api-rs/{} graphql-rust/???", // TODO
            env!("CARGO_PKG_VERSION")
        )
    }
}

impl TibberClient {
    pub fn new<S: Into<String>>(token: S) -> Self {
        Self {
            token: token.into(),
        }
    }

    #[must_use]
    pub fn new_demo() -> Self {
        Self {
            token: DEMO_TOKEN.into(),
        }
    }

    /// # Errors
    ///
    /// Will return Err if the client fails establish the subscription
    pub async fn subscribe_live_measurement<S: Into<String>>(
        self,
        id: S,
    ) -> Result<Subscription<LiveMeasurementSubscription>, TibberClientError> {
        Subscription::new(
            self,
            SubscriptionVariables {
                home_id: Id::from(id),
            },
        )
        .await
    }

    /// # Errors
    ///
    /// Will return Err if the client fails establish the subscription
    #[deprecated = "Do not use this in production!"]
    pub async fn subscribe_test_measurement<S: Into<String>>(
        self,
        id: S,
    ) -> Result<Subscription<TestMeasurementSubscription>, TibberClientError> {
        Subscription::new(
            self,
            SubscriptionVariables {
                home_id: Id::from(id),
            },
        )
        .await
    }

    /// # Errors
    ///
    /// Will return Err if the client fails to fetch the data
    pub async fn query_homes(&self) -> Result<Vec<String>, TibberClientError> {
        let r = self.fetch_data::<Homes, _>(()).await?;
        Ok(r.viewer
            .homes
            .into_iter()
            .flatten()
            .map(|h| h.id.inner().to_string())
            .collect())
    }
}

#[derive(Debug, Error)]
pub enum TibberClientError {
    #[error(transparent)]
    RequestFailed(anyhow::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl PartialEq for TibberClientError {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
