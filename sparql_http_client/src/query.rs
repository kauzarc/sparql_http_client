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
use crate::response::QueryResponse;

/// An owned, validated, normalized SPARQL query string.
///
/// Implementors hold the query text and declare the [`QueryResponse`] type
/// their query kind produces. The two built-in implementations are
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
    /// The response type produced when this query is executed.
    type Response: QueryResponse;

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
/// Call [`run`](SparqlQuery::run) to send the request and deserialize the response.
///
/// The type parameter `Q` determines the response type:
/// - `SparqlQuery<SelectQueryString>` → [`SelectQueryResponse`](crate::response::SelectQueryResponse)
/// - `SparqlQuery<AskQueryString>` → [`AskQueryResponse`](crate::response::AskQueryResponse)
///
/// See also the type aliases [`SelectQuery`] and [`AskQuery`].
#[derive(Debug)]
pub struct SparqlQuery<Q> {
    endpoint: Endpoint,
    query: Q,
}

impl<Q: QueryString> SparqlQuery<Q> {
    /// Sends the query to the endpoint and deserializes the response.
    ///
    /// # Errors
    ///
    /// Returns a [`reqwest::Error`] if the HTTP request fails or the response
    /// cannot be deserialized.
    pub async fn run(self) -> Result<Q::Response, reqwest::Error> {
        self.endpoint
            .request()
            .form(&[("query", &*self.query)])
            .send()
            .await?
            .json::<Q::Response>()
            .await
    }
}

/// Type alias for a SELECT query bound to an endpoint.
///
/// Returned by [`query!`](crate::query) for `SELECT` statements, or by
/// [`Endpoint::build_query`] with a [`SelectQueryString`].
pub type SelectQuery = SparqlQuery<SelectQueryString>;

/// Type alias for an ASK query bound to an endpoint.
///
/// Returned by [`query!`](crate::query) for `ASK` statements, or by
/// [`Endpoint::build_query`] with an [`AskQueryString`].
pub type AskQuery = SparqlQuery<AskQueryString>;

/// The kind of a SPARQL query.
///
/// Used in [`QueryStringError::WrongKind`] to describe a mismatch between the
/// expected and actual query type — for example, passing an ASK query string
/// where a SELECT is expected.
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
