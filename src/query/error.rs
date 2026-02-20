use spargebra::SparqlSyntaxError;

use super::QueryType;

#[derive(Debug, thiserror::Error)]
pub enum QueryStringError {
    #[error("syntax error: {0}")]
    Syntax(#[from] SparqlSyntaxError),

    #[error("expected {expected} query but got {provided}")]
    WrongKind {
        expected: QueryType,
        provided: QueryType,
    },
}
