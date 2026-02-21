use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AskQueryResponse {
    pub head: AskHead,
    pub boolean: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AskHead {
    pub link: Option<Box<[Box<str>]>>,
}

impl From<AskQueryResponse> for bool {
    fn from(r: AskQueryResponse) -> Self {
        r.boolean
    }
}

impl From<&AskQueryResponse> for bool {
    fn from(r: &AskQueryResponse) -> Self {
        r.boolean
    }
}

#[cfg(test)]
mod tests {
    use super::AskQueryResponse;

    #[test]
    fn deserialize() -> anyhow::Result<()> {
        let _: AskQueryResponse = serde_json::from_str(
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
