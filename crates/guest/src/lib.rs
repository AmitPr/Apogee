use apogee_sdk::entrypoint;
use apogee_sdk::filesystem;
use apogee_sdk::http::{Request, Response};

#[entrypoint(http)]
pub fn handle_http_request(req: Request) -> Result<Response, String> {
    // Read the index.html file from the filesystem
    let index_html = filesystem::read_file("index.html")?;
    // format {route} into the index.html file
    let index_html = std::str::from_utf8(&index_html)
        .unwrap()
        .replace("{route}", &req.uri);
    if req.headers.iter().any(|h| h.key == b"x-wit-throw-error") {
        Err("Error - x-wit-throw-error header found".to_string())
    } else {
        Ok(Response {
            status: 200,
            version: req.version,
            headers: vec![apogee_sdk::http::Header {
                key: b"x-wit-test".to_vec(),
                value: b"true".to_vec(),
            }],
            body: index_html.into_bytes(),
        })
    }
}
