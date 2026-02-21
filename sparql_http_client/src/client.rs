use reqwest::{
    header::{HeaderValue, ACCEPT, USER_AGENT},
    RequestBuilder,
};

use crate::query::{QueryString, SparqlQuery};

/// Identifies this client to the SPARQL endpoint via the HTTP `User-Agent` header.
///
/// The header value is formatted as:
/// `{name}/{version} ({contact}) sparql_http_client/{crate_version}`
///
/// Many public SPARQL endpoints ask callers to provide a meaningful user agent
/// so administrators can identify and contact heavy users.
///
/// # Example
///
/// ```
/// use sparql_http_client::UserAgent;
///
/// let agent = UserAgent {
///     name: "my-app".into(),
///     version: "1.0.0".into(),
///     contact: "mailto:user@example.com".into(),
/// };
/// ```
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

/// An HTTP client for issuing SPARQL requests.
///
/// Wraps [`reqwest::Client`] together with an optional [`UserAgent`].
/// Use [`SparqlClient::default`] for anonymous access or [`SparqlClient::new`]
/// to attach a descriptive user agent to every request.
///
/// `SparqlClient` is cheap to clone — the underlying [`reqwest::Client`] shares
/// its connection pool via reference counting.
///
/// # Example
///
/// ```
/// use sparql_http_client::{SparqlClient, UserAgent};
///
/// // Anonymous client
/// let client = SparqlClient::default();
///
/// // Client with a custom user agent
/// let client = SparqlClient::new(UserAgent {
///     name: "my-app".into(),
///     version: "1.0.0".into(),
///     contact: "mailto:user@example.com".into(),
/// });
/// ```
#[derive(Debug, Default, Clone)]
pub struct SparqlClient {
    inner: reqwest::Client,
    agent: UserAgent,
}

impl SparqlClient {
    /// Creates a new client with the given [`UserAgent`].
    pub fn new(user_agent: UserAgent) -> Self {
        Self {
            inner: reqwest::Client::new(),
            agent: user_agent,
        }
    }
}

/// A SPARQL endpoint that executes queries over HTTP.
///
/// Combines an HTTP client with a URL. Create executable queries with
/// [`build_query`](Endpoint::build_query) or the [`query!`](crate::query) macro.
///
/// `Endpoint` is cheap to clone — the underlying [`reqwest::Client`] shares
/// its connection pool via reference counting.
///
/// # Example
///
/// ```
/// use sparql_http_client::{Endpoint, SparqlClient};
///
/// let endpoint = Endpoint::new(
///     SparqlClient::default(),
///     "https://query.wikidata.org/bigdata/namespace/wdq/sparql",
/// );
/// ```
#[derive(Debug, Clone)]
pub struct Endpoint {
    url: Box<str>,
    client: SparqlClient,
}

impl Endpoint {
    /// Creates a new endpoint from a client and a URL.
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

    /// Wraps `query` in a [`SparqlQuery`] ready to be executed against this endpoint.
    ///
    /// This method consumes the endpoint. To reuse the endpoint for multiple
    /// queries, clone it first:
    ///
    /// ```
    /// use sparql_http_client::{Endpoint, SparqlClient, SelectQueryString};
    ///
    /// let endpoint = Endpoint::new(SparqlClient::default(), "https://example.org/sparql");
    /// let qs: SelectQueryString = "SELECT ?s WHERE { ?s ?p ?o }".parse().unwrap();
    ///
    /// let _query = endpoint.clone().build_query(qs);
    /// // endpoint is still usable here
    /// ```
    ///
    /// Prefer the [`query!`](crate::query) macro for compile-time validation.
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
