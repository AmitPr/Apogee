use sdk::entrypoint;
use sdk::http::{Request, Response};


#[entrypoint(http)]
pub fn handle_http_request(req: Request) -> Result<Response, String> {
    println!("Got request!");
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
            body: b"Hello, world!".to_vec(),
        })
    }
}
