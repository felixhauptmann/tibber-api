use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "./graphql/query/wsSubscriptionUrl.graphql",
    schema_path = "./graphql/schema.json",
    response_derives = "Debug"
)]
pub struct WSSubscriptionUrl;

#[derive(GraphQLQuery)]
#[graphql(
query_path = "./graphql/query/homes.graphql",
schema_path = "./graphql/schema.json",
response_derives = "Debug"
)]
pub struct Homes;
