# Rust Cool Nginx Desktop App

这是一个基于 Rust 和 Tauri 构建的桌面应用程序，将原有的 Web 服务器功能封装为桌面应用。

## 功能特点

- 将原有的 Web 服务器功能封装为 Windows 桌面应用
- 使用 SQLite 数据库存储配置信息
- 支持动态刷新配置，无需重启应用
- 提供直观的图形用户界面

## 构建和运行

### 前置条件

- Rust 开发环境 (推荐使用最新稳定版)
- 安装 Tauri CLI: `cargo install tauri-cli`

### 构建应用

```bash
# 构建应用
cargo tauri build
```

### 开发模式运行

```bash
# 在开发模式下运行应用
cargo tauri dev
```

## 项目结构

```
├── public/                 # 前端静态资源
├── src/                    # Rust 核心代码
│   ├── config/             # 配置管理模块
│   ├── db/                 # 数据库操作模块
│   ├── modules/            # 功能模块
│   └── ...
├── src-tauri/              # Tauri 桌面应用代码
│   ├── main.rs             # Tauri 应用入口
│   ├── Cargo.toml          # Tauri 依赖配置
│   └── tauri.conf.json     # Tauri 配置文件
├── nginx.conf              # 默认配置文件
└── ...
```

## 数据存储

应用使用 SQLite 数据库存储配置信息，数据库文件为 `config.db`，会在应用首次运行时自动创建。

## 动态刷新

当用户修改配置并保存后，应用会：
1. 将新配置保存到 SQLite 数据库
2. 更新内存中的配置
3. 应用新配置，无需重启整个应用

## 支持平台

当前版本主要支持 Windows 平台，后续可以扩展支持 Linux 和 macOS。