pub mod ask;
pub mod error;
pub mod select;

pub use ask::AskQueryString;
pub use error::QueryStringError;
pub use select::SelectQueryString;

use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use spargebra::Query;

use crate::client::Endpoint;
use crate::response::{AskQueryResponse, SelectQueryResponse, StreamError};

/// An owned, validated, normalized SPARQL query string.
///
/// Implementors hold the query text. The two built-in implementations are
/// [`SelectQueryString`] and [`AskQueryString`].
///
/// Obtain an instance by parsing at runtime with [`str::parse`]:
///
/// ```
/// use sparql_http_client::SelectQueryString;
///
/// let qs: SelectQueryString = "SELECT ?s WHERE { ?s ?p ?o }".parse().unwrap();
/// ```
///
/// or at compile time via the [`query!`](crate::query) macro, which also
/// binds the query to an [`Endpoint`] in one step.
pub trait QueryString:
    Sized + Clone + Deref<Target = str> + FromStr<Err = QueryStringError>
{
    #[doc(hidden)]
    fn new_unchecked(s: &str) -> Self;

    /// Binds this query to `endpoint`, producing an executable [`SparqlQuery`].
    ///
    /// Prefer [`Endpoint::build_query`], which reads more naturally.
    fn build(self, endpoint: Endpoint) -> SparqlQuery<Self> {
        SparqlQuery {
            endpoint,
            query: self,
        }
    }
}

/// An executable SPARQL query bound to an [`Endpoint`].
///
/// Created by [`Endpoint::build_query`] or the [`query!`](crate::query) macro.
///
/// - `SparqlQuery<SelectQueryString>`: call [`run_stream`](SparqlQuery::run_stream)
/// - `SparqlQuery<AskQueryString>`: call [`run`](SparqlQuery::run)
///
/// See also the type alias [`AskQuery`].
#[derive(Debug)]
pub struct SparqlQuery<Q> {
    endpoint: Endpoint,
    query: Q,
}

impl SparqlQuery<AskQueryString> {
    /// Sends the ASK query to the endpoint and deserializes the response.
    pub async fn run(self) -> Result<AskQueryResponse, reqwest::Error> {
        self.endpoint
            .request()
            .form(&[("query", &*self.query)])
            .send()
            .await?
            .json::<AskQueryResponse>()
            .await
    }
}

impl SparqlQuery<SelectQueryString> {
    /// Sends the query and streams result rows as they arrive over the network.
    ///
    /// The endpoint is asked for `text/tab-separated-values`; the
    /// [`vars`](SelectQueryResponse::vars) field is populated from the first line,
    /// then rows are yielded one at a time via [`SelectQueryResponse::into_rows`].
    pub async fn run(self) -> Result<SelectQueryResponse, StreamError> {
        let response = self
            .endpoint
            .request_tsv()
            .form(&[("query", &*self.query)])
            .send()
            .await?;
        SelectQueryResponse::from_response(response).await
    }
}

/// Type alias for an ASK query bound to an endpoint.
///
/// Returned by [`query!`](crate::query) for `ASK` statements, or by
/// [`Endpoint::build_query`] with an [`AskQueryString`].
pub type AskQuery = SparqlQuery<AskQueryString>;

/// The kind of a SPARQL query.
///
/// Used in [`QueryStringError::WrongKind`] to describe a mismatch between the
/// expected and actual query type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    /// A `SELECT` query, returning a table of variable bindings.
    Select,
    /// A `CONSTRUCT` query, returning an RDF graph. Not currently supported.
    Construct,
    /// A `DESCRIBE` query, returning an RDF graph. Not currently supported.
    Describe,
    /// An `ASK` query, returning a boolean.
    Ask,
}

impl From<&Query> for QueryType {
    fn from(q: &Query) -> Self {
        match q {
            Query::Select { .. } => Self::Select,
            Query::Construct { .. } => Self::Construct,
            Query::Describe { .. } => Self::Describe,
            Query::Ask { .. } => Self::Ask,
        }
    }
}

impl fmt::Display for QueryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Select => write!(f, "SELECT"),
            Self::Construct => write!(f, "CONSTRUCT"),
            Self::Describe => write!(f, "DESCRIBE"),
            Self::Ask => write!(f, "ASK"),
        }
    }
}
