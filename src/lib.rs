use anyhow::anyhow;
use graphql_client::reqwest::post_graphql;
use graphql_client::GraphQLQuery;
use log::debug;
use reqwest::Client;
use thiserror::Error;

use crate::gql::query;
use crate::gql::query::ws_subscription_url;
use crate::gql::query::WSSubscriptionUrl;
use crate::gql::subscription::LiveMeasurement;
use crate::gql::subscription::TestMeasurement;
use crate::subscription::Subscription;

mod gql;
pub mod subscription;

const API_URL: &str = "https://api.tibber.com/v1-beta/gql/";
const DEMO_TOKEN: &str = "5K4MVS-OjfWhK_4yrjOlFe1F6kJXPVf7eQYggo8ebAE";

pub struct TibberClient {
    token: String,
}

impl TibberClient {
    async fn fetch_data<T: GraphQLQuery>(
        &self,
        variables: <T as GraphQLQuery>::Variables,
    ) -> Result<<T as GraphQLQuery>::ResponseData, TibberClientError>
    where
        <T as GraphQLQuery>::ResponseData: std::fmt::Debug,
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

        let response_body = post_graphql::<T, _>(&client, API_URL, variables)
            .await
            .map_err(|e| TibberClientError::RequestFailed(e.into()))?;

        debug!("Got query response: {:?}", response_body);

        let response_data = match response_body.data {
            Some(d) => d,
            None => {
                return Err(TibberClientError::RequestFailed(anyhow!(
                    "Got empty response!"
                )))
            }
        };
        Ok(response_data)
    }

    async fn get_subscription_url(&self) -> Result<String, TibberClientError> {
        let r = self
            .fetch_data::<WSSubscriptionUrl>(ws_subscription_url::Variables)
            .await?;
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
    pub fn new_demo() -> Self {
        Self {
            token: DEMO_TOKEN.into(),
        }
    }

    pub async fn subscribe_live_measurement<S: Into<String>>(
        self,
        id: S,
    ) -> Result<Subscription<LiveMeasurement>, TibberClientError> {
        Subscription::new(
            self,
            gql::subscription::live_measurement::Variables { id: id.into() },
        )
        .await
    }

    #[deprecated = "Do not use this in production!"]
    pub async fn subscribe_test_measurement<S: Into<String>>(
        self,
        id: S,
    ) -> Result<Subscription<TestMeasurement>, TibberClientError> {
        Subscription::new(
            self,
            gql::subscription::test_measurement::Variables { id: id.into() },
        )
        .await
    }

    pub async fn query_homes(&self) -> Result<Vec<String>, TibberClientError> {
        let r = self
            .fetch_data::<query::Homes>(query::homes::Variables)
            .await?;
        Ok(r.viewer.homes.into_iter().flatten().map(|h| h.id).collect())
        // .ok_or(anyhow!("API response malformed!").into())
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
