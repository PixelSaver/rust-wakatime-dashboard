use anyhow::Result;
use pkce;
use std::process::Command;
use tiny_http::{Response, Server};

const CLIENT_ID: &str = "56Q7QczaBEH6o8J9YTtMReNIsHOwOrZPB59RAZj64nI";

fn gen_pkce() -> (Vec<u8>, String) {
    let verify = pkce::code_verifier(128);
    let challenge = pkce::code_challenge(&verify);
    (verify, challenge)
}

fn get_url(challenge: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let request = client
        .get("https://hackatime.hackclub.com/oauth/authorize")
        .query(&[
            ("client_id", CLIENT_ID),
            ("redirect_uri", "http://127.0.0.1:8080/callback"),
            ("response_type", "code"),
            ("scope", "profile read"),
            ("state", "RANDOM_STRING"),
            ("code_challenge", &challenge),
            ("code_challenge_method", "S256"),
        ])
        .build()?;
    Ok(request.url().to_string())
}

fn wait_for_code() -> anyhow::Result<String> {
    let server = Server::http("127.0.0.1:8080").unwrap();

    for request in server.incoming_requests() {
        let url = request.url().to_string();

        if let Some(start) = url.find("code=") {
            // Skip the 'code=' thing
            let code_part = &url[(start + 5)..];
            if let Some(code) = code_part.split("&").next() {
                request.respond(Response::from_string(
                    "Login successful! Close this tab and navigate back to the app.",
                ))?;

                return Ok(code.to_string());
            } else {
                return Err(anyhow::anyhow!("Invalid code response"));
            };
        } else {
            request.respond(Response::from_string("Waiting for login details..."))?;
        }
    }
    unreachable!()
}

fn open_browser(auth_url: &str) {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", auth_url])
            .spawn()
            .unwrap();
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(auth_url).spawn().unwrap();
    } else {
        Command::new("xdg-open").arg(auth_url).spawn().unwrap();
    }
}

pub fn auth_user() -> Result<()> {
    let (_verifier, challenge) = gen_pkce();

    let auth_url = get_url(&challenge)?;

    open_browser(&auth_url);
    let code = wait_for_code()?;

    println!("Got code: {}", code);

    Ok(())
}
