use std::collections::HashMap;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SelectQueryResponse {
    pub head: SelectHead,
    pub results: Results,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SelectHead {
    pub vars: Vec<String>,
    pub link: Option<Vec<String>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Results {
    pub bindings: Vec<HashMap<String, RDFTerm>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct RDFTerm {
    pub value: String,
    #[serde(flatten)]
    pub kind: RDFType,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
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

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum LiteralType {
    WithLanguage {
        #[serde(rename = "xml:lang")]
        lang: String,
    },
    WithDataType {
        datatype: String,
    },
    Simple {},
}

#[cfg(test)]
mod tests {
    use super::*;

    fn struct_format() -> SelectQueryResponse {
        SelectQueryResponse {
            head: SelectHead {
                vars: vec!["obj".into()],
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
                ],
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
