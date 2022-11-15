use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use wasmtime::component::{Component, Linker};
use wasmtime::{Engine, Store};

wit_bindgen_host_wasmtime_rust::generate!({
    import: "../bindings/wit/http_service.wit",
    name: "http",
});

impl TryFrom<hyper::Method> for http_service::Method {
    type Error = String;
    fn try_from(method: hyper::Method) -> Result<Self, Self::Error> {
        match method {
            hyper::Method::OPTIONS => Ok(http_service::Method::Options),
            hyper::Method::GET => Ok(http_service::Method::Get),
            hyper::Method::POST => Ok(http_service::Method::Post),
            hyper::Method::PUT => Ok(http_service::Method::Put),
            hyper::Method::DELETE => Ok(http_service::Method::Delete),
            hyper::Method::HEAD => Ok(http_service::Method::Head),
            hyper::Method::TRACE => Ok(http_service::Method::Trace),
            hyper::Method::CONNECT => Ok(http_service::Method::Connect),
            hyper::Method::PATCH => Ok(http_service::Method::Patch),
            _ => Err("Unknown method".to_string()),
        }
    }
}

impl TryFrom<hyper::Version> for http_service::Version {
    type Error = String;
    fn try_from(version: hyper::Version) -> Result<Self, Self::Error> {
        match version {
            hyper::Version::HTTP_09 => Ok(http_service::Version::HttpV09),
            hyper::Version::HTTP_10 => Ok(http_service::Version::HttpV10),
            hyper::Version::HTTP_11 => Ok(http_service::Version::HttpV11),
            hyper::Version::HTTP_2 => Ok(http_service::Version::HttpV2),
            hyper::Version::HTTP_3 => Ok(http_service::Version::HttpV3),
            _ => Err("Unknown version".to_string()),
        }
    }
}

impl From<http_service::Version> for hyper::Version {
    fn from(val: http_service::Version) -> Self {
        match val {
            http_service::Version::HttpV09 => hyper::Version::HTTP_09,
            http_service::Version::HttpV10 => hyper::Version::HTTP_10,
            http_service::Version::HttpV11 => hyper::Version::HTTP_11,
            http_service::Version::HttpV2 => hyper::Version::HTTP_2,
            http_service::Version::HttpV3 => hyper::Version::HTTP_3,
        }
    }
}

impl<T> TryFrom<hyper::Request<T>> for http_service::Request {
    type Error = String;
    fn try_from(req: hyper::Request<T>) -> Result<Self, Self::Error> {
        let (parts, _body) = req.into_parts();
        Ok(http_service::Request {
            method: parts.method.try_into()?,
            uri: parts.uri.to_string(),
            headers: parts
                .headers
                .iter()
                .map(|(k, v)| {
                    (
                        k.to_string(),
                        v.to_str().map_err(|e| e.to_string()).unwrap().to_string(),
                    )
                })
                .collect(),
            version: parts.version.try_into()?,
        })
    }
}

impl<T> TryInto<hyper::Response<T>> for http_service::Response
where
    T: Default,
{
    type Error = String;
    fn try_into(self) -> Result<hyper::Response<T>, Self::Error> {
        let mut res = hyper::Response::builder()
            .status(self.status)
            .version(self.version.into());
        for (k, v) in self.headers {
            res = res.header(k, v);
        }
        Ok(res.body(T::default()).unwrap())
    }
}

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

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| {
        let state_for_closure = state.clone();
        let svc = service_fn(move |req: Request<Body>| {
            let state_for_closure = state_for_closure.clone();
            let mut store = Store::new(&state_for_closure.engine, ());
            let linker = Linker::new(&state_for_closure.engine);
            let (_, instance) =
                Http::instantiate(&mut store, &state_for_closure.component, &linker).unwrap();
            async move {
                let res = instance
                    .get_typed_func::<(http_service::Request,), (Result<http_service::Response, String>,), _>(
                        &mut store,
                        "handle-http-request",
                    )
                    .unwrap()
                    .call(&mut store, (req.try_into().unwrap(),))
                    .unwrap()
                    .0;
                let res = match res {
                    Ok(res) => res,
                    Err(e) => {
                        println!("Error: {}", e);
                        http_service::Response {
                            status: 500,
                            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
                            version: http_service::Version::HttpV11,
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
        eprintln!("server error: {}", e);
    }

    Ok(())
}

// fn main() {
//     let wasm_path = std::env::current_dir()
//         .unwrap()
//         .join("../wasm/guest_component.wasm");

//     let req = http_service::Request {
//         method: http_service::Method::Get,
//         uri: "/".to_string(),
//         version: http_service::Version::HttpV10,
//         headers: vec![],
//     };

//     let mut config = wasmtime::Config::new().wasm_component_model(true);

//     let engine = Engine::new(&config).unwrap();
//     let mut store = Store::new(&engine, ());
//     let mut linker = Linker::new(&engine);

//     let ctx = WasiCtxBuilder::new().build();
//     // wasmtime_wasi::sync::snapshots::preview_1::add_wasi_snapshot_preview1_to_linker(&mut linker, |s| s).unwrap();
//     // wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();

//     let component = Component::from_file(&engine, wasm_path).unwrap();

//     let (_, instance) = http_service::instantiate(&mut store, &component, &linker).unwrap();

//     println!(
//         "{:?}",
//         instance
//             .get_typed_func::<(http_service::Request,), (http_service::Response,), _>(
//                 &mut store,
//                 "handle-http-request"
//             )
//             .unwrap()
//             .call(&mut store, (req,))
//             .unwrap()
//     );
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let test: Arc<Mutex<String>> = Arc::new(Mutex::from("Foo".to_string()));

//     let addr = SocketAddr::from(([127, 0, 0, 1], 4321));

//     let make_svc = make_service_fn(|_conn| {
//         let test_for_closure = test.clone();
//         let svc_fn = service_fn(move |req: Request<Body>| {
//             let test_for_closure = test_for_closure.clone();
//             async move {
//                 if req.version() == Version::HTTP_11 {
//                     let foo = test_for_closure.lock().unwrap();
//                     Ok(Response::new(Body::from(foo.clone())))
//                 } else {
//                     Err("not HTTP/1.1, abort connection")
//                 }
//             }
//         });
//         async move { Ok::<_, Infallible>(svc_fn) }
//     });

//     let server = Server::bind(&addr).serve(make_svc);

//     if let Err(e) = server.await {
//         eprintln!("server error: {}", e);
//     }

//     Ok(())
// }
