#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("An error occured durring query: {0}.")]
    QueryError(#[from] reqwest::Error),
}
