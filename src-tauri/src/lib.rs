#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      // 在这里启动我们的Web服务器
      start_server();
      
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

fn start_server() {
    std::thread::spawn(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            use hyper::service::{make_service_fn, service_fn};
            use hyper::{Body, Request, Server};
            use hyper_staticfile::Static;
            use std::convert::Infallible;
            use std::net::SocketAddr;
            use std::path::Path;

            let static_files = Static::new(Path::new("../frontend"));

            let make_svc = make_service_fn(move |_conn| {
                let static_files = static_files.clone();
                async move {
                    Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                        let static_files = static_files.clone();
                        async move {
                            // 提供静态文件服务
                            let response = static_files.serve(req).await;
                            Ok::<_, Infallible>(response.unwrap())
                        }
                    }))
                }
            });

            let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
            let server = Server::bind(&addr).serve(make_svc);
            
            println!("Server running on http://{}", addr);

            if let Err(e) = server.await {
                eprintln!("Server error: {}", e);
            }
        });
    });
}