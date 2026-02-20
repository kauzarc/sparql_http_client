use super::*;
use crate::{client, error, response};

pub struct SelectQuery<'a> {
    endpoint: &'a client::Endpoint,
    query: spargebra::Query,
}

impl<'a> SelectQuery<'a> {
    pub(crate) fn new(
        endpoint: &'a client::Endpoint,
        query: &str,
    ) -> Result<SelectQuery<'a>, error::QueryError> {
        let query = spargebra::SparqlParser::new().parse_query(query)?;
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

    #[tokio::test]
    async fn run() -> anyhow::Result<()> {
        client::Endpoint::new(
            client::SparqlClient::default(),
            "https://query.wikidata.org/bigdata/namespace/wdq/sparql",
        )
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
