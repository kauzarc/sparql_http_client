use crate::{error, responses};
use reqwest::header;
use std::string;

#[derive(Debug)]
pub struct UserAgent {
    pub name: string::String,
    pub version: string::String,
    pub contact: string::String,
}

impl UserAgent {
    fn header_value(&self) -> header::HeaderValue {
        header::HeaderValue::from_str(&format!(
            "{}/{} ({}) {}/{}",
            self.name,
            self.version,
            self.contact,
            clap::crate_name!(),
            clap::crate_version!()
        ))
        .expect("valid charcters")
    }
}

#[derive(Debug)]
pub struct SparqlClient {
    inner: reqwest::Client,
    agent: UserAgent,
}

impl SparqlClient {
    pub fn new(user_agent: UserAgent) -> Self {
        Self {
            inner: reqwest::Client::new(),
            agent: user_agent,
        }
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
            .inner
            .post(endpoint)
            .header(
                header::ACCEPT,
                header::HeaderValue::from_static("application/sparql-results+json"),
            )
            .header(header::USER_AGENT, self.agent.header_value())
            .form(&[("query", query)])
            .send()
            .await?
            .json::<responses::QueryResponse>()
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_client() -> SparqlClient {
        SparqlClient::new(UserAgent {
            name: "unit-test".into(),
            version: clap::crate_version!().into(),
            contact: "https://github.com/kauzarc/sparql_http_client".into(),
        })
    }

    #[tokio::test]
    async fn query() -> anyhow::Result<()> {
        test_client()
            .query(
                "https://query.wikidata.org/bigdata/namespace/wdq/sparql",
                r#"
                PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
                PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

                SELECT ?obj WHERE {
                    ?sub ?pred ?obj .
                } LIMIT 3
                "#,
            )
            .await?;

        Ok(())
    }
}
