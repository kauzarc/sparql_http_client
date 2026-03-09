use std::str::FromStr;

use serde::{de, Deserialize, Deserializer};
use thiserror::Error;

/// Error returned when parsing a TSV-encoded RDF term fails.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseTermError {
    #[error("unrecognized TSV cell: {0:?}")]
    UnrecognizedCell(Box<str>),
    #[error("unterminated string literal")]
    UnterminatedLiteral,
    #[error("unterminated escape sequence")]
    UnterminatedEscape,
    #[error("unknown escape sequence: \\{0}")]
    UnknownEscape(char),
    #[error("incomplete unicode escape: {0:?}")]
    IncompleteUnicodeEscape(Box<str>),
    #[error("invalid unicode escape: {0:?}")]
    InvalidUnicodeEscape(Box<str>),
    #[error("invalid unicode codepoint: U+{0:04X}")]
    InvalidUnicodeCodepoint(u32),
    #[error("invalid datatype IRI: {0:?}")]
    InvalidDatatypeIri(Box<str>),
    #[error("unexpected suffix after literal: {0:?}")]
    UnexpectedLiteralSuffix(Box<str>),
}

/// A single RDF term: the value bound to a variable in one result row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RDFTerm {
    pub value: Box<str>,
    pub kind: RDFType,
}

impl RDFTerm {
    pub fn is_iri(&self) -> bool {
        matches!(self.kind, RDFType::IRI)
    }

    pub fn is_literal(&self) -> bool {
        matches!(self.kind, RDFType::Literal(_))
    }

    pub fn is_blank_node(&self) -> bool {
        matches!(self.kind, RDFType::BlankNode)
    }

    /// Returns the language tag if this is a language-tagged literal.
    pub fn lang(&self) -> Option<&str> {
        match &self.kind {
            RDFType::Literal(LiteralType::Lang(lang)) => Some(lang),
            _ => None,
        }
    }

    /// Returns the datatype IRI if this is a datatyped literal.
    pub fn datatype(&self) -> Option<&str> {
        match &self.kind {
            RDFType::Literal(LiteralType::Datatype(dt)) => Some(dt),
            _ => None,
        }
    }
}

impl RDFTerm {
    /// Caller must guarantee `s` starts with `<` and ends with `>`.
    fn from_bracketed_iri(s: &str) -> Self {
        RDFTerm {
            value: s[1..s.len() - 1].into(),
            kind: RDFType::IRI,
        }
    }

    /// Caller must guarantee `s` starts with `_:`.
    fn from_prefixed_blank_node(s: &str) -> Self {
        RDFTerm {
            value: s[2..].into(),
            kind: RDFType::BlankNode,
        }
    }

    /// Caller must guarantee `s` starts with `"`.
    fn from_quoted_literal(s: &str) -> Result<Self, ParseTermError> {
        let (value, rest) = parse_quoted_str(s)?;
        let literal_type = match rest.as_bytes() {
            [b'@', ..] => LiteralType::Lang(rest[1..].into()),
            [b'^', b'^', ..] => {
                let dt = &rest[2..];
                let datatype = match dt.as_bytes() {
                    [b'<', .., b'>'] => &dt[1..dt.len() - 1],
                    _ => return Err(ParseTermError::InvalidDatatypeIri(dt.into())),
                };
                LiteralType::Datatype(datatype.into())
            }
            [] => LiteralType::Plain,
            _ => return Err(ParseTermError::UnexpectedLiteralSuffix(rest.into())),
        };
        Ok(RDFTerm {
            value: value.into(),
            kind: RDFType::Literal(literal_type),
        })
    }
}

impl FromStr for RDFTerm {
    type Err = ParseTermError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.as_bytes() {
            [b'<', .., b'>'] => Ok(Self::from_bracketed_iri(s)),
            [b'_', b':', ..] => Ok(Self::from_prefixed_blank_node(s)),
            [b'"', ..] => Self::from_quoted_literal(s),
            _ => Err(ParseTermError::UnrecognizedCell(s.into())),
        }
    }
}

impl<'de> Deserialize<'de> for RDFTerm {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(RDFTermVisitor)
    }
}

struct RDFTermVisitor;

impl<'de> de::Visitor<'de> for RDFTermVisitor {
    type Value = RDFTerm;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a TSV-encoded RDF term")
    }

    fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
        s.parse().map_err(E::custom)
    }
}

