use cynic::Id;

use crate::gql::schema;

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub(crate) struct WebsocketSubscriptionUrl {
    pub(crate) viewer: WebsocketSubscriptionUrlViewer,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Viewer")]
pub(crate) struct WebsocketSubscriptionUrlViewer {
    pub(crate) websocket_subscription_url: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub(crate) struct Homes {
    pub(crate) viewer: HomesViewer,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Viewer")]
pub(crate) struct HomesViewer {
    pub(crate) homes: Vec<Option<Home>>,
}

#[derive(cynic::QueryFragment, Debug)]
pub(crate) struct Home {
    pub(crate) id: Id,
}
