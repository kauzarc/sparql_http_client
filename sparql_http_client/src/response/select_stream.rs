use std::collections::HashMap;
use std::io;
use std::pin::Pin;

use async_stream::stream;
use csv_async::{AsyncReaderBuilder, StringRecord};
use futures_util::{stream::Stream, StreamExt};
use serde::{Deserialize, de::IntoDeserializer, de::value::Error as DeError};
use tokio_util::io::StreamReader;

use super::select::RDFTerm;

/// Error produced by a [`SelectQueryResponse`] stream.
#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    /// The HTTP request or network transfer failed.
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    /// The response body could not be parsed as SPARQL TSV.
    #[error("parse error: {0}")]
    Parse(String),
}

/// A single result row: variable name → RDF term.
///
/// Variables that are unbound in a given row are absent from the map.
pub type Row = HashMap<Box<str>, RDFTerm>;

/// A streaming SPARQL SELECT response received as tab-separated values.
///
/// Returned by [`SparqlQuery<SelectQueryString>::run`](crate::SparqlQuery::run).
/// The [`vars`](SelectQueryResponse::vars) field is populated as soon as the first
/// line of the response is received. Rows are then yielded one at a time via
/// [`into_rows`](SelectQueryResponse::into_rows) as they arrive over the network.
///
/// # Example
///
/// ```no_run
/// use futures_util::StreamExt;
/// use sparql_http_client::{Endpoint, SparqlClient, SelectQueryString};
///
/// # #[tokio::main] async fn main() -> anyhow::Result<()> {
/// let qs: SelectQueryString = "SELECT ?s WHERE { ?s ?p ?o } LIMIT 10".parse()?;
/// let stream = Endpoint::new(SparqlClient::default(), "https://example.org/sparql")
///     .build_query(qs)
///     .run()
///     .await?;
///
/// println!("vars: {:?}", stream.vars);
///
/// let mut rows = std::pin::pin!(stream.into_rows());
/// while let Some(row) = rows.next().await {
///     println!("{:?}", row?);
/// }
/// # Ok(()) }
/// ```
pub struct SelectQueryResponse {
    /// The projected variable names from the query's SELECT clause.
    pub vars: Box<[Box<str>]>,
    rows: Pin<Box<dyn Stream<Item = Result<Row, StreamError>> + Send>>,
}

impl SelectQueryResponse {
    pub(crate) async fn from_response(response: reqwest::Response) -> Result<Self, StreamError> {
        let byte_stream = response.bytes_stream().map(|r| r.map_err(io::Error::other));
        let stream_reader = StreamReader::new(byte_stream);
        let mut builder = AsyncReaderBuilder::new();
        builder.delimiter(b'\t');
        let mut csv_reader = builder.create_reader(stream_reader);

        let headers = csv_reader
            .headers()
            .await
            .map_err(|e| StreamError::Parse(e.to_string()))?
            .clone();

        let vars: Box<[Box<str>]> = headers
            .iter()
            .map(|h| h.trim_start_matches('?').into())
            .collect();

        let vars_stream = vars.clone();
        let rows = Box::pin(stream! {
            let mut record = StringRecord::new();
            loop {
                match csv_reader.read_record(&mut record).await {
                    Err(e) => {
                        yield Err(StreamError::Parse(e.to_string()));
                        return;
                    }
                    Ok(false) => break,
                    Ok(true) => {
                        let mut row = Row::new();
                        for (var, cell) in vars_stream.iter().zip(record.iter()) {
                            if cell.is_empty() {
                                continue;
                            }
                            match deserialize_cell(cell) {
                                Err(e) => {
                                    yield Err(StreamError::Parse(e.to_string()));
                                    return;
                                }
                                Ok(term) => {
                                    row.insert(var.clone(), term);
                                }
                            }
                        }
                        yield Ok(row);
                    }
                }
            }
        });

        Ok(Self { vars, rows })
    }

    /// Consumes this value and returns the row stream.
    ///
    /// Use [`vars`](SelectQueryResponse::vars) before calling this if you need
    /// the projected variable names.
    pub fn into_rows(self) -> impl Stream<Item = Result<Row, StreamError>> {
        self.rows
    }
}

fn deserialize_cell(s: &str) -> Result<RDFTerm, DeError> {
    RDFTerm::deserialize(s.into_deserializer())
}
