pub mod ask;
pub mod error;
pub mod select;

pub use ask::AskQueryString;
pub use error::QueryStringError;
pub use select::SelectQueryString;

use std::fmt;
use std::str::FromStr;

use spargebra::Query;

use crate::client::Endpoint;

pub trait QueryString: Sized + Clone + FromStr<Err = QueryStringError> {
    type Query<'a>;

    #[doc(hidden)]
    fn new_unchecked(s: &str) -> Self;
    fn build<'a>(self, endpoint: &'a Endpoint) -> Self::Query<'a>;
}

#[derive(Debug)]
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
