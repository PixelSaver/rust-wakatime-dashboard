use eframe::egui;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
struct Project {
    name: String,
    repo: Option<String>,

    #[serde(rename = "total_seconds")]
    time: i64,
}
#[derive(Deserialize, Debug)]
struct ProjectsResponse {
    projects: Vec<String>,
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
            username: "PixelSaver".into(),
            state: AppState::Leaderboard,
        }
    }
}

impl eframe::App for WakatimeDash {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
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
                    .text()
                    .unwrap();
                ui.label(&project_details);
                println!("Project details: {:?}", project_details);
            }
        });
    }
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rust Wackatime Dash",
        native_options,
        Box::new(|cc| Ok(Box::new(WakatimeDash::new(cc)))),
    )?;
    Ok(())
}
