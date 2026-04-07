mod config;
mod cors;
mod monitoring;
mod server;

use std::sync::{Arc, RwLock};

use config::ServerConfig;

/// Global configuration, loaded once at startup.
static CONFIG: std::sync::OnceLock<Arc<RwLock<ServerConfig>>> = std::sync::OnceLock::new();

fn global_config() -> &'static Arc<RwLock<ServerConfig>> {
    CONFIG.get_or_init(config::load_config)
}

// ── Tauri commands ──────────────────────────────────────────────────────────

/// Return the current in-memory configuration.
#[tauri::command]
fn get_config() -> Result<ServerConfig, String> {
    let cfg = global_config()
        .read()
        .map_err(|e| format!("读取配置失败: {}", e))?;
    println!(
        "从内存加载配置成功: {:?}, 静态文件根目录: {}, 静态文件服务: {}, 反向代理: {}",
        cfg.server.listen_addr,
        cfg.server.static_root,
        cfg.features.static_file_serving,
        cfg.features.reverse_proxy,
    );
    Ok(cfg.clone())
}

/// Persist a new configuration to memory and disk.
#[tauri::command]
fn update_config(new_config: ServerConfig) -> Result<(), String> {
    println!("开始更新配置: {:?}", new_config);

    {
        let mut cfg = global_config()
            .write()
            .map_err(|e| format!("获取写入锁失败: {}", e))?;
        *cfg = new_config.clone();
    }

    config::save_config(&new_config)?;
    Ok(())
}

// ── Entry point ─────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Manager;

    let result = tauri::Builder::default()
        .setup(|app| {
            // Adjust window size
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_size(tauri::PhysicalSize::new(1920, 1080));
                let _ = window.center();
                println!("窗口已调整大小: 1920x1080");
            }

            // Start the embedded HTTP server
            server::start(Arc::clone(global_config()));

            if cfg!(debug_assertions) {
                if let Err(e) = app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                ) {
                    eprintln!("Failed to initialize log plugin: {}", e);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_config, update_config])
        .run(tauri::generate_context!());

    match result {
        Ok(_) => println!("Tauri application exited successfully"),
        Err(e) => {
            eprintln!("Error while running tauri application: {}", e);
            std::process::exit(1);
        }
    }
}
