use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// The entrypoint macro generates boilerplate that is required to interface with
/// the Wasmtime host.
///
/// ```ignore
/// #[entrypoint(http)]
/// fn handle_http_request(req: sdk::http::Request) -> sdk::http::Response {
///
/// }
#[proc_macro_attribute]
pub fn entrypoint(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let attr = parse_macro_input!(attr as syn::Ident);
    match attr.to_string().as_str() {
        "http" => generate_http_entrypoint(input),
        _ => panic!("Unknown entrypoint"),
    }
}

pub(crate) fn generate_http_entrypoint(func: ItemFn) -> TokenStream {
    let name = &func.sig.ident;
    quote!(
        mod http_service_module {
            mod http_component {
                pub use ::apogee_sdk::http::{call_handle_http_request, post_return_handle_http_request};
            }
            struct HttpHandler;
            impl ::apogee_sdk::http::HttpComponent for HttpHandler {
                fn handle_http_request(req: ::apogee_sdk::http::Request) -> Result<::apogee_sdk::http::Response, String> {
                    super::#name(req)
                }
            }
            #[cfg(target_arch = "wasm32")]
            use ::apogee_sdk::http::__link_section;
            #[cfg(target_arch = "wasm32")]
            ::apogee_sdk::export_http_component!(HttpHandler);
        }
        #func
    )
    .into()
}
