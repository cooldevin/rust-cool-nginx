# Rust Cool Nginx Desktop App

这是一个基于 Rust 和 Tauri 构建的桌面应用程序，将原有的 Web 服务器功能封装为桌面应用。

## 功能特点

- 将原有的 Web 服务器功能封装为 Windows 桌面应用
- 使用 JSON 文件存储配置信息
- 支持动态刷新配置，无需重启应用
- 提供直观的图形用户界面
- 响应式窗口大小，根据屏幕尺寸自动调整
- 集成静态文件服务器功能

## 构建和运行

### 前置条件

- Rust 开发环境 (推荐使用最新稳定版)

### 构建应用

```bash
# 进入 Tauri 项目目录
cd src-tauri

# 构建应用
cargo build
```

或者使用 Tauri CLI (需要先安装):

```bash
# 安装 Tauri CLI
cargo install tauri-cli

# 构建应用
cargo tauri build
```

### 开发模式运行

```bash
# 进入 Tauri 项目目录
cd src-tauri

# 在开发模式下运行应用
cargo run
```

或者使用 Tauri CLI:

```bash
# 安装 Tauri CLI (如果尚未安装)
cargo install tauri-cli

# 在开发模式下运行应用
cargo tauri dev
```

## 项目结构

```
├── public/                 # 前端静态资源
│   ├── index.html          # 主页
│   ├── config.html         # 配置管理页面
│   ├── status.html         # 监控状态页面
│   ├── static-file.html    # 静态文件服务器页面
│   ├── file-viewer.html    # 文件查看器页面
│   └── css/                # 样式文件
├── src-tauri/              # Tauri 桌面应用代码
│   ├── src/                
│   │   ├── main.rs         # Tauri 应用入口
│   │   └── lib.rs          # 应用核心逻辑
│   ├── Cargo.toml          # Tauri 依赖配置
│   └── tauri.conf.json     # Tauri 配置文件
├── nginx.conf              # 默认配置文件
├── logs/                   # 日志目录
└── ...
```

## 数据存储

应用使用 JSON 文件存储配置信息，配置文件为 `nginx.conf`，会在应用首次运行时自动创建。

## 动态刷新

当用户修改配置并保存后，应用会：
1. 将新配置保存到 JSON 配置文件
2. 更新内存中的配置
3. 应用新配置，无需重启整个应用

## 页面功能

- **主页 (index.html)** - 应用入口和功能导航
- **配置管理 (config.html)** - NGINX 服务器配置的查看和修改
- **监控状态 (status.html)** - 服务器状态监控和实时请求查看
- **静态文件服务器 (static-file.html)** - 静态文件服务器功能管理和测试
- **文件查看器 (file-viewer.html)** - 查看服务器上的文件内容

## 支持平台

当前版本主要支持 Windows 平台，后续可以扩展支持 Linux 和 macOS。