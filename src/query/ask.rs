use super::*;
use crate::{client, error, response};

pub struct AskQuery<'a> {
    endpoint: &'a client::Endpoint,
    query: spargebra::Query,
}

impl<'a> AskQuery<'a> {
    pub(crate) fn new(
        endpoint: &'a client::Endpoint,
        query: &str,
    ) -> Result<AskQuery<'a>, error::QueryError> {
        let query = spargebra::SparqlParser::new().parse_query(query)?;
        let query_type = QueryType::from(&query);

        if let QueryType::Ask = query_type {
            Ok(Self { endpoint, query })
        } else {
            Err(error::QueryError::TypeError {
                expected: QueryType::Ask,
                provided: query_type,
            })
        }
    }

    pub async fn run(self) -> Result<response::AskQueryResponse, error::QueryError> {
        Ok(self
            .endpoint
            .request()
            .form(&[("query", self.query.to_string())])
            .send()
            .await?
            .json::<response::AskQueryResponse>()
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
        .ask(
            r#"
            PREFIX wd: <http://www.wikidata.org/entity/>
            PREFIX wdt: <http://www.wikidata.org/prop/direct/>

            ASK { wd:Q243 wdt:P31 wd:Q570116 }
            "#,
        )?
        .run()
        .await?;

        Ok(())
    }
}
