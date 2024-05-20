use crate::{error, query};
use reqwest::{header, RequestBuilder};
use std::string;

#[derive(Debug)]
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
            clap::crate_name!(),
            clap::crate_version!()
        ))
        .expect("valid charcters")
    }
}

#[derive(Debug)]
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

    pub fn endpoint<'a>(&'a self, url: &str) -> Endpoint<'a> {
        Endpoint {
            url: url.into(),
            client: &self,
        }
    }
}
pub struct Endpoint<'a> {
    url: string::String,
    client: &'a SparqlClient,
}

impl<'a> Endpoint<'a> {
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

    pub fn select<Q>(&self, query: Q) -> Result<query::select::SelectQuery, error::QueryError>
    where
        Q: TryInto<spargebra::Query, Error = spargebra::ParseError>,
    {
        query::select::SelectQuery::new(self, query)
    }
}
