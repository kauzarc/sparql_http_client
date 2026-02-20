# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build                        # build all crates
cargo test                         # run all tests
cargo test -p sparql_http_client   # run tests for the main crate only
cargo test <test_name>             # run a single test by name
cargo clippy -- -D warnings        # lint
```

Tests in `sparql_http_client` hit live HTTP endpoints (`httpbin.org`, `wikidata.org`) and require network access.

## Architecture

This is a Cargo workspace with two crates:

- **`sparql_http_client`** — the public library
- **`sparql_http_client_macros`** — proc-macro crate (must stay a separate crate per Rust rules)

### Query pipeline

The central abstraction is the `QueryString` trait (`query.rs`). Each SPARQL query kind is a distinct newtype implementing it:

| Type | Response type |
|---|---|
| `SelectQueryString` | `SelectQueryResponse` |
| `AskQueryString` | `AskQueryResponse` |

`QueryString::build(&endpoint)` wraps the string in a `SparqlQuery<'_, Q>`, which borrows the `Endpoint` for its lifetime. Calling `.run().await` sends a form-encoded POST and deserializes the JSON response into `Q::Response`.

### Compile-time vs runtime validation

- **Runtime path**: `"...".parse::<SelectQueryString>()` — calls spargebra, returns `Result<_, QueryStringError>`
- **Compile-time path**: `query!(endpoint, "...")` — same spargebra parse runs inside the proc macro; emits a call to `new_unchecked` (skipping the runtime parse) with the normalised query string. The type (`SelectQuery` / `AskQuery`) is inferred from the parsed query kind.

`new_unchecked` is `#[doc(hidden)]`; it exists solely for the macro's generated code.

### Adding a new query kind

1. Add a `QueryString` impl (see `query/select.rs` as a template)
2. Add a `QueryResponse` impl for the response type
3. Handle the new `spargebra::Query` variant in `query_string_type` in `sparql_http_client_macros/src/lib.rs`
