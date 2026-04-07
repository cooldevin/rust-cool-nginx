use hyper::header::HeaderValue;
use hyper::{Body, Response};

/// Add standard CORS headers to a response builder for full API access.
pub fn add_cors_headers(builder: hyper::http::response::Builder) -> hyper::http::response::Builder {
    builder
        .header("Access-Control-Allow-Origin", "*")
        .header(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        )
        .header(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        )
}

/// Insert CORS headers into an existing response (e.g. from static file serving).
pub fn insert_cors_headers(response: &mut Response<Body>) {
    let headers = response.headers_mut();
    headers.insert(
        "Access-Control-Allow-Origin",
        HeaderValue::from_static("*"),
    );
    headers.insert(
        "Access-Control-Allow-Methods",
        HeaderValue::from_static("GET, OPTIONS"),
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        HeaderValue::from_static("Content-Type"),
    );
}

/// Build a CORS pre-flight (OPTIONS) response.
pub fn preflight_response() -> Response<Body> {
    add_cors_headers(Response::builder())
        .header("Access-Control-Max-Age", "86400")
        .status(200)
        .body(Body::from(""))
        .expect("failed to build preflight response")
}
