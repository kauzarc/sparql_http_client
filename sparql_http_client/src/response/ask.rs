use std::string;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AskQueryResponse {
    pub head: AskHead,
    pub boolean: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AskHead {
    pub link: Option<Vec<string::String>>,
}

#[cfg(test)]
mod test {
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
