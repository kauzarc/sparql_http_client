mod ask;
mod select;
mod select_stream;

pub use ask::*;
pub use select::*;
pub use select_stream::*;

use serde::de::DeserializeOwned;

/// Marker trait for types that can be deserialized from a SPARQL query response.
///
/// Implemented by [`AskQueryResponse`].
/// Used as the associated type bound in [`QueryString::Response`](crate::QueryString::Response).
pub trait QueryResponse: DeserializeOwned {}

impl QueryResponse for AskQueryResponse {}
