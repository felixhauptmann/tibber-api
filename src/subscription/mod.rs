use std::fmt::Debug;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_tungstenite::tungstenite::http;
use async_tungstenite::tungstenite::{client::IntoClientRequest, http::HeaderValue};
use futures::Stream;
pub use futures::StreamExt;
use graphql_client::GraphQLQuery;
use graphql_ws_client::graphql::StreamingOperation;
use graphql_ws_client::Client;
use graphql_ws_client::Subscription as WSSubscription;
use serde_json::json;
use thiserror::Error;

use crate::gql::subscription::live_measurement;
use crate::gql::subscription::live_measurement::LiveMeasurementLiveMeasurement;
use crate::gql::subscription::test_measurement;
use crate::gql::subscription::test_measurement::TestMeasurementTestMeasurement;
use crate::gql::subscription::StreamQuery;
use crate::{LiveMeasurement, TestMeasurement, TibberClient, TibberClientError};

pub struct Subscription<T: StreamQuery> {
    subscription: WSSubscription<StreamingOperation<T>>,
}

impl<T: StreamQuery> Subscription<T> {
    pub(crate) async fn new(
        client: TibberClient,
        variables: T::Variables,
    ) -> Result<Self, TibberClientError>
    where
        <T as GraphQLQuery>::Variables: Unpin + Send,
    {
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
            .subscribe(StreamingOperation::<T>::new(variables))
            .await
            .unwrap();

        Ok(Self { subscription })
    }
}

impl Stream for Subscription<LiveMeasurement> {
    type Item = Result<Measurement, SubscriptionError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use graphql_client::Response;

        self.subscription.poll_next_unpin(cx).map(|r| {
            r.map(|r| match r {
                Ok(Response {
                    data:
                        Some(live_measurement::ResponseData {
                            live_measurement: Some(d),
                        }),
                    ..
                }) => Ok(d.into()),
                Ok(_) => Err(SubscriptionError::NoData),
                Err(e) => Err(SubscriptionError::WebsocketError(e)),
            })
        })
    }
}

impl Stream for Subscription<TestMeasurement> {
    type Item = Result<Measurement, SubscriptionError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use graphql_client::Response;

        self.subscription.poll_next_unpin(cx).map(|r| {
            r.map(|r| match r {
                Ok(Response {
                    data:
                        Some(test_measurement::ResponseData {
                            test_measurement: Some(d),
                        }),
                    ..
                }) => Ok(d.into()),
                Ok(_) => Err(SubscriptionError::NoData),
                Err(e) => Err(SubscriptionError::WebsocketError(e)),
            })
        })
    }
}

#[derive(Debug)]
pub struct Measurement {
    pub timestamp: String,
    pub power: f64,
    pub accumulated_consumption: f64,
    pub accumulated_consumption_last_hour: f64,
    pub accumulated_production: f64,
    pub accumulated_production_last_hour: f64,
    pub accumulated_cost: Option<f64>,
    pub accumulated_reward: Option<f64>,
}

impl From<LiveMeasurementLiveMeasurement> for Measurement {
    fn from(value: LiveMeasurementLiveMeasurement) -> Self {
        Self {
            timestamp: value.timestamp,
            power: value.power,
            accumulated_consumption: value.accumulated_consumption,
            accumulated_consumption_last_hour: value.accumulated_consumption_last_hour,
            accumulated_production: value.accumulated_production,
            accumulated_production_last_hour: value.accumulated_production_last_hour,
            accumulated_cost: value.accumulated_cost,
            accumulated_reward: value.accumulated_reward,
        }
    }
}

impl From<TestMeasurementTestMeasurement> for Measurement {
    fn from(value: TestMeasurementTestMeasurement) -> Self {
        Self {
            timestamp: value.timestamp,
            power: value.power,
            accumulated_consumption: value.accumulated_consumption,
            accumulated_consumption_last_hour: value.accumulated_consumption_last_hour,
            accumulated_production: value.accumulated_production,
            accumulated_production_last_hour: value.accumulated_production_last_hour,
            accumulated_cost: value.accumulated_cost,
            accumulated_reward: value.accumulated_reward,
        }
    }
}

#[derive(Debug, Error)]
pub enum SubscriptionError {
    #[error(transparent)]
    WebsocketError(graphql_ws_client::Error),
    #[error("Message contains no data")]
    NoData,
}
