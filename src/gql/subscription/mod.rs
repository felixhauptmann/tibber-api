use cynic::QueryFragment;

use crate::gql::schema;

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "RootSubscription", variables = "SubscriptionVariables")]
pub struct LiveMeasurementSubscription {
    #[arguments(homeId: $home_id)]
    pub(crate) live_measurement: Option<LiveMeasurement>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "RootSubscription", variables = "SubscriptionVariables")]
pub struct TestMeasurementSubscription {
    #[arguments(homeId: $home_id)]
    pub(crate) test_measurement: Option<LiveMeasurement>,
}

#[derive(cynic::QueryVariables)]
pub struct SubscriptionVariables {
    pub(crate) home_id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct LiveMeasurement {
    pub timestamp: String,
    pub power: f64,
    pub accumulated_consumption: f64,
    pub accumulated_consumption_last_hour: f64,
    pub accumulated_production: f64,
    pub accumulated_production_last_hour: f64,
    pub accumulated_cost: Option<f64>,
    pub accumulated_reward: Option<f64>,
}

pub trait SubscriptionQuery: QueryFragment + for<'de> serde::Deserialize<'de> + 'static {}

impl SubscriptionQuery for LiveMeasurementSubscription {}
impl SubscriptionQuery for TestMeasurementSubscription {}
