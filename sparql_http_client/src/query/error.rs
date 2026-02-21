use spargebra::SparqlSyntaxError;

use super::QueryType;

/// An error returned when parsing a SPARQL query string fails.
///
/// Produced by [`SelectQueryString`](crate::SelectQueryString) and
/// [`AskQueryString`](crate::AskQueryString) when parsing via [`str::parse`].
///
/// # Example
///
/// ```
/// use sparql_http_client::{SelectQueryString, QueryStringError, QueryType};
///
/// let err = "ASK { <http://example.org/> a <http://example.org/Thing> }"
///     .parse::<SelectQueryString>()
///     .unwrap_err();
///
/// assert!(matches!(
///     err,
///     QueryStringError::WrongKind { expected: QueryType::Select, .. }
/// ));
/// ```
#[derive(Debug, thiserror::Error)]
pub enum QueryStringError {
    /// The query string contains a SPARQL syntax error.
    #[error("syntax error: {0}")]
    Syntax(#[from] SparqlSyntaxError),

    /// The query was syntactically valid but its kind did not match the expected type.
    ///
    /// For example, passing an ASK query to [`SelectQueryString`](crate::SelectQueryString).
    #[error("expected {expected} query but got {provided}")]
    WrongKind {
        expected: QueryType,
        provided: QueryType,
    },
}
