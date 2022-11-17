mod wasi;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use wasmtime::component::{Component, Linker};
use wasmtime::{Engine, Store};

use sdk::http::imports::{HeaderParam, HttpComponent, Method};
use sdk::http::imports::{HeaderResult, Request as WasmRequest, Response as WasmResponse, Version};

struct WasmState {
    component: Component,
    engine: Engine,
}

fn init_wasmtime() -> WasmState {
    let wasm_path = std::env::current_dir()
        .unwrap()
        .join("../../wasm/guest_component.wasm");

    let mut config = wasmtime::Config::new();
    config.wasm_component_model(true);

    let engine = Engine::new(&config).unwrap();
    let component = Component::from_file(&engine, wasm_path).unwrap();
    WasmState { component, engine }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(init_wasmtime());

    let make_svc = make_service_fn(|_conn| {
        let state_for_closure = state.clone();
        let svc = service_fn(move |req: Request<Body>| {
            let state_for_closure = state_for_closure.clone();
            let (parts, body) = req.into_parts();

            let mut store = Store::new(&state_for_closure.engine, wasi::RequestCtx { body });
            let mut linker = Linker::new(&state_for_closure.engine);
            
            wasi::testwasi::add_to_linker(&mut linker, |cx| cx).unwrap();

            let (component, _instance) =
                HttpComponent::instantiate(&mut store, &state_for_closure.component, &linker)
                    .unwrap();
            async move {
                let method = Method::try_from(parts.method.clone()).unwrap();
                let version = Version::try_from(parts.version).unwrap();
                let uri: &str = parts.uri.path();
                let headers: Vec<HeaderParam> = parts
                    .headers
                    .iter()
                    .map(|(key, value)| {
                        let header: HeaderParam = HeaderParam {
                            key: key.as_str().as_bytes(),
                            value: value.as_bytes(),
                        };
                        header
                    })
                    .collect();
                let headers: &[HeaderParam] = headers.as_slice();

                let req = WasmRequest {
                    version,
                    method,
                    uri,
                    headers,
                };

                let res = component
                    .handle_http_request(&mut store, req)
                    .unwrap_or_else(|e| Err(format!("Error calling wasm handler: {e}")));

                let res = match res {
                    Ok(res) => res,
                    Err(e) => {
                        println!("Error: {e}");
                        WasmResponse {
                            status: 500,
                            headers: vec![HeaderResult {
                                key: b"Content-Type".to_vec(),
                                value: b"text/plain".to_vec(),
                            }],
                            version: Version::HttpV11,
                        }
                    }
                };
                let res: Response<String> = res.try_into().unwrap();
                Ok::<_, Infallible>(res)
            }
        });
        async move { Ok::<_, Infallible>(svc) }
    });

    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {e}");
    }

    Ok(())
}
