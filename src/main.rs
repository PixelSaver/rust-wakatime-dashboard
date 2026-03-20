use crate::auth::TokenResponse;
use anyhow::anyhow;
use eframe::egui;
use tokio::{runtime::Runtime, sync::mpsc};
mod api;
use api::*;
mod auth;
mod cache_token;

#[derive(Debug)]
enum Screen {
    Login,
    Projects,
    Leaderboard,
    User,
    YSWS,
}
struct AppData {
    token: Option<TokenResponse>,
    projects: Option<Vec<Project>>,
    user: Option<api::Me>,
    is_loading: bool,
}

pub enum AsyncMessage {
    LoginSuccess(TokenResponse, api::Me),
    UserLoaded(api::Me),
    ProjectsLoaded(Vec<Project>),
    Error(String)
}

struct WakatimeDash {
    data: AppData,
    screen: Screen,
    error: Option<String>,
    rt: Runtime,
    tx: mpsc::UnboundedSender<AsyncMessage>,
    rx: mpsc::UnboundedReceiver<AsyncMessage>,
}

impl WakatimeDash {
    fn show_login(&mut self, ui: &mut egui::Ui) {
        ui.heading("Login");

        if self.data.is_loading {
            ui.heading("Loading...");
        }

        if let Some(e) = &self.error {
            ui.colored_label(egui::Color32::RED, e);
        }

        if ui.button("Login with Wakatime").clicked() {
            self.data.is_loading = true;
            self.error = None;
            
            let tx = self.tx.clone();
            
            self.rt.spawn(async move {
                let result: anyhow::Result<(TokenResponse, api::Me)> = async {
                    let token = auth::auth_user().await?;
                    let user = api::fetch_me(&token.access_token).await?;
                    
                    Ok((token, user))
                }.await;
                match result {
                    Ok((token, user)) => {
                        let _ = tx.send(AsyncMessage::LoginSuccess( token, user ));
                    },
                    Err(e) => {
                        let _ = tx.send(AsyncMessage::Error(e.to_string()));
                    }
                }
            });
        }
    }
    fn show_user(&mut self, ui: &mut egui::Ui) {
        ui.heading("User Profile");

        if self.data.is_loading {
            ui.heading("Loading...");
            return;
        }

        if let Some(e) = &self.error {
            ui.colored_label(egui::Color32::RED, e);
        }

        if let Some(user) = &self.data.user {
            if let Some(gh_username) = &user.github_username {
                ui.label(format!("GitHub: {}", gh_username));
            }
            if let Some(slack_id) = &user.slack_id {
                ui.label(format!("Slack ID: {}", slack_id));
            }
            // ui.label(format!(
            //     "Trust Level: {} ({})",
            //     user.trust_factor.trust_level,
            //     user.trust_factor.trust_value
            // ));

            ui.separator();

            ui.label("Emails:");
            for email in &user.emails {
                ui.label(format!("- {}", email));
            }
        } else {
            if ui.button("Load projects").clicked() {}
        }

        ui.separator();

        if ui.button("Go to projects").clicked() {
            self.screen = Screen::Projects;
        }
        if ui.button("Logout").clicked() {
            self.data.token = None;
            self.data.user = None;
            self.screen = Screen::Login;
        }
    }
    fn show_projects(&mut self, ui: &mut egui::Ui) {
        ui.heading("Projects");
        
        if self.data.is_loading {
            ui.heading("Loading...");
            return;
        }
        
        if let Some(p) = &self.data.projects {
            for project in p {
                ui.label(&project.name);
            }
        } else {
            if ui.button("Load projects").clicked() {
                self.data.is_loading = true;
                let tx = self.tx.clone();
                let token = self.data.token.as_ref().unwrap().access_token.clone();
                let user_id = self.data.user.as_ref().unwrap().id;
                
                self.rt.spawn(async move {
                    let projects = match api::fetch_projects(&token, user_id).await {
                        Ok(projects) => projects,
                        Err(e) => {
                            tx.send(AsyncMessage::Error(e.to_string())).ok();
                            return;
                        }
                    };
                    tx.send(AsyncMessage::ProjectsLoaded(projects)).ok();
                });
                // let projects = match self.fetch_projects() {
                //     Ok(projects) => { 
                //         self.data.is_loading = false;
                //         projects 
                //     },
                //     Err(e) => {
                //         self.error = Some(e.to_string());
                //         eprintln!("Failed to fetch projects, {}", e);
                //         return;
                //     }
                // };
                // self.data.projects = Some(projects);
                // self.data.is_loading = false;
            }
            ui.label("No projects loaded");
        }
        
    }

    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            screen: Screen::Login,
            data: AppData {
                token: None,
                projects: None,
                user: None,
                is_loading: false,
            },
            error: None,
            rt: Runtime::new().unwrap(),
            tx,
            rx,
        }
    }
}

impl eframe::App for WakatimeDash {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(msg) = self.rx.try_recv() {
            match msg {
                AsyncMessage::LoginSuccess( token, user ) => {
                    self.data.token = Some(token);
                    self.data.user = Some(user);
                    self.screen = Screen::User;
                    self.data.is_loading = false;
                },
                AsyncMessage::ProjectsLoaded( projects ) => {
                    self.data.projects = Some(projects);
                    self.screen = Screen::Projects;
                    self.data.is_loading = false;
                }
                _ => {}
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.screen {
                Screen::Login => self.show_login(ui),
                Screen::User => self.show_user(ui),
                Screen::Projects => self.show_projects(ui),
                _ => {}
            }
        });
    }
}

fn main() -> anyhow::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rust Wackatime Dash",
        native_options,
        Box::new(|cc| Ok(Box::new(WakatimeDash::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run eframe: {}", e))?;
    Ok(())
}
