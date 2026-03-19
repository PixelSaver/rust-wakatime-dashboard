use crate::auth::TokenResponse;
use anyhow::anyhow;
use eframe::egui;
use serde::Deserialize;
mod api;
use api::Project;
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

struct WakatimeDash {
    data: AppData,
    screen: Screen,
    error: Option<String>,
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
            match auth::auth_user() {
                Ok(token_response) => {
                    self.data.token = Some(token_response);

                    match self.fetch_me() {
                        Ok(user) => {
                            self.data.user = Some(user);
                            self.screen = Screen::User;
                        }
                        Err(e) => {
                            self.error =
                                Some(anyhow!("Fetching User Data failed: {}", e).to_string());
                        }
                    }
                }
                Err(e) => {
                    self.error = Some(anyhow!("Authenticate failed: {}", e).to_string());
                }
            }

            self.data.is_loading = false;
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
                let projects = match self.fetch_projects() {
                    Ok(projects) => { 
                        self.data.is_loading = false;
                        projects 
                    },
                    Err(e) => {
                        self.error = Some(e.to_string());
                        eprintln!("Failed to fetch projects, {}", e);
                        return;
                    }
                };
                self.data.projects = Some(projects);
                self.data.is_loading = false;
            }
            ui.label("No projects loaded");
        }
        
    }

    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            screen: Screen::Login,
            data: AppData {
                token: None,
                projects: None,
                user: None,
                is_loading: false,
            },
            error: None,
        }
    }
}

impl eframe::App for WakatimeDash {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.screen {
                Screen::Login => self.show_login(ui),
                Screen::User => self.show_user(ui),
                // Screen::Leaderboard => self.show_leaderboard(ui),
                // Screen::YSWS => self.show_ysws(ui),
                Screen::Projects => self.show_projects(ui),
                _ => {}
            }
            // if ui.button("Get projects").clicked() {
            //     let url =
            //         format!("https://hackatime.hackclub.com/api/v1/users/pixelsaver/projects");
            //     let client = reqwest::blocking::Client::new();
            //     let project_names = client
            //         .get(&url)
            //         .send()
            //         .unwrap()
            //         .json::<ProjectsResponse>()
            //         .unwrap()
            //         .projects;
            //     let details_url = format!(
            //         "https://hackatime.hackclub.com/api/v1/users/pixelsaver/projects/details"
            //     );
            //     let project_details = client
            //         .get(&details_url)
            //         .query(&[(
            //             "projects",
            //             project_names.join(","),
            //         )])
            //         .send()
            //         .unwrap()
            //         .json::<ProjectDetailsResponse>()
            //         .unwrap();
            //     for project in &project_details.projects {
            //         ui.label(&project.name);
            //     }
            // }
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
    // let client = reqwest::blocking::Client::new();
    // let url = format!("https://hackatime.hackclub.com/api/summary");
    // let content = client.get(&url).send().unwrap().text().unwrap();
    // println!("{:?}", content);
    Ok(())
}
