use serde::{de, Deserialize, Deserializer};

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
        matches!(self.kind, RDFType::Literal { .. })
    }

    pub fn is_blank_node(&self) -> bool {
        matches!(self.kind, RDFType::BlankNode)
    }

    /// Returns the language tag if this is a language-tagged literal.
    pub fn lang(&self) -> Option<&str> {
        if let RDFType::Literal { lang, .. } = &self.kind {
            lang.as_deref()
        } else {
            None
        }
    }

    /// Returns the datatype IRI if this is a datatyped literal.
    pub fn datatype(&self) -> Option<&str> {
        if let RDFType::Literal { datatype, .. } = &self.kind {
            datatype.as_deref()
        } else {
            None
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
        // IRI: <http://example.org/>
        if let Some(iri) = s.strip_prefix('<').and_then(|s| s.strip_suffix('>')) {
            return Ok(RDFTerm {
                value: iri.into(),
                kind: RDFType::IRI,
            });
        }

        // Blank node: _:b0
        if let Some(id) = s.strip_prefix("_:") {
            return Ok(RDFTerm {
                value: id.into(),
                kind: RDFType::BlankNode,
            });
        }

        // Literal: "..."  "..."@lang  "..."^^<datatype>
        if s.starts_with('"') {
            let (value, rest) = parse_quoted_str(s).map_err(E::custom)?;
            let kind = if let Some(lang) = rest.strip_prefix('@') {
                RDFType::Literal {
                    lang: Some(lang.into()),
                    datatype: None,
                }
            } else if let Some(dt) = rest.strip_prefix("^^") {
                let datatype = dt
                    .strip_prefix('<')
                    .and_then(|s| s.strip_suffix('>'))
                    .ok_or_else(|| E::custom(format!("invalid datatype IRI: {dt:?}")))?;
                RDFType::Literal {
                    lang: None,
                    datatype: Some(datatype.into()),
                }
            } else if rest.is_empty() {
                RDFType::Literal {
                    lang: None,
                    datatype: None,
                }
            } else {
                return Err(E::custom(format!(
                    "unexpected suffix after literal: {rest:?}"
                )));
            };
            return Ok(RDFTerm {
                value: value.into(),
                kind,
            });
        }

        Err(E::custom(format!("unrecognized TSV cell: {s:?}")))
    }
}

/// Parses a quoted N-Triples-style string starting at `s[0] == '"'`.
/// Returns the unescaped content and the remaining suffix after the closing `"`.
fn parse_quoted_str(s: &str) -> Result<(String, &str), String> {
    let inner = &s[1..]; // skip opening '"'
    let mut value = String::new();
    let mut chars = inner.char_indices();
    loop {
        match chars.next() {
            None => return Err("unterminated string literal".into()),
            Some((i, '"')) => return Ok((value, &inner[i + 1..])),
            Some((_, '\\')) => match chars.next() {
                None => return Err("unterminated escape sequence".into()),
                Some((_, 'n')) => value.push('\n'),
                Some((_, 'r')) => value.push('\r'),
                Some((_, 't')) => value.push('\t'),
                Some((_, '"')) => value.push('"'),
                Some((_, '\'')) => value.push('\''),
                Some((_, '\\')) => value.push('\\'),
                Some((_, 'u')) => {
                    let hex: String = chars.by_ref().take(4).map(|(_, c)| c).collect();
                    if hex.len() != 4 {
                        return Err(format!("incomplete \\u escape: {hex:?}"));
                    }
                    let code = u32::from_str_radix(&hex, 16)
                        .map_err(|_| format!("invalid \\u escape: {hex:?}"))?;
                    let ch = char::from_u32(code)
                        .ok_or_else(|| format!("invalid unicode codepoint U+{code:04X}"))?;
                    value.push(ch);
                }
                Some((_, 'U')) => {
                    let hex: String = chars.by_ref().take(8).map(|(_, c)| c).collect();
                    if hex.len() != 8 {
                        return Err(format!("incomplete \\U escape: {hex:?}"));
                    }
                    let code = u32::from_str_radix(&hex, 16)
                        .map_err(|_| format!("invalid \\U escape: {hex:?}"))?;
                    let ch = char::from_u32(code)
                        .ok_or_else(|| format!("invalid unicode codepoint U+{code:08X}"))?;
                    value.push(ch);
                }
                Some((_, c)) => return Err(format!("unknown escape \\{c}")),
            },
            Some((_, c)) => value.push(c),
        }
    }
}

/// The type of an RDF term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RDFType {
    /// An IRI.
    IRI,
    /// A literal, optionally with a language tag or datatype IRI.
    Literal {
        lang: Option<Box<str>>,
        datatype: Option<Box<str>>,
    },
    /// A blank node.
    BlankNode,
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
        assert_eq!(
            term.kind,
            RDFType::Literal {
                lang: None,
                datatype: None,
            }
        );
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
