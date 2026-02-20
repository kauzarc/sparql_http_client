# sparql_http_client

Simple sparql client for rust.

Example :

```Rust
use sparql_http_client::{Endpoint, SelectQuery, SparqlClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let endpoint = Endpoint::new(
        SparqlClient::default(),
        "https://query.wikidata.org/bigdata/namespace/wdq/sparql",
    );

    let query: SelectQuery = endpoint.build_query(r#"
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
    "#.parse()?);

    for bindings in query.run().await?.results.bindings {
        println!("{:?}", bindings);
    }

    Ok(())
}
```