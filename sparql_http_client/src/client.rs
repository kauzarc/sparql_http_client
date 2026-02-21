use reqwest::{
    header::{HeaderValue, ACCEPT, USER_AGENT},
    RequestBuilder,
};

use crate::query::{QueryString, SparqlQuery};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct UserAgent {
    pub name: Box<str>,
    pub version: Box<str>,
    pub contact: Box<str>,
}

impl UserAgent {
    fn header_value(&self) -> HeaderValue {
        HeaderValue::from_str(&format!(
            "{}/{} ({}) {}/{}",
            self.name,
            self.version,
            self.contact,
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("valid characters")
    }
}

#[derive(Debug, Default, Clone)]
pub struct SparqlClient {
    inner: reqwest::Client,
    agent: UserAgent,
}

impl SparqlClient {
    pub fn new(user_agent: UserAgent) -> Self {
        Self {
            inner: reqwest::Client::new(),
            agent: user_agent,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Endpoint {
    url: Box<str>,
    client: SparqlClient,
}

impl Endpoint {
    pub fn new(client: SparqlClient, url: &str) -> Self {
        Self {
            url: url.into(),
            client,
        }
    }

    pub(crate) fn request(&self) -> RequestBuilder {
        self.client
            .inner
            .post(&*self.url)
            .header(
                ACCEPT,
                HeaderValue::from_static("application/sparql-results+json"),
            )
            .header(USER_AGENT, self.client.agent.header_value())
    }

    pub fn build_query<Q>(self, query: Q) -> SparqlQuery<Q>
    where
        Q: QueryString,
    {
        query.build(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_client() {
        SparqlClient::default();
    }
}
