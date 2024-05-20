use crate::query;

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("error during parsing: {0}")]
    ParseError(#[from] spargebra::ParseError),

    #[error(
        "type error: {:?} was expected but {:?} was provided when",
        provided,
        expected
    )]
    TypeError {
        expected: query::QueryType,
        provided: query::QueryType,
    },

    #[error("error in request: {0}.")]
    RequestError(#[from] reqwest::Error),
}
