mod ask;
mod select;

pub use ask::*;
pub use select::*;

use serde::de::DeserializeOwned;

pub trait QueryResponse: DeserializeOwned {}

impl QueryResponse for SelectQueryResponse {}
impl QueryResponse for AskQueryResponse {}
