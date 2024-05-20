use super::*;
use crate::{client, error, response};

pub struct SelectQuery<'a> {
    endpoint: &'a client::Endpoint<'a>,
    query: spargebra::Query,
}

impl<'a> SelectQuery<'a> {
    pub(crate) fn new<Q>(
        endpoint: &'a client::Endpoint<'a>,
        query: Q,
    ) -> Result<SelectQuery<'a>, error::QueryError>
    where
        Q: TryInto<spargebra::Query, Error = spargebra::ParseError>,
    {
        let query = query.try_into()?;
        let query_type = QueryType::from(&query);

        if let QueryType::Select = query_type {
            Ok(Self { endpoint, query })
        } else {
            Err(error::QueryError::TypeError {
                expected: QueryType::Select,
                provided: query_type,
            })
        }
    }

    pub async fn run(self) -> Result<response::SelectQueryResponse, error::QueryError> {
        Ok(self
            .endpoint
            .request()
            .form(&[("query", self.query.to_string())])
            .send()
            .await?
            .json::<response::SelectQueryResponse>()
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_client() -> client::SparqlClient {
        client::SparqlClient::new(client::UserAgent {
            name: "unit-test".into(),
            version: clap::crate_version!().into(),
            contact: "https://github.com/kauzarc/sparql_http_client".into(),
        })
    }

    #[tokio::test]
    async fn run() -> anyhow::Result<()> {
        test_client()
            .endpoint("https://query.wikidata.org/bigdata/namespace/wdq/sparql")
            .select(
                r#"
                PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
                PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

                SELECT ?obj WHERE {
                    ?sub ?pred ?obj .
                } LIMIT 3
                "#,
            )?
            .run()
            .await?;

        Ok(())
    }
}
