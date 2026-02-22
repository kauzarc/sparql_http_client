use std::collections::HashMap;
use std::io;
use std::pin::Pin;

use async_stream::stream;
use csv_async::{AsyncReaderBuilder, StringRecord};
use futures_util::{stream::Stream, StreamExt};
use tokio_util::io::StreamReader;

use super::select::{LiteralType, RDFTerm, RDFType, SelectHead};

/// Error produced by a [`SelectQueryStream`].
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
/// Returned by [`SparqlQuery<SelectQueryString>::run_stream`](crate::SparqlQuery::run_stream).
/// The [`head`](SelectQueryStream::head) field is populated as soon as the first
/// line of the response is received. Rows are then yielded one at a time via
/// [`into_rows`](SelectQueryStream::into_rows) as they arrive over the network.
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
///     .run_stream()
///     .await?;
///
/// println!("vars: {:?}", stream.head.vars);
///
/// let mut rows = std::pin::pin!(stream.into_rows());
/// while let Some(row) = rows.next().await {
///     println!("{:?}", row?);
/// }
/// # Ok(()) }
/// ```
pub struct SelectQueryStream {
    /// The query head containing the projected variable names.
    pub head: SelectHead,
    rows: Pin<Box<dyn Stream<Item = Result<Row, StreamError>> + Send>>,
}

impl SelectQueryStream {
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

        let head = SelectHead {
            vars: vars.clone(),
            link: None,
        };

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
                        for (var, cell) in vars.iter().zip(record.iter()) {
                            match parse_tsv_cell(cell) {
                                Err(e) => {
                                    yield Err(e);
                                    return;
                                }
                                Ok(None) => {}
                                Ok(Some(term)) => {
                                    row.insert(var.clone(), term);
                                }
                            }
                        }
                        yield Ok(row);
                    }
                }
            }
        });

        Ok(Self { head, rows })
    }

    /// Consumes this value and returns the row stream.
    ///
    /// Use [`head`](SelectQueryStream::head) before calling this if you need
    /// the projected variable names.
    pub fn into_rows(self) -> impl Stream<Item = Result<Row, StreamError>> {
        self.rows
    }
}

fn parse_tsv_cell(s: &str) -> Result<Option<RDFTerm>, StreamError> {
    if s.is_empty() {
        return Ok(None);
    }

    // IRI: <http://example.org/>
    if let Some(iri) = s.strip_prefix('<').and_then(|s| s.strip_suffix('>')) {
        return Ok(Some(RDFTerm {
            value: iri.into(),
            kind: RDFType::IRI,
        }));
    }

    // Blank node: _:b0
    if let Some(id) = s.strip_prefix("_:") {
        return Ok(Some(RDFTerm {
            value: id.into(),
            kind: RDFType::BlankNode,
        }));
    }

    // Literal: "..."  "..."@lang  "..."^^<datatype>
    if s.starts_with('"') {
        let (value, rest) = parse_quoted_str(s)?;
        let kind = if let Some(lang) = rest.strip_prefix('@') {
            RDFType::Literal {
                kind: LiteralType::WithLanguage { lang: lang.into() },
            }
        } else if let Some(dt) = rest.strip_prefix("^^") {
            let datatype = dt
                .strip_prefix('<')
                .and_then(|s| s.strip_suffix('>'))
                .ok_or_else(|| StreamError::Parse(format!("invalid datatype IRI: {dt:?}")))?;
            RDFType::Literal {
                kind: LiteralType::WithDataType {
                    datatype: datatype.into(),
                },
            }
        } else if rest.is_empty() {
            RDFType::Literal {
                kind: LiteralType::Simple {},
            }
        } else {
            return Err(StreamError::Parse(format!(
                "unexpected suffix after literal: {rest:?}"
            )));
        };
        return Ok(Some(RDFTerm {
            value: value.into(),
            kind,
        }));
    }

    Err(StreamError::Parse(format!("unrecognized TSV cell: {s:?}")))
}

