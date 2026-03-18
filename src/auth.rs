use tiny_http::{Server, Response};
use std::process::Command;

fn wait_for_code() -> String {
    let server = Server::http("127.0.0.1:8080").unwrap();
    
    for request in server.incoming_requests() {
        let url = request.url().to_string();
        
    }
}

fn open_browser(auth_url: &str) {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", auth_url]).spawn().unwrap();
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(auth_url).spawn().unwrap();
    } else {
        Command::new("xdg-open").arg(auth_url).spawn().unwrap();
    }
}
