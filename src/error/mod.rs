use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] hyper::Error),
    
    #[error("HTTP error: {0}")]
    HttpHttpError(#[from] hyper::http::Error),
    
    #[error("URI parse error: {0}")]
    UriParseError(#[from] hyper::http::uri::InvalidUri),
    
    #[error("URI construction error: {0}")]
    UriConstructionError(#[from] hyper::http::uri::InvalidUriParts),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("TLS error: {0}")]
    TlsError(String),
    
    #[error("Other error: {0}")]
    Other(String),
}