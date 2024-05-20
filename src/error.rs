#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error durring query: {0}.")]
    QueryError(#[from] reqwest::Error),
}