/// Parses a quoted N-Triples-style string starting at `s[0] == '"'`.
/// Returns the unescaped content and the remaining suffix after the closing `"`.
fn parse_quoted_str(s: &str) -> Result<(String, &str), StreamError> {
    let inner = &s[1..]; // skip opening '"'
    let mut value = String::new();
    let mut chars = inner.char_indices();
    loop {
        match chars.next() {
            None => return Err(StreamError::Parse("unterminated string literal".into())),
            Some((i, '"')) => {
                return Ok((value, &inner[i + 1..]));
            }
            Some((_, '\\')) => match chars.next() {
                None => return Err(StreamError::Parse("unterminated escape sequence".into())),
                Some((_, 'n')) => value.push('\n'),
                Some((_, 'r')) => value.push('\r'),
                Some((_, 't')) => value.push('\t'),
                Some((_, '"')) => value.push('"'),
                Some((_, '\'')) => value.push('\''),
                Some((_, '\\')) => value.push('\\'),
                Some((_, 'u')) => {
                    let hex: String = chars.by_ref().take(4).map(|(_, c)| c).collect();
                    if hex.len() != 4 {
                        return Err(StreamError::Parse(format!(
                            "incomplete \\u escape: {hex:?}"
                        )));
                    }
                    let code = u32::from_str_radix(&hex, 16)
                        .map_err(|_| StreamError::Parse(format!("invalid \\u escape: {hex:?}")))?;
                    let ch = char::from_u32(code).ok_or_else(|| {
                        StreamError::Parse(format!("invalid unicode codepoint U+{code:04X}"))
                    })?;
                    value.push(ch);
                }
                Some((_, 'U')) => {
                    let hex: String = chars.by_ref().take(8).map(|(_, c)| c).collect();
                    if hex.len() != 8 {
                        return Err(StreamError::Parse(format!(
                            "incomplete \\U escape: {hex:?}"
                        )));
                    }
                    let code = u32::from_str_radix(&hex, 16)
                        .map_err(|_| StreamError::Parse(format!("invalid \\U escape: {hex:?}")))?;
                    let ch = char::from_u32(code).ok_or_else(|| {
                        StreamError::Parse(format!("invalid unicode codepoint U+{code:08X}"))
                    })?;
                    value.push(ch);
                }
                Some((_, c)) => {
                    return Err(StreamError::Parse(format!("unknown escape \\{c}")));
                }
            },
            Some((_, c)) => value.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        assert_eq!(parse_tsv_cell("").unwrap(), None);
    }

    #[test]
    fn parse_iri() {
        let term = parse_tsv_cell("<http://example.org/>").unwrap().unwrap();
        assert_eq!(&*term.value, "http://example.org/");
        assert_eq!(term.kind, RDFType::IRI);
    }

    #[test]
    fn parse_blank_node() {
        let term = parse_tsv_cell("_:b0").unwrap().unwrap();
        assert_eq!(&*term.value, "b0");
        assert_eq!(term.kind, RDFType::BlankNode);
    }

    #[test]
    fn parse_simple_literal() {
        let term = parse_tsv_cell("\"hello\"").unwrap().unwrap();
        assert_eq!(&*term.value, "hello");
        assert_eq!(
            term.kind,
            RDFType::Literal {
                kind: LiteralType::Simple {}
            }
        );
    }

    #[test]
    fn parse_lang_literal() {
        let term = parse_tsv_cell("\"hello\"@en").unwrap().unwrap();
        assert_eq!(&*term.value, "hello");
        assert_eq!(term.lang(), Some("en"));
    }

    #[test]
    fn parse_typed_literal() {
        let term = parse_tsv_cell("\"42\"^^<http://www.w3.org/2001/XMLSchema#integer>")
            .unwrap()
            .unwrap();
        assert_eq!(&*term.value, "42");
        assert_eq!(
            term.datatype(),
            Some("http://www.w3.org/2001/XMLSchema#integer")
        );
    }

    #[test]
    fn parse_escaped_quotes_in_literal() {
        let term = parse_tsv_cell(r#""hello \"world\"""#).unwrap().unwrap();
        assert_eq!(&*term.value, r#"hello "world""#);
    }

    #[test]
    fn parse_unicode_escape() {
        let term = parse_tsv_cell(r#""\u0041""#).unwrap().unwrap();
        assert_eq!(&*term.value, "A");
    }
}
