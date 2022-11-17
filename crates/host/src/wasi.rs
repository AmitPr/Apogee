use hyper::Body;


wit_bindgen_host_wasmtime_rust::generate!({
    path: "../bindings/wit/testwasi.wit",
});

#[derive(Default)]
pub struct RequestCtx {
    pub body: Body,
}

impl testwasi::Testwasi for RequestCtx {
    fn log(&mut self, bytes: Vec<u8>) -> Result<(), wasmtime_wasi::Error> {
        match std::str::from_utf8(&bytes) {
            Ok(s) => println!("{s}"),
            Err(_) => println!("\nbinary: {bytes:?}"),
        }
        Ok(())
    }

    fn log_err(&mut self, bytes: Vec<u8>) -> Result<(), wasmtime_wasi::Error> {
        match std::str::from_utf8(&bytes) {
            Ok(s) => eprintln!("{s}"),
            Err(_) => eprintln!("\nbinary: {bytes:?}"),
        }
        Ok(())
    }
}