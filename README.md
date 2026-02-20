# sparql_http_client

Simple SPARQL client for Rust.

## Compile-time query validation

The `query!` macro parses and validates the SPARQL query string **at compile time**:

- Syntax errors are caught as compile errors, not runtime panics.
- The query kind (`SELECT`, `ASK`, …) is resolved at compile time, so the return type is
  already [`SelectQuery`] or [`AskQuery`] — no runtime type dispatch, no `Result` to unwrap.

```rust
// This is a compile error — caught before the binary is ever built:
let q = query!(endpoint, "SELCT ?s WHERE { ?s ?p ?o }");
//                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// error: SPARQL syntax error: …
```

## Example

```rust
use sparql_http_client::{query, Endpoint, SparqlClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let endpoint = Endpoint::new(
        SparqlClient::default(),
        "https://query.wikidata.org/bigdata/namespace/wdq/sparql",
    );

    let query = query!(endpoint, r#"
        PREFIX wdt: <http://www.wikidata.org/prop/direct/>
        PREFIX wd: <http://www.wikidata.org/entity/>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?countryLabel ?population ?capitalLabel WHERE {
            ?country    wdt:P31 wd:Q3624078 ;
                        wdt:P36 ?capital ;
                        wdt:P1082 ?population ;
                        rdfs:label ?countryLabel .

            ?capital rdfs:label ?capitalLabel .

            FILTER(LANG(?countryLabel) = "en") .
            FILTER(LANG(?capitalLabel) = "en") .
        }
        ORDER BY DESC(?population)
        LIMIT 3
    "#);

    for bindings in query.run().await?.results.bindings {
        println!("{:?}", bindings);
    }

    Ok(())
}
```
