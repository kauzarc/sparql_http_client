use std::{marker, string};

#[derive(Debug, thiserror::Error)]
#[error("An error occured durring query: {0}.")]
pub struct QueryError(#[from] reqwest::Error);

pub struct SparqlClient<ReturType> {
    endpoint: string::String,
    marker: marker::PhantomData<ReturType>,
}

impl<ReturType> SparqlClient<ReturType> {
    pub fn new<T: string::ToString>(endpoint: &T) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            marker: marker::PhantomData,
        }
    }

    pub async fn query(&self, query_text: &str) -> Result<(), QueryError> {
        let _ = query_text;
        reqwest::get(&self.endpoint).await?;
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::collections;
    use tokio::time;

    #[tokio::test]
    async fn test_tokio() {
        time::sleep(time::Duration::from_millis(500)).await;
    }

    #[tokio::test]
    async fn test_reqwest() -> anyhow::Result<()> {
        reqwest::get("https://httpbin.org/ip")
            .await?
            .json::<collections::HashMap<String, String>>()
            .await?;

        Ok(())
    }
}
