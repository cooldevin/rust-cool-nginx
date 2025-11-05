# Rust-Cool-Nginx

[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)
基于 Rust 语言实现的高性能 Web 服务器，借鉴 Nginx 的核心功能设计理念，提供反向代理、静态文件服务、负载均衡等特性。

## 🌟 特性

### 基础功能
- **高性能异步处理** - 基于 Tokio 和 Hyper 构建的异步运行时
- **静态文件服务** - 高效处理 HTML、CSS、JavaScript、图片等静态资源
- **反向代理** - 支持 HTTP/HTTPS 协议代理
- **SSL/TLS 支持** - 安全的 HTTPS 连接支持

### 高级功能
- **负载均衡** - 支持多种负载均衡算法（轮询、加权轮询等）
- **内容压缩** - Gzip 压缩优化网络传输
- **访问控制** - IP 黑白名单和请求频率限制
- **WebSocket 支持** - 实时双向通信支持
- **缓存机制** - HTTP 缓存优化性能

### 安全特性
- **IP 访问控制** - 基于 IP 的访问控制列表
- **速率限制** - 防止恶意请求和 DDoS 攻击
- **SSL/TLS 加密** - 数据传输加密保护

## 🚀 快速开始

### 环境要求
- Rust 1.70 或更高版本
- Cargo 包管理器

## 🙏 鸣谢

- [Tokio](https://tokio.rs/) - 异步运行时
- [Hyper](https://hyper.rs/) - HTTP 实现
- [Nginx](https://nginx.org/) - 灵感来源