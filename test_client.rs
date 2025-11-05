use hyper::{Method, Request};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use http_body_util::Full;
use hyper::body::{Bytes};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 创建客户端
    let client = Client::builder(TokioExecutor::new()).build_http();
    
    // 创建请求
    let req = Request::builder()
        .method(Method::POST)
        .uri("http://127.0.0.1:3000/admin-api/est/v1/lane-cabin/page")
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from("")))?;

    // 发送请求
    let res = client.request(req).await?;

    println!("Response Status: {}", res.status());
    let body = http_body_util::BodyExt::collect(res.into_body()).await?;
    let body_bytes = body.to_bytes();
    println!("Response Body: {:?}", String::from_utf8_lossy(&body_bytes));

    Ok(())
}