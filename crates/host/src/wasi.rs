// use wasmtime_wasi_host::WasiCtx;

// wit_bindgen_host_wasmtime_rust::generate!({
//     path: "../bindings/wit/testwasi.wit",
// });

// #[derive(Default)]
// pub struct RequestCtx {
//     wasi: WasiCtx,
// }

// impl testwasi::Testwasi for RequestCtx {
//     fn log(&mut self, bytes: Vec<u8>) -> Result<(), wasmtime_wasi::Error> {
//         match std::str::from_utf8(&bytes) {
//             Ok(s) => print!("{s}"),
//             Err(_) => print!("\nbinary: {bytes:?}"),
//         }
//         Ok(())
//     }

//     fn log_err(&mut self, bytes: Vec<u8>) -> Result<(), wasmtime_wasi::Error> {
//         match std::str::from_utf8(&bytes) {
//             Ok(s) => eprint!("{s}"),
//             Err(_) => eprint!("\nbinary: {bytes:?}"),
//         }
//         Ok(())
//     }
// }
