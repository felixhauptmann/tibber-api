use std::fmt::Debug;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_tungstenite::tungstenite::http;
use async_tungstenite::tungstenite::{client::IntoClientRequest, http::HeaderValue};
use cynic::schema::SubscriptionRoot;
use cynic::{GraphQlResponse, QueryFragment, StreamingOperation};
use futures::Stream;
pub use futures::StreamExt;
use graphql_ws_client::Client;
use serde_json::json;
use thiserror::Error;

pub use crate::gql::subscription::LiveMeasurement as Measurement;
use crate::gql::subscription::{
    LiveMeasurementSubscription, SubscriptionQuery, SubscriptionVariables,
    TestMeasurementSubscription,
};
use crate::{TibberClient, TibberClientError};

pub struct Subscription<S: SubscriptionQuery>
where
    <S as QueryFragment>::SchemaType: SubscriptionRoot,
{
    subscription: graphql_ws_client::Subscription<StreamingOperation<S, SubscriptionVariables>>,
}

impl<S: SubscriptionQuery> Subscription<S>
where
    <S as QueryFragment>::SchemaType: SubscriptionRoot,
{
    pub(crate) async fn new(
        client: TibberClient,
        variables: SubscriptionVariables,
    ) -> Result<Self, TibberClientError> {
        let ws_url = client.get_subscription_url().await?;

        let mut request = ws_url.into_client_request().unwrap();
        request.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            HeaderValue::from_str("graphql-transport-ws").unwrap(),
        );
        request.headers_mut().insert(
            http::header::USER_AGENT,
            HeaderValue::from_str(&TibberClient::get_user_agent()).unwrap(),
        );

        let (connection, _) = async_tungstenite::tokio::connect_async(request)
            .await
            .unwrap();

        let subscription = Client::build(connection)
            .payload(json!({"token": client.token}))
            .unwrap()
            .subscribe(StreamingOperation::subscription(variables))
            .await
            .unwrap();

        Ok(Self { subscription })
    }

    pub async fn stop(self) -> Result<(), SubscriptionError> {
        self.subscription.stop().await.map_err(|e| e.into())
    }
}

impl Stream for Subscription<LiveMeasurementSubscription> {
    type Item = Result<Measurement, SubscriptionError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.subscription.poll_next_unpin(cx).map(|r| {
            r.map(|r| match r {
                Ok(GraphQlResponse {
                    data:
                        Some(LiveMeasurementSubscription {
                            live_measurement: Some(live_measurement),
                        }),
                    ..
                }) => Ok(live_measurement),
                Ok(_) => Err(SubscriptionError::NoData),
                Err(e) => Err(SubscriptionError::WebsocketError(e)),
            })
        })
    }
}

impl Stream for Subscription<TestMeasurementSubscription> {
    type Item = Result<Measurement, SubscriptionError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.subscription.poll_next_unpin(cx).map(|r| {
            r.map(|r| match r {
                Ok(GraphQlResponse {
                    data:
                        Some(TestMeasurementSubscription {
                            test_measurement: Some(live_measurement),
                        }),
                    ..
                }) => Ok(live_measurement),
                Ok(_) => Err(SubscriptionError::NoData),
                Err(e) => Err(e.into()),
            })
        })
    }
}

#[derive(Debug, Error)]
pub enum SubscriptionError {
    #[error(transparent)]
    WebsocketError(#[from] graphql_ws_client::Error),
    #[error("Message contains no data")]
    NoData,
}
