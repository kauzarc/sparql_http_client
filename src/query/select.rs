use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use spargebra::SparqlParser;

use crate::{client, response};
use super::{QueryString, QueryStringError, QueryType};

#[derive(Clone)]
pub struct SelectQueryString(Arc<str>);

impl FromStr for SelectQueryString {
    type Err = QueryStringError;

    fn from_str(s: &str) -> Result<Self, QueryStringError> {
        let q = SparqlParser::new().parse_query(s)?;
        match QueryType::from(&q) {
            QueryType::Select => Ok(Self(q.to_string().into())),
            provided => Err(QueryStringError::WrongKind {
                expected: QueryType::Select,
                provided,
            }),
        }
    }
}

impl Deref for SelectQueryString {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl QueryString for SelectQueryString {
    type Query<'a> = SelectQuery<'a>;

    fn new_unchecked(s: &str) -> Self {
        Self(Arc::from(s))
    }

    fn build<'a>(self, endpoint: &'a client::Endpoint) -> SelectQuery<'a> {
        SelectQuery { endpoint, query: self }
    }
}

pub struct SelectQuery<'a> {
    endpoint: &'a client::Endpoint,
    query: SelectQueryString,
}

impl<'a> SelectQuery<'a> {
    pub async fn run(self) -> Result<response::SelectQueryResponse, reqwest::Error> {
        self.endpoint
            .request()
            .form(&[("query", &*self.query)])
            .send()
            .await?
            .json::<response::SelectQueryResponse>()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run() -> anyhow::Result<()> {
        let qs: SelectQueryString = r#"
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

            SELECT ?obj WHERE {
                ?sub ?pred ?obj .
            } LIMIT 3
        "#.parse()?;

        client::Endpoint::new(
            client::SparqlClient::default(),
            "https://query.wikidata.org/bigdata/namespace/wdq/sparql",
        )
        .build_query(qs)
        .run()
        .await?;

        Ok(())
    }
}
