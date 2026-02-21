use std::collections::HashMap;
use std::slice::Iter;

use serde::{Deserialize, Serialize};

/// The deserialized response to a SPARQL SELECT query.
///
/// Use [`rows`](SelectQueryResponse::rows) to iterate over result rows, and
/// [`vars`](SelectQueryResponse::vars) to inspect the projected variables.
/// Implements [`IntoIterator`] so you can iterate rows directly with `for row in &response`.
///
/// # Example
///
/// ```
/// use sparql_http_client::response::{SelectQueryResponse, SelectHead, Results, RDFTerm, RDFType};
/// use std::collections::HashMap;
///
/// let response = SelectQueryResponse {
///     head: SelectHead { vars: vec!["s".into()].into(), link: None },
///     results: Results {
///         bindings: vec![
///             HashMap::from([("s".into(), RDFTerm {
///                 value: "http://example.org/".into(),
///                 kind: RDFType::IRI,
///             })]),
///         ].into(),
///     },
/// };
///
/// assert_eq!(response.vars(), &["s".into()]);
///
/// for row in &response {
///     println!("{}", row["s"].value);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectQueryResponse {
    pub head: SelectHead,
    pub results: Results,
}

impl SelectQueryResponse {
    /// Returns the projected variable names declared in the query's `SELECT` clause.
    ///
    /// For `SELECT ?s ?p WHERE { ... }` this returns `["s", "p"]`.
    pub fn vars(&self) -> &[Box<str>] {
        &self.head.vars
    }

    /// Returns a slice of the result rows.
    ///
    /// Each row is a [`HashMap`] mapping variable names (without the `?`) to
    /// their [`RDFTerm`] value. Variables that are unbound in a given row are
    /// absent from the map.
    pub fn rows(&self) -> &[HashMap<Box<str>, RDFTerm>] {
        &self.results.bindings
    }
}

impl<'a> IntoIterator for &'a SelectQueryResponse {
    type Item = &'a HashMap<Box<str>, RDFTerm>;
    type IntoIter = Iter<'a, HashMap<Box<str>, RDFTerm>>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows().iter()
    }
}

/// The `head` section of a SPARQL SELECT response.
///
/// Contains the list of projected variable names. Prefer
/// [`SelectQueryResponse::vars`] over accessing this directly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectHead {
    pub vars: Box<[Box<str>]>,
    pub link: Option<Box<[Box<str>]>>,
}

/// The `results` section of a SPARQL SELECT response.
///
/// Contains the binding rows. Prefer [`SelectQueryResponse::rows`] or
/// iterating with `for row in &response` over accessing this directly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Results {
    pub bindings: Box<[HashMap<Box<str>, RDFTerm>]>,
}

/// A single RDF term: the value bound to a variable in one result row.
///
/// The [`value`](RDFTerm::value) field always holds the string representation
/// regardless of kind — useful when you just need the value and don't care
/// about the RDF type. Use [`kind`](RDFTerm::kind) or the convenience methods
/// when the type matters.
///
/// # Example
///
/// ```
/// use sparql_http_client::response::{RDFTerm, RDFType, LiteralType};
///
/// let iri = RDFTerm { value: "http://example.org/".into(), kind: RDFType::IRI };
/// assert!(iri.is_iri());
/// assert_eq!(&*iri.value, "http://example.org/");
///
/// let lit = RDFTerm {
///     value: "hello".into(),
///     kind: RDFType::Literal { kind: LiteralType::WithLanguage { lang: "en".into() } },
/// };
/// assert_eq!(lit.lang(), Some("en"));
/// assert_eq!(lit.datatype(), None);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RDFTerm {
    pub value: Box<str>,
    #[serde(flatten)]
    pub kind: RDFType,
}

impl RDFTerm {
    /// Returns `true` if this term is an IRI.
    pub fn is_iri(&self) -> bool {
        matches!(self.kind, RDFType::IRI)
    }

    /// Returns `true` if this term is a literal.
    pub fn is_literal(&self) -> bool {
        matches!(self.kind, RDFType::Literal { .. })
    }

    /// Returns `true` if this term is a blank node.
    pub fn is_blank_node(&self) -> bool {
        matches!(self.kind, RDFType::BlankNode)
    }

