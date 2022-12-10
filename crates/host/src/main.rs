use anyhow::anyhow;
use clap::Parser;
use cli::Args;
use config::Config;
use ctx::RequestCtx;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use service::Service;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use wasmtime::component::Linker;
use wasmtime::{Engine, Store};

use sdk::http::imports::{HeaderParam, HttpComponent, Method};
use sdk::http::imports::{HeaderResult, Request as WasmRequest, Response as WasmResponse, Version};

mod cli;
mod config;
mod ctx;
mod filesystem;
mod service;

struct WasmState {
    config: Config,
    services: HashMap<String, Box<Service>>,
    engine: Engine,
}

fn init_wasmtime() -> anyhow::Result<Engine> {
    let mut config = wasmtime::Config::new();
    config.wasm_component_model(true);

    let engine = Engine::new(&config)?;
    Ok(engine)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Load configuration file
    let config_path = Path::new(&args.config).canonicalize()?;
    let config = toml::from_str::<Config>(std::fs::read_to_string(&config_path)?.as_str())?;

    // Initialize the Wasmtime runtime
    let engine = init_wasmtime()?;

    // Load all defined services
    let dir = config_path
        .parent()
        .ok_or_else(|| anyhow!("Cannot open base directory"))?;
    std::env::set_current_dir(dir)?;

    let mut services = HashMap::new();
    for entry in std::fs::read_dir(".")? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if !path.join("service.toml").exists() {
            continue;
        }
        let service = Service::load(path.clone(), &engine);

        if let Ok(service) = service {
            services.insert(service.name.clone(), Box::new(service));
        } else {
            eprintln!(
                "Error loading service in {}: {}",
                path.display(),
                service.err().unwrap()
            );
        }
    }

    // Create a `WasmState` instance that will be shared across all threads
    let state = Arc::new(WasmState {
        config,
        services,
        engine,
    });

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
            async move {
                // Route the request to the appropriate service
                let service = state_for_closure
                    .config
                    .route(req.uri().to_string())
                    .and_then(|service| state_for_closure.services.get(&service.name));

                if service.is_none() {
                    return Ok::<_, Infallible>(
                        Response::builder()
                            .status(404)
                            .body(Body::from("Not Found"))
                            .unwrap(),
                    );
                }

                // Construct a `RequestCtx` instance for the WASM execution
                let service = service.unwrap();
                let ctx = service.construct_ctx();

                if ctx.is_err() {
                    return Ok::<_, Infallible>(
                        Response::builder()
                            .status(500)
                            .body(Body::from("Internal Server Error"))
                            .unwrap(),
                    );
                }

                // Destructure the request parts and body
                let (parts, body) = req.into_parts();

                // Create a new `Store` and `Linker` for the WASI module
                // TODO: Set preopened directories from a config file
                let mut store = Store::new(&state_for_closure.engine, ctx.unwrap());

                let mut linker = Linker::new(&state_for_closure.engine);

                // Add the WASI module to the linker
                wasmtime_wasi_host::add_to_linker(&mut linker, |cx: &mut RequestCtx| &mut cx.wasi)
                    .unwrap();

                // Add custom SDK filesystem module
                filesystem::add_to_linker(&mut linker, |cx: &mut RequestCtx| cx).unwrap();

                // Instantiate the HTTP component
                let (component, _instance) =
                    HttpComponent::instantiate(&mut store, &service.component, &linker).unwrap();

                // Return a `Future` that handles the request and produces the response
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
                Ok::<_, Infallible>(
                    res.response_builder()
                        .body(Body::from(String::from_utf8_lossy(&body).to_string()))
                        .unwrap(),
                )
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
