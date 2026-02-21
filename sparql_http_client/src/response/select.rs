use std::collections::HashMap;
use std::slice::Iter;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectQueryResponse {
    pub head: SelectHead,
    pub results: Results,
}

impl SelectQueryResponse {
    pub fn vars(&self) -> &[Box<str>] {
        &self.head.vars
    }

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectHead {
    pub vars: Box<[Box<str>]>,
    pub link: Option<Box<[Box<str>]>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Results {
    pub bindings: Box<[HashMap<Box<str>, RDFTerm>]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RDFTerm {
    pub value: Box<str>,
    #[serde(flatten)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum RDFType {
    #[serde(rename = "uri")]
    IRI,
    #[serde(rename = "literal")]
    Literal {
        #[serde(flatten)]
        kind: LiteralType,
    },
    #[serde(rename = "bnode")]
    BlankNode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum LiteralType {
    WithLanguage {
        #[serde(rename = "xml:lang")]
        lang: Box<str>,
    },
    WithDataType {
        datatype: Box<str>,
    },
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
