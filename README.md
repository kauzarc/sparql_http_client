# sparql_http_client

Simple sparql client for rust.

Example :

```Rust
use anyhow;
use sparql_http_client::SparqlClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = SparqlClient::default();
    let endpoint = client.endpoint("https://query.wikidata.org/bigdata/namespace/wdq/sparql");

    let query_response = endpoint
        .select(
            r#"
            PREFIX wdt: <http://www.wikidata.org/prop/direct/>
            PREFIX wd: <http://www.wikidata.org/entity/>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

            SELECT ?countryLabel ?population ?capitalLabel WHERE {
                ?country	wdt:P31 wd:Q3624078 ;
                            wdt:P36 ?capital ;
                            wdt:P1082 ?population ;
                            rdfs:label ?countryLabel .
                
                ?capital rdfs:label ?capitalLabel .
                
                FILTER(LANG(?countryLabel) = "en") .
                FILTER(LANG(?capitalLabel) = "en") .
            }
            ORDER BY DESC(?population)
            LIMIT 3
            "#,
        )?
        .run()
        .await?;

    for bindings in query_response.results.bindings {
        println!("{:?}", bindings);
    }

    Ok(())
}
```