/// Parses a quoted N-Triples-style string starting at `s[0] == '"'`.
/// Returns the unescaped content and the remaining suffix after the closing `"`.
fn parse_quoted_str(s: &str) -> Result<(String, &str), ParseTermError> {
    let inner = &s[1..]; // skip opening '"'
    let mut value = String::new();
    let mut chars = inner.char_indices();
    loop {
        match chars.next() {
            None => return Err(ParseTermError::UnterminatedLiteral),
            Some((i, '"')) => return Ok((value, &inner[i + 1..])),
            Some((_, '\\')) => match chars.next() {
                None => return Err(ParseTermError::UnterminatedEscape),
                Some((_, 'n')) => value.push('\n'),
                Some((_, 'r')) => value.push('\r'),
                Some((_, 't')) => value.push('\t'),
                Some((_, '"')) => value.push('"'),
                Some((_, '\'')) => value.push('\''),
                Some((_, '\\')) => value.push('\\'),
                Some((_, 'u')) => value.push(parse_unicode_escape(&mut chars, 4)?),
                Some((_, 'U')) => value.push(parse_unicode_escape(&mut chars, 8)?),
                Some((_, c)) => return Err(ParseTermError::UnknownEscape(c)),
            },
            Some((_, c)) => value.push(c),
        }
    }
}

fn parse_unicode_escape(
    chars: &mut impl Iterator<Item = (usize, char)>,
    n: usize,
) -> Result<char, ParseTermError> {
    let hex: String = chars.by_ref().take(n).map(|(_, c)| c).collect();
    if hex.len() != n {
        return Err(ParseTermError::IncompleteUnicodeEscape(hex.into()));
    }
    let code = u32::from_str_radix(&hex, 16)
        .map_err(|_| ParseTermError::InvalidUnicodeEscape(hex.into()))?;
    char::from_u32(code).ok_or(ParseTermError::InvalidUnicodeCodepoint(code))
}

/// The type of an RDF term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RDFType {
    /// An IRI.
    IRI,
    /// A literal, with its optional annotation.
    Literal(LiteralType),
    /// A blank node.
    BlankNode,
}

/// The annotation carried by an RDF literal.
///
/// These three cases are mutually exclusive per the RDF specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralType {
    /// A plain literal with no language tag or datatype.
    Plain,
    /// A language-tagged literal (`"..."@lang`).
    Lang(Box<str>),
    /// A datatyped literal (`"..."^^<datatype>`).
    Datatype(Box<str>),
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    fn parse_term(s: &str) -> serde_json::Result<RDFTerm> {
        serde_json::from_value(Value::String(s.to_string()))
    }

    #[test]
    fn parse_iri() {
        let term = parse_term("<http://example.org/>").unwrap();
        assert_eq!(&*term.value, "http://example.org/");
        assert_eq!(term.kind, RDFType::IRI);
    }

    #[test]
    fn parse_blank_node() {
        let term = parse_term("_:b0").unwrap();
        assert_eq!(&*term.value, "b0");
        assert_eq!(term.kind, RDFType::BlankNode);
    }

    #[test]
    fn parse_simple_literal() {
        let term = parse_term(r#""hello""#).unwrap();
        assert_eq!(&*term.value, "hello");
        assert_eq!(term.kind, RDFType::Literal(LiteralType::Plain));
    }

    #[test]
    fn parse_lang_literal() {
        let term = parse_term(r#""hello"@en"#).unwrap();
        assert_eq!(&*term.value, "hello");
        assert_eq!(term.lang(), Some("en"));
    }

    #[test]
    fn parse_typed_literal() {
        let term = parse_term(r#""42"^^<http://www.w3.org/2001/XMLSchema#integer>"#).unwrap();
        assert_eq!(&*term.value, "42");
        assert_eq!(
            term.datatype(),
            Some("http://www.w3.org/2001/XMLSchema#integer")
        );
    }

    #[test]
    fn parse_escaped_quotes_in_literal() {
        let term = parse_term(r#""hello \"world\"""#).unwrap();
        assert_eq!(&*term.value, r#"hello "world""#);
    }

    #[test]
    fn parse_unicode_escape() {
        let term = parse_term(r#""\u0041""#).unwrap();
        assert_eq!(&*term.value, "A");
    }
}
