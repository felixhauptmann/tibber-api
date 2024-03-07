use graphql_client::GraphQLQuery;

pub trait StreamQuery: GraphQLQuery + Unpin + Send + 'static {}

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "./graphql/subscription/measurement.graphql",
    schema_path = "./graphql/schema.json",
    response_derives = "Debug"
)]
pub struct LiveMeasurement;

impl StreamQuery for LiveMeasurement {}

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "./graphql/subscription/measurement.graphql",
    schema_path = "./graphql/schema.json",
    response_derives = "Debug"
)]
pub struct TestMeasurement;

impl StreamQuery for TestMeasurement {}