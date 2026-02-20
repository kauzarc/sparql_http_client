use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use spargebra::SparqlParser;

use crate::{client, response};
use super::{QueryString, QueryStringError, QueryType};

#[derive(Clone)]
pub struct AskQueryString(Arc<str>);

impl FromStr for AskQueryString {
    type Err = QueryStringError;

    fn from_str(s: &str) -> Result<Self, QueryStringError> {
        let q = SparqlParser::new().parse_query(s)?;
        match QueryType::from(&q) {
            QueryType::Ask => Ok(Self(q.to_string().into())),
            provided => Err(QueryStringError::WrongKind {
                expected: QueryType::Ask,
                provided,
            }),
        }
    }
}

impl Deref for AskQueryString {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl QueryString for AskQueryString {
    type Query<'a> = AskQuery<'a>;

    fn new_unchecked(s: &str) -> Self {
        Self(Arc::from(s))
    }

    fn build<'a>(self, endpoint: &'a client::Endpoint) -> AskQuery<'a> {
        AskQuery { endpoint, query: self }
    }
}

pub struct AskQuery<'a> {
    endpoint: &'a client::Endpoint,
    query: AskQueryString,
}

impl<'a> AskQuery<'a> {
    pub async fn run(self) -> Result<response::AskQueryResponse, reqwest::Error> {
        self.endpoint
            .request()
            .form(&[("query", &*self.query)])
            .send()
            .await?
            .json::<response::AskQueryResponse>()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run() -> anyhow::Result<()> {
        let qs: AskQueryString = r#"
            PREFIX wd: <http://www.wikidata.org/entity/>
            PREFIX wdt: <http://www.wikidata.org/prop/direct/>

            ASK { wd:Q243 wdt:P31 wd:Q570116 }
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
