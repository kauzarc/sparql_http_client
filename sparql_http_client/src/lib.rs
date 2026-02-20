pub mod client;
pub mod query;
pub mod response;

pub use client::{Endpoint, SparqlClient, UserAgent};
pub use query::{AskQuery, AskQueryString, QueryString, QueryStringError, QueryType, SelectQuery, SelectQueryString, SparqlQuery};
pub use response::QueryResponse;

#[cfg(test)]
mod tests {
    use std::collections;
    use tokio::time;

    #[tokio::test]
    async fn tokio() {
        time::sleep(time::Duration::from_millis(500)).await;
    }

    #[tokio::test]
    async fn reqwest() -> anyhow::Result<()> {
        reqwest::get("https://httpbin.org/ip")
            .await?
            .json::<collections::HashMap<String, String>>()
            .await?;

        Ok(())
    }
}
