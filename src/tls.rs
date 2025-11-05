use std::sync::Arc;
use tokio_rustls::{
    rustls::{Certificate, PrivateKey, ServerConfig, ClientConfig, RootCertStore},
    TlsAcceptor, TlsConnector,
};
use tokio::fs;

use crate::error::ProxyError;

pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
    pub client_auth: bool, // 是否启用客户端证书验证
    pub ocsp_stapling: bool, // 是否启用 OCSP 装订
}

impl TlsConfig {
    pub fn new(cert_path: String, key_path: String) -> Self {
        Self { 
            cert_path, 
            key_path,
            client_auth: false,
            ocsp_stapling: false,
        }
    }
    
    pub fn with_client_auth(mut self, client_auth: bool) -> Self {
        self.client_auth = client_auth;
        self
    }
    
    pub fn with_ocsp_stapling(mut self, ocsp_stapling: bool) -> Self {
        self.ocsp_stapling = ocsp_stapling;
        self
    }
}

pub async fn load_tls_config(config: &TlsConfig) -> Result<TlsAcceptor, ProxyError> {
    // 加载证书
    let cert_data = fs::read(&config.cert_path).await?;
    let certs = load_certs(&cert_data)?;
    
    // 加载私钥
    let key_data = fs::read(&config.key_path).await?;
    let mut keys = load_keys(&key_data)?;
    
    if keys.is_empty() {
        return Err(ProxyError::Other("No private keys found".into()));
    }
    
    let builder = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();
        
    let config = builder
        .with_single_cert(certs, keys.remove(0))
        .map_err(|err| ProxyError::Other(format!("Failed to create TLS config: {}", err)))?;
        
    Ok(TlsAcceptor::from(Arc::new(config)))
}

// 创建 TLS 客户端连接器
pub fn create_tls_client_connector() -> Result<TlsConnector, ProxyError> {
    let mut root_store = RootCertStore::empty();
    
    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
        
    Ok(TlsConnector::from(Arc::new(config)))
}

fn load_certs(data: &[u8]) -> Result<Vec<Certificate>, ProxyError> {
    let mut reader = std::io::Cursor::new(data);
    let certs = rustls_pemfile::certs(&mut reader)
        .map_err(|err| ProxyError::Other(format!("Failed to load certificates: {}", err)))?;
    Ok(certs.into_iter().map(Certificate).collect())
}

fn load_keys(data: &[u8]) -> Result<Vec<PrivateKey>, ProxyError> {
    let mut reader = std::io::Cursor::new(data);
    
    // 尝试加载 PKCS#8 私钥
    if let Ok(keys) = rustls_pemfile::pkcs8_private_keys(&mut reader) {
        if !keys.is_empty() {
            return Ok(keys.into_iter().map(PrivateKey).collect());
        }
    }
    
    // 重置 reader 并尝试加载 RSA 私钥
    reader.set_position(0);
    if let Ok(keys) = rustls_pemfile::rsa_private_keys(&mut reader) {
        Ok(keys.into_iter().map(PrivateKey).collect())
    } else {
        Ok(vec![])
    }
}

// TLS 终端代理功能
pub struct TlsTerminationProxy;

impl TlsTerminationProxy {
    pub fn new() -> Self {
        Self
    }
    
    // 处理 TLS 终端代理
    pub async fn handle_tls_termination(&self) -> Result<(), ProxyError> {
        // 这里应该实现 TLS 终端代理逻辑
        // 为简化起见，我们只是展示结构
        Ok(())
    }
}