use std::{collections, string};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum QueryResponse {
    Select { head: SelectHead, results: Results },
    Ask { head: AskHead, boolean: bool },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SelectHead {
    pub vars: Vec<string::String>,
    pub link: Option<Vec<string::String>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AskHead {
    pub link: Option<Vec<string::String>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Results {
    pub bindings: Vec<collections::HashMap<string::String, RDFTerm>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum RDFTerm {
    #[serde(rename = "uri")]
    IRI { value: string::String },
    #[serde(rename = "literal")]
    Literal { value: string::String },
    #[serde(rename = "literal")]
    LiteralWithLanguage {
        value: string::String,
        #[serde(rename = "xml:lang")]
        lang: string::String,
    },
    #[serde(rename = "literal")]
    LiteralWithDataType {
        value: string::String,
        datatype: string::String,
    },
    #[serde(rename = "bnode")]
    BlankNode { value: string::String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() -> anyhow::Result<()> {
        let _ = serde_json::to_string(&QueryResponse::Select {
            head: SelectHead {
                vars: vec!["obj".into()],
                link: None,
            },
            results: Results {
                bindings: vec![
                    collections::HashMap::from([(
                        "obj".into(),
                        RDFTerm::IRI {
                            value: "http://creativecommons.org/publicdomain/zero/1.0/".into(),
                        },
                    )]),
                    collections::HashMap::from([(
                        "obj".into(),
                        RDFTerm::Literal {
                            value: "1.0.0".into(),
                        },
                    )]),
                    collections::HashMap::from([(
                        "obj".into(),
                        RDFTerm::LiteralWithDataType {
                            value: "2023-01-30T23:00:08Z".into(),
                            datatype: "http://www.w3.org/2001/XMLSchema#dateTime".into(),
                        },
                    )]),
                ],
            },
        })?;

        Ok(())
    }

    #[test]
    fn deserialize() -> anyhow::Result<()> {
        let _: QueryResponse = serde_json::from_str(
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
            "#,
        )?;

        let _: QueryResponse = serde_json::from_str(
            r#"
            { 
                "head" : { } ,
                "boolean" : true
            }
            "#,
        )?;

        Ok(())
    }
}
