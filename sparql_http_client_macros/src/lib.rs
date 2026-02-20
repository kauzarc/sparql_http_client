use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use spargebra::{Query, SparqlParser};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, LitStr, Token,
};

struct MacroInput {
    endpoint: Expr,
    query_str: LitStr,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let endpoint = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;
        let query_str = input.parse::<LitStr>()?;
        let _ = input.parse::<Token![,]>();
        Ok(Self { endpoint, query_str })
    }
}

fn parse_sparql(query_str: &LitStr) -> syn::Result<Query> {
    SparqlParser::new()
        .parse_query(&query_str.value())
        .map_err(|e| syn::Error::new_spanned(query_str, format!("SPARQL syntax error: {e}")))
}

fn query_string_type(parsed: &Query, query_str: &LitStr) -> syn::Result<TokenStream2> {
    match parsed {
        Query::Select { .. } => Ok(quote! { ::sparql_http_client::SelectQueryString }),
        Query::Ask { .. } => Ok(quote! { ::sparql_http_client::AskQueryString }),
        _ => Err(syn::Error::new_spanned(
            query_str,
            "only SELECT and ASK queries are currently supported",
        )),
    }
}

fn build_query_expr(endpoint: &Expr, qs_type: TokenStream2, normalized: &str) -> TokenStream2 {
    quote! {
        (#endpoint).build_query(
            <#qs_type as ::sparql_http_client::QueryString>::new_unchecked(#normalized)
        )
    }
}

/// Creates a [`sparql_http_client::SparqlQuery`] with compile-time SPARQL syntax validation.
///
/// The query kind (`SELECT`, `ASK`, …) is resolved at compile time, so the returned value is
/// already typed as [`sparql_http_client::SelectQuery`] or [`sparql_http_client::AskQuery`],
/// and `.run().await` yields the matching response type with no runtime parsing overhead.
///
/// A malformed or unsupported query kind is a **compile error**.
///
/// # Example
///
/// ```rust,ignore
/// let query = query!(endpoint, "SELECT ?s WHERE { ?s ?p ?o }");
/// let response = query.run().await?;
/// ```
#[proc_macro]
pub fn query(input: TokenStream) -> TokenStream {
    let MacroInput { endpoint, query_str } = parse_macro_input!(input as MacroInput);

    let parsed = match parse_sparql(&query_str) {
        Ok(q) => q,
        Err(e) => return e.to_compile_error().into(),
    };

    let qs_type = match query_string_type(&parsed, &query_str) {
        Ok(t) => t,
        Err(e) => return e.to_compile_error().into(),
    };

    build_query_expr(&endpoint, qs_type, &parsed.to_string()).into()
}
