# sparql_http_client

An async, typed SPARQL HTTP client for Rust with optional compile-time query validation.

## Quick start

```rust,no_run
use sparql_http_client::{Endpoint, SparqlClient, query};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = Endpoint::new(
        SparqlClient::default(),
        "https://query.wikidata.org/bigdata/namespace/wdq/sparql",
    );

    let response = query!(endpoint, r#"
        PREFIX wdt: <http://www.wikidata.org/prop/direct/>
        PREFIX wd:  <http://www.wikidata.org/entity/>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?countryLabel ?population WHERE {
            ?country wdt:P31  wd:Q3624078 ;
                     wdt:P1082 ?population ;
                     rdfs:label ?countryLabel .
            FILTER(LANG(?countryLabel) = "en")
        }
        ORDER BY DESC(?population)
        LIMIT 5
    "#)
    .run()
    .await?;

    for row in &response {
        let label = row.get("countryLabel").map(|t| t.value.as_ref()).unwrap_or("?");
        let pop   = row.get("population").map(|t| t.value.as_ref()).unwrap_or("?");
        println!("{label}: {pop}");
    }

    Ok(())
}
```

## Compile-time query validation

The `query!` macro validates SPARQL syntax at compile time:

- Syntax errors are caught as compile errors, not runtime panics
- The query kind (`SELECT`, `ASK`, …) is resolved at compile time, so the
  return type is already `SelectQuery` or `AskQuery` — no runtime dispatch,
  no `Result` to unwrap

```rust,ignore
// This is a compile error — caught before the binary is ever run:
let q = query!(endpoint, "SELCT ?s WHERE { ?s ?p ?o }");
//                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// error: SPARQL syntax error: …
```

For runtime validation, parse a string using `str::parse`:

```rust
use sparql_http_client::SelectQueryString;

let qs: Result<SelectQueryString, _> = "SELECT ?s WHERE { ?s ?p ?o }".parse();
assert!(qs.is_ok());

let qs: Result<SelectQueryString, _> = "not sparql".parse();
assert!(qs.is_err());
```

## Query types

| Query kind | String type | Response type |
|---|---|---|
| `SELECT` | `SelectQueryString` | `SelectQueryResponse` |
| `ASK` | `AskQueryString` | `AskQueryResponse` |

## ASK queries

```rust,no_run
use sparql_http_client::{Endpoint, SparqlClient, query};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = Endpoint::new(
        SparqlClient::default(),
        "https://query.wikidata.org/bigdata/namespace/wdq/sparql",
    );

    let is_country: bool = query!(
        endpoint,
        "ASK { <http://www.wikidata.org/entity/Q142> \
               <http://www.wikidata.org/prop/direct/P31> \
               <http://www.wikidata.org/entity/Q3624078> }"
    )
    .run()
    .await?
    .into();

    println!("Is a sovereign state: {is_country}");
    Ok(())
}
```

## Setting a User-Agent

Many public SPARQL endpoints ask callers to provide a meaningful `User-Agent`
so administrators can identify and contact heavy users:

```rust,no_run
use sparql_http_client::{Endpoint, SparqlClient, UserAgent};

let endpoint = Endpoint::new(
    SparqlClient::new(UserAgent {
        name: "my-app".into(),
        version: "1.0.0".into(),
        contact: "mailto:user@example.com".into(),
    }),
    "https://query.wikidata.org/bigdata/namespace/wdq/sparql",
);
```

## Accessing response data

`term.value` always holds the string representation of an RDF term regardless
of its type, so reading values requires no type matching:

```rust,ignore
for row in &response {
    if let Some(term) = row.get("label") {
        println!("{}", term.value);
    }
}
```

When the RDF type matters, use the convenience methods on `RDFTerm`:

```rust,ignore
for row in &response {
    if let Some(term) = row.get("obj") {
        if term.is_iri() {
            println!("IRI: {}", term.value);
        } else if let Some(lang) = term.lang() {
            println!("Literal (lang={lang}): {}", term.value);
        } else if let Some(dt) = term.datatype() {
            println!("Literal (type={dt}): {}", term.value);
        } else {
            println!("Literal: {}", term.value);
        }
    }
}
```

Or match on `kind` for exhaustive handling:

```rust,ignore
use sparql_http_client::response::{RDFType, LiteralType};

for row in &response {
    if let Some(term) = row.get("obj") {
        match &term.kind {
            RDFType::IRI => println!("IRI: {}", term.value),
            RDFType::BlankNode => println!("blank node"),
            RDFType::Literal { kind: LiteralType::WithLanguage { lang } } => {
                println!("\"{}\"@{lang}", term.value);
            }
            RDFType::Literal { kind: LiteralType::WithDataType { datatype } } => {
                println!("\"{}\"^^{datatype}", term.value);
            }
            RDFType::Literal { kind: LiteralType::Simple {} } => {
                println!("\"{}\"", term.value);
            }
        }
    }
}
```
