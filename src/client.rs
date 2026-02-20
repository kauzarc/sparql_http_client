use crate::{error, query};
use reqwest::{header, RequestBuilder};
use std::string;

#[derive(Debug, Default, Clone)]
pub struct UserAgent {
    pub name: string::String,
    pub version: string::String,
    pub contact: string::String,
}

impl UserAgent {
    fn header_value(&self) -> header::HeaderValue {
        header::HeaderValue::from_str(&format!(
            "{}/{} ({}) {}/{}",
            self.name,
            self.version,
            self.contact,
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("valid charcters")
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

#[derive(Clone)]
pub struct Endpoint {
    url: string::String,
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
            .post(&self.url)
            .header(
                header::ACCEPT,
                header::HeaderValue::from_static("application/sparql-results+json"),
            )
            .header(header::USER_AGENT, self.client.agent.header_value())
    }

    pub fn select<Q>(&self, query: Q) -> Result<query::select::SelectQuery<'_>, error::QueryError>
    where
        Q: TryInto<spargebra::Query, Error = spargebra::ParseError>,
    {
        query::select::SelectQuery::new(self, query)
    }

    pub fn ask<Q>(&self, query: Q) -> Result<query::ask::AskQuery<'_>, error::QueryError>
    where
        Q: TryInto<spargebra::Query, Error = spargebra::ParseError>,
    {
        query::ask::AskQuery::new(self, query)
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