    /// Returns the language tag if this is a language-tagged literal, otherwise `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use sparql_http_client::response::{RDFTerm, RDFType, LiteralType};
    ///
    /// let term = RDFTerm {
    ///     value: "hello".into(),
    ///     kind: RDFType::Literal { kind: LiteralType::WithLanguage { lang: "en".into() } },
    /// };
    /// assert_eq!(term.lang(), Some("en"));
    /// ```
    pub fn lang(&self) -> Option<&str> {
        if let RDFType::Literal {
            kind: LiteralType::WithLanguage { lang },
        } = &self.kind
        {
            Some(lang)
        } else {
            None
        }
    }

    /// Returns the datatype IRI if this is a datatyped literal, otherwise `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use sparql_http_client::response::{RDFTerm, RDFType, LiteralType};
    ///
    /// let term = RDFTerm {
    ///     value: "42".into(),
    ///     kind: RDFType::Literal {
    ///         kind: LiteralType::WithDataType {
    ///             datatype: "http://www.w3.org/2001/XMLSchema#integer".into(),
    ///         },
    ///     },
    /// };
    /// assert_eq!(term.datatype(), Some("http://www.w3.org/2001/XMLSchema#integer"));
    /// ```
    pub fn datatype(&self) -> Option<&str> {
        if let RDFType::Literal {
            kind: LiteralType::WithDataType { datatype },
        } = &self.kind
        {
            Some(datatype)
        } else {
            None
        }
    }
}

/// The type of an RDF term in a SPARQL SELECT response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum RDFType {
    /// An IRI (Internationalized Resource Identifier).
    #[serde(rename = "uri")]
    IRI,
    /// A literal value, optionally with a language tag or datatype.
    #[serde(rename = "literal")]
    Literal {
        #[serde(flatten)]
        kind: LiteralType,
    },
    /// A blank node.
    #[serde(rename = "bnode")]
    BlankNode,
}

/// The subtype of an RDF literal term.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum LiteralType {
    /// A language-tagged literal, e.g. `"hello"@en`.
    WithLanguage {
        #[serde(rename = "xml:lang")]
        lang: Box<str>,
    },
    /// A datatyped literal, e.g. `"42"^^xsd:integer`.
    WithDataType { datatype: Box<str> },
    /// A plain literal with no language tag or datatype.
    Simple {},
}

#[cfg(test)]
mod tests {
    use super::*;

    fn struct_format() -> SelectQueryResponse {
        SelectQueryResponse {
            head: SelectHead {
                vars: vec!["obj".into()].into(),
                link: None,
            },
            results: Results {
                bindings: vec![
                    HashMap::from([(
                        "obj".into(),
                        RDFTerm {
                            value: "http://creativecommons.org/publicdomain/zero/1.0/".into(),
                            kind: RDFType::IRI,
                        },
                    )]),
                    HashMap::from([(
                        "obj".into(),
                        RDFTerm {
                            value: "1.0.0".into(),
                            kind: RDFType::Literal {
                                kind: LiteralType::Simple {},
                            },
                        },
                    )]),
                    HashMap::from([(
                        "obj".into(),
                        RDFTerm {
                            value: "2023-01-30T23:00:08Z".into(),
                            kind: RDFType::Literal {
                                kind: LiteralType::WithDataType {
                                    datatype: "http://www.w3.org/2001/XMLSchema#dateTime".into(),
                                },
                            },
                        },
                    )]),
                ]
                .into(),
            },
        }
    }

    fn text_format() -> &'static str {
        r#"
        {
            "head": {
                "vars": [
                    "obj"
                ]
            },
            "results": {
                "bindings": [
                    {
                        "obj": {
                            "type": "uri",
                            "value": "http://creativecommons.org/publicdomain/zero/1.0/"
                        }
                    },
                    {
                        "obj": {
                            "type": "literal",
                            "value": "1.0.0"
                        }
                    },
                    {
                        "obj": {
                            "datatype": "http://www.w3.org/2001/XMLSchema#dateTime",
                            "type": "literal",
                            "value": "2023-01-30T23:00:08Z"
                        }
                    }
                ]
            }
        }
        "#
    }

    #[test]
    fn serialize() -> anyhow::Result<()> {
        let _ = serde_json::to_string(&struct_format())?;

        Ok(())
    }

    #[test]
    fn deserialize() -> anyhow::Result<()> {
        let into_struct: SelectQueryResponse = serde_json::from_str(text_format())?;

        assert_eq!(into_struct, struct_format());

        Ok(())
    }
}
