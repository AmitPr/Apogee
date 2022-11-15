use sdk::entrypoint;
use sdk::http::{Request, Response};

#[entrypoint(http)]
pub fn handle_http_request(req: Request) -> Result<Response, String> {
    if req.headers.iter().any(|(k, _)| k == "x-wit-throw-error") {
        Err("Error - x-wit-throw-error header found".to_string())
    } else {
        Ok(Response {
            status: 200,
            headers: vec![("x-wit-test".to_string(), "true".to_string())],
            version: req.version,
        })
    }
}
