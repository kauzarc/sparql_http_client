pub mod client;
pub mod query;
pub mod response;

pub use client::{Endpoint, SparqlClient, UserAgent};
pub use query::{AskQueryString, QueryString, QueryStringError, QueryType, SelectQueryString};

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
