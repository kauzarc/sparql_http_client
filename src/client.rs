use crate::{error, responses};

#[derive(Debug)]
pub struct SparqlClient(reqwest::Client);

impl SparqlClient {
    pub fn new() -> Self {
        Self(reqwest::Client::new())
    }

    pub async fn query<U>(
        &self,
        endpoint: U,
        query: &str,
    ) -> Result<responses::QueryResponse, error::Error>
    where
        U: reqwest::IntoUrl,
    {
        Ok(self
            .0
            .post(endpoint)
            .form(&[("query", query)])
            .send()
            .await?
            .json::<responses::QueryResponse>()
            .await?)
    }
}
