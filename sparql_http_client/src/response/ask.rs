use serde::{Deserialize, Serialize};

/// The deserialized response to a SPARQL ASK query.
///
/// Convert to `bool` via [`From`] to extract the result:
///
/// ```
/// use sparql_http_client::response::{AskQueryResponse, AskHead};
///
/// let response = AskQueryResponse { head: AskHead { link: None }, boolean: true };
/// assert!(bool::from(response));
/// ```
///
/// The raw result is also directly accessible as `response.boolean`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AskQueryResponse {
    pub head: AskHead,
    pub boolean: bool,
}

/// The `head` section of a SPARQL ASK response.
///
/// Typically empty; the `link` field is rarely populated by endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AskHead {
    pub link: Option<Box<[Box<str>]>>,
}

/// Extracts the boolean result, consuming the response.
impl From<AskQueryResponse> for bool {
    fn from(r: AskQueryResponse) -> Self {
        r.boolean
    }
}

/// Extracts the boolean result from a reference to the response.
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
