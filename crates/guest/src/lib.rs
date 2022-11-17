use sdk::entrypoint;
use sdk::http::{Request, Response};
// use std::io::Read;
// use std::net::TcpStream;
// use std::os::fd::{FromRawFd, OwnedFd};


#[entrypoint(http)]
pub fn handle_http_request(req: Request) -> Result<Response, String> {
    println!("Got request: {req:?}");
    // println!("Attempting to read from fd 3");
    // let fd = unsafe { OwnedFd::from_raw_fd(3) };
    // let mut stream = TcpStream::from(fd);
    // let mut buf = [0; 1024];
    // let n = stream.read(&mut buf).unwrap();
    // println!("read {n} bytes");
    if req.headers.iter().any(|h| h.key == b"x-wit-throw-error") {
        Err("Error - x-wit-throw-error header found".to_string())
    } else {
        Ok(Response {
            status: 200,
            version: req.version,
            headers: vec![sdk::http::Header {
                key: b"x-wit-test".to_vec(),
                value: b"true".to_vec(),
            }],
        })
    }
}
