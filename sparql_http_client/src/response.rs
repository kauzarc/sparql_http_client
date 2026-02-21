//! SPARQL query response types.
//!
//! Each query string type has an associated response type:
//!
//! | Query | Response |
//! |---|---|
//! | [`SelectQueryString`](crate::SelectQueryString) | [`SelectQueryResponse`] |
//! | [`AskQueryString`](crate::AskQueryString) | [`AskQueryResponse`] |

mod ask;
mod select;

pub use ask::*;
pub use select::*;

use serde::de::DeserializeOwned;

/// Marker trait for types that can be deserialized from a SPARQL query response.
///
/// Implemented by [`SelectQueryResponse`] and [`AskQueryResponse`].
/// Used as the associated type bound in [`QueryString::Response`](crate::QueryString::Response).
pub trait QueryResponse: DeserializeOwned {}

impl QueryResponse for SelectQueryResponse {}
impl QueryResponse for AskQueryResponse {}
