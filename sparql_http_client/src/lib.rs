pub mod client;
pub mod query;
pub mod response;

pub use client::{Endpoint, SparqlClient, UserAgent};
pub use query::{
    AskQuery, AskQueryString, QueryString, QueryStringError, QueryType, SelectQuery,
    SelectQueryString, SparqlQuery,
};
pub use response::QueryResponse;
pub use sparql_http_client_macros::query;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn tokio() {
        sleep(Duration::from_millis(500)).await;
    }

    #[tokio::test]
    async fn reqwest() -> anyhow::Result<()> {
        reqwest::get("https://httpbin.org/ip")
            .await?
            .json::<HashMap<String, String>>()
            .await?;

        Ok(())
    }
}
