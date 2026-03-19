use anyhow::Result;
use pkce;
use rand::{self, RngExt};
use serde::{Deserialize, Serialize};
use open;
use chrono::{DateTime, Utc};
use tiny_http::{Response, Server};

use crate::cache_token;

const CLIENT_ID: &str = "56Q7QczaBEH6o8J9YTtMReNIsHOwOrZPB59RAZj64nI";
const REDIRECT_URI: &str = "http://127.0.0.1:49153/callback";

#[derive(Deserialize, Serialize, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub scope: String,
    pub created_at: i64,
}

impl TokenResponse {
    pub fn is_valid(&self) -> bool {
        let expires = match DateTime::from_timestamp(self.expires_in + self.created_at, 0) {
            Some(e) => e,
            None => return false,
        };
        let now: DateTime<Utc> = DateTime::from(Utc::now());
        return now < expires;
    }
}

fn gen_pkce() -> (Vec<u8>, String) {
    let verify = pkce::code_verifier(128);
    let challenge = pkce::code_challenge(&verify);
    (verify, challenge)
}

fn get_auth_code_url(challenge: &str, state: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let request = client
        .get("https://hackatime.hackclub.com/oauth/authorize")
        .query(&[
            ("client_id", CLIENT_ID),
            ("redirect_uri", REDIRECT_URI),
            ("response_type", "code"),
            ("scope", "profile read"),
            ("state", &state),
            ("code_challenge", &challenge),
            ("code_challenge_method", "S256"),
        ])
        .build()?;
    Ok(request.url().to_string())
}

// fn get_token_url(verify: &str, auth_code: &str) -> Result<String> {
//     let client = reqwest::blocking::Client::new();
//     Ok(request.url().to_string())
// }

fn wait_for_code() -> anyhow::Result<String> {
    let server = Server::http("127.0.0.1:49153").unwrap();

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
    open::that(auth_url).unwrap();
}

pub fn auth_user() -> Result<TokenResponse> {
    if let Some(token) = cache_token::load_token() {
        if token.is_valid() {
            return Ok(token);
        }
    };
    
    
    let (verifier, challenge) = gen_pkce();
    let state: String = rand::rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    let auth_url = get_auth_code_url(&challenge, &state)?;
    println!("Auth code url: {:?}", auth_url);

    open_browser(&auth_url);
    let code = wait_for_code()?;

    // println!("Got code: {}", code);

    // let token_url = get_token_url(&String::from_utf8(verifier)?, &code)?;

    let client = reqwest::blocking::Client::new();
    let request = client
        .post("https://hackatime.hackclub.com/oauth/token")
        .query(&[
            ("client_id", CLIENT_ID),
            ("code", &code),
            ("redirect_uri", REDIRECT_URI),
            ("grant_type", "authorization_code"),
            ("code_verifier", &String::from_utf8(verifier)?),
        ])
        .build()?;
    println!("Request: {:?}", request.url());
    let request = client.post(&request.url().to_string()).send()?;
    let _status = request.status();
    let text = request.text()?;

    // println!("Status: {}", status);
    // println!("Raw body: {}", text);
    
    // let response = client
    //     .post(&token_url)
    //     .send()?
    //     .json::<TokenResponse>()?;

    // println!("Auth code: {}", response.access_token);
    // 
    let token = serde_json::from_str::<TokenResponse>(&text)?;
    cache_token::save_token(&token)?;
    Ok(token)
}
