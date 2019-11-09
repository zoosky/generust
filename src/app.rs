use crate::server::start_server;

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
pub(crate) fn start(cfg: {{crate_name}}_service::AppConfig) -> {{crate_name}}_core::Result<()> {
  let (port_tx, port_rx) = std::sync::mpsc::channel();
  match cfg.task().as_ref() {
    "app" => {
      let _ = std::thread::spawn(move || start_server(cfg, port_tx));
      let port = port_rx.recv().unwrap();
      web_view::builder()
        .title({{crate_name}}_core::APPNAME)
        .content(web_view::Content::Url(format!("http://127.0.0.1:{}", port)))
        .size(1920, 1080)
        .resizable(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .run()
        .map_err(|e| {{crate_name}}_core::Error::from(format!("Error creating webview: {:?}", e)))
    }
    _ => start_server(cfg, port_tx)
  }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub(crate) fn start(cfg: {{crate_name}}_service::AppConfig) -> {{crate_name}}_core::Result<()> {
  let (port_tx, _) = std::sync::mpsc::channel();
  start_server(cfg, port_tx)
}
