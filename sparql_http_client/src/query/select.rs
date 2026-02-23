use std::fmt;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use spargebra::SparqlParser;

use super::{QueryString, QueryStringError, QueryType};

/// An owned, validated, normalized SELECT query string.
///
/// Parse from a `&str` at runtime via [`str::parse`]:
///
/// ```
/// use sparql_http_client::SelectQueryString;
///
/// let qs: SelectQueryString = "SELECT ?s WHERE { ?s ?p ?o }".parse().unwrap();
/// ```
///
/// Passing the wrong query kind returns a [`QueryStringError::WrongKind`](crate::QueryStringError::WrongKind):
///
/// ```
/// use sparql_http_client::SelectQueryString;
///
/// let result = "ASK { <http://example.org/> a <http://example.org/Thing> }"
///     .parse::<SelectQueryString>();
/// assert!(result.is_err());
/// ```
///
/// The stored string is the *normalised* form produced by the SPARQL parser,
/// which may differ from the original in whitespace and casing.
/// [`Deref`] and [`Display`](std::fmt::Display) both yield the normalised string:
///
/// ```
/// use sparql_http_client::SelectQueryString;
///
/// let qs: SelectQueryString = "select ?s where { ?s ?p ?o }".parse().unwrap();
/// assert!(qs.to_string().starts_with("SELECT"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl fmt::Display for SelectQueryString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

impl QueryString for SelectQueryString {
    fn new_unchecked(s: &str) -> Self {
        Self(Arc::from(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{Endpoint, SparqlClient};

    const WIKIDATA: &str = "https://query.wikidata.org/bigdata/namespace/wdq/sparql";

    const QUERY: &str = r#"
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?obj WHERE {
            ?sub ?pred ?obj .
        } LIMIT 3
    "#;

    #[tokio::test]
    async fn run_stream() -> anyhow::Result<()> {
        use futures_util::StreamExt;

        let qs: SelectQueryString = QUERY.parse()?;
        let stream = Endpoint::new(SparqlClient::default(), WIKIDATA)
            .build_query(qs)
            .run()
            .await?;

        assert!(!stream.vars.is_empty());

        let mut rows = std::pin::pin!(stream.into_rows());
        while let Some(row) = rows.next().await {
            row?;
        }

        Ok(())
    }
}
