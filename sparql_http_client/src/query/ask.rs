use std::fmt;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use spargebra::SparqlParser;

use super::{QueryString, QueryStringError, QueryType};

/// An owned, validated, normalized ASK query string.
///
/// Parse from a `&str` at runtime via [`str::parse`]:
///
/// ```
/// use sparql_http_client::AskQueryString;
///
/// let qs: AskQueryString =
///     "ASK { <http://example.org/> a <http://example.org/Thing> }".parse().unwrap();
/// ```
///
/// Passing the wrong query kind returns a [`QueryStringError::WrongKind`](crate::QueryStringError::WrongKind):
///
/// ```
/// use sparql_http_client::AskQueryString;
///
/// let result = "SELECT ?s WHERE { ?s ?p ?o }".parse::<AskQueryString>();
/// assert!(result.is_err());
/// ```
///
/// [`Deref`] and [`Display`](std::fmt::Display) both yield the normalised query string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl fmt::Display for AskQueryString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

impl QueryString for AskQueryString {
    fn new_unchecked(s: &str) -> Self {
        Self(Arc::from(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{Endpoint, SparqlClient};

    #[tokio::test]
    async fn run() -> anyhow::Result<()> {
        let qs: AskQueryString = r#"
            PREFIX wd: <http://www.wikidata.org/entity/>
            PREFIX wdt: <http://www.wikidata.org/prop/direct/>

            ASK { wd:Q243 wdt:P31 wd:Q570116 }
        "#
        .parse()?;

        Endpoint::new(
            SparqlClient::default(),
            "https://query.wikidata.org/bigdata/namespace/wdq/sparql",
        )
        .build_query(qs)
        .run()
        .await?;

        Ok(())
    }
}
