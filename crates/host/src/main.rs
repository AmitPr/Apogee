use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use wasmtime_wasi_host::WasiCtx;
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


#[derive(Default)]
pub struct RequestCtx {
    wasi: WasiCtx,
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
    // Initialize the Wasmtime runtime
    let state = Arc::new(init_wasmtime());

    // Create a `make_service_fn` closure that returns a `Service` instance
    // for each incoming connection
    let make_svc = make_service_fn(|_conn| {
        // Create a clone of the `state` variable so we can use it inside the closure
        let state = state.clone();

        // Return a `service_fn` closure that takes a `Request<Body>` and
        // returns a `Future` representing the response to the request
        let svc = service_fn(move |req: Request<Body>| {
            // Create a clone of the `state_for_closure` variable so we can
            // use it inside the closure
            let state_for_closure = state.clone();

            // Destructure the request parts and body
            let (parts, body) = req.into_parts();

            // Create a new `Store` and `Linker` for the WASI module
            let mut store = Store::new(&state_for_closure.engine, RequestCtx::default());
            let mut linker = Linker::new(&state_for_closure.engine);

            // Add the WASI module to the linker
            wasmtime_wasi_host::add_to_linker(&mut linker, |cx: &mut RequestCtx| &mut cx.wasi).unwrap();

            // Instantiate the HTTP component
            let (component, _instance) =
                HttpComponent::instantiate(&mut store, &state_for_closure.component, &linker)
                    .unwrap();

            // Return a `Future` that handles the request and produces the response
            async move {
                // Convert the `Method` and `Version` from their raw
                // representation to their corresponding structs
                let method = Method::try_from(parts.method.clone()).unwrap();
                let version = Version::try_from(parts.version).unwrap();

                // Get the request URI as a string
                let uri: &str = parts.uri.path();

                // Convert the request headers to a vector of `HeaderParam`
                let headers: Vec<HeaderParam> = parts
                    .headers
                    .iter()
                    .map(|(key, value)| HeaderParam {
                        key: key.as_str().as_bytes(),
                        value: value.as_bytes(),
                    })
                    .collect();

                // Grab the request body
                let body = hyper::body::to_bytes(body).await.unwrap().to_vec();

                // Create a `WasmRequest` from the request parts
                let req = WasmRequest {
                    version,
                    method,
                    uri,
                    headers: headers.as_slice(),
                    body: body.as_slice(),
                };

                // Call the `handle_http_request` method on the `Http`
                // instance, and handle any errors that occur
                let res = component
                    .handle_http_request(&mut store, req)
                    .unwrap_or_else(|e| Err(format!("Error calling wasm handler: {e}")));

                // Match the result of calling handle_http_request on the Http instance, handling any errors that occur
                let res = match res {
                    Ok(res) => res,
                    Err(e) => WasmResponse {
                        status: 500,
                        headers: vec![HeaderResult {
                            key: b"Content-Type".to_vec(),
                            value: b"text/plain".to_vec(),
                        }],
                        version: Version::HttpV11,
                        body: e.as_bytes().to_vec(),
                    },
                };

                // Convert the WasmResponse to a Response and handle any errors that occur
                let body = res.body.clone();
                let res: Response<String> = res
                    .response_builder()
                    .body(String::from_utf8_lossy(&body).to_string())
                    .unwrap();
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
