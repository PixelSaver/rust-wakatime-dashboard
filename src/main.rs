use eframe::egui;
use anyhow;
use serde::Deserialize;
mod auth;

#[derive(Deserialize, Debug, Clone)]
struct Project {
    name: String,
    repo_url: Option<String>,
    total_seconds: i64,
    languages: Vec<String>,
}
#[derive(Deserialize, Debug)]
struct ProjectsResponse {
    projects: Vec<String>,
}
#[derive(Deserialize, Debug)]
struct ProjectDetailsResponse {
    projects: Vec<Project>,
}

enum AppState {
    Projects { projects: Option<Vec<Project>> },
    User,
    Leaderboard,
    YSWS,
}
struct WakatimeDash {
    username: String,
    state: AppState,
}

impl WakatimeDash {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            username: "pixelsaver".into(),
            state: AppState::Leaderboard,
        }
    }
}

impl eframe::App for WakatimeDash {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.state {
                AppState::Projects { projects } => {
                    if let Some(p) = projects {
                        for project in p {
                            ui.label(&project.name);
                        }
                    } else {
                        ui.label("No projects loaded");
                    }
                }
                _ => {}
            }
            if ui.button("Get projects").clicked() {
                let url =
                    format!("https://hackatime.hackclub.com/api/v1/users/pixelsaver/projects");
                let client = reqwest::blocking::Client::new();
                let project_names = client
                    .get(&url)
                    .send()
                    .unwrap()
                    .json::<ProjectsResponse>()
                    .unwrap()
                    .projects;
                let details_url = format!(
                    "https://hackatime.hackclub.com/api/v1/users/pixelsaver/projects/details"
                );
                let project_details = client
                    .get(&details_url)
                    .query(&[(
                        "projects",
                        project_names.join(","),
                    )])
                    .send()
                    .unwrap()
                    .json::<ProjectDetailsResponse>()
                    .unwrap();
                for project in &project_details.projects {
                    ui.label(&project.name);
                }
                self.state = AppState::Projects {
                    projects: Some(project_details.projects),
                };
            }
        });
    }
}

fn main() -> anyhow::Result<()> {
    // let native_options = eframe::NativeOptions::default();
    // eframe::run_native(
    //     "Rust Wackatime Dash",
    //     native_options,
    //     Box::new(|cc| Ok(Box::new(WakatimeDash::new(cc)))),
    // )?;
    // let client = reqwest::blocking::Client::new();
    // let url = format!("https://hackatime.hackclub.com/api/summary");
    // let content = client.get(&url).send().unwrap().text().unwrap();
    // println!("{:?}", content);
    auth::auth_user()?;
    Ok(())
}
