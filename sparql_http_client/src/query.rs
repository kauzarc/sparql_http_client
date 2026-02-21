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

pub trait QueryString:
    Sized + Clone + Deref<Target = str> + FromStr<Err = QueryStringError>
{
    type Response: QueryResponse;

    #[doc(hidden)]
    fn new_unchecked(s: &str) -> Self;

    fn build(self, endpoint: Endpoint) -> SparqlQuery<Self> {
        SparqlQuery {
            endpoint,
            query: self,
        }
    }
}

#[derive(Debug)]
pub struct SparqlQuery<Q> {
    endpoint: Endpoint,
    query: Q,
}

impl<Q: QueryString> SparqlQuery<Q> {
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

pub type SelectQuery = SparqlQuery<SelectQueryString>;
pub type AskQuery = SparqlQuery<AskQueryString>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Select,
    Construct,
    Describe,
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
