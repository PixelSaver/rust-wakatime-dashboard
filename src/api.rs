use anyhow::anyhow;
use serde::Deserialize;
use eframe::egui::{self, Ui};
use crate::auth::TokenResponse;
use crate::{AppData, Screen, WakatimeDash};

#[derive(Debug, serde::Deserialize)]
pub struct TrustFactor {
    pub trust_level: String,
    pub trust_value: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct Me {
    pub id: u64,
    pub emails: Vec<String>,
    pub slack_id: Option<String>,
    pub github_username: Option<String>,
    // pub trust_factor: TrustFactor,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Project {
    pub name: String,
    pub total_seconds: i64,
    pub repo_url: Option<String>,
    pub languages: Vec<String>,
    pub archived: bool,
}
impl Project {
    fn display(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(&self.name);
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                ui.label(format!("{}s", self.total_seconds));
            });
        });
    }
}

#[derive(Deserialize, Debug)]
struct ProjectListItem {
    name: String,
    total_seconds: i64,
    languages: Vec<String>,
    archived: bool,
}
#[derive(Deserialize, Debug)]
struct ProjectListResponse {
    projects: Vec<ProjectListItem>,
}
#[derive(Deserialize, Debug)]
pub struct ProjectDetails {
    pub repo_url: Option<String>,
}
#[derive(Deserialize, Debug)]
struct ProjectDetailsResponse {
    projects: Vec<ProjectDetails>,
}

pub async fn fetch_me(token: &str) -> anyhow::Result<Me> {
    let client = reqwest::Client::new();
    // println!("{}", token);

    let url = format!("https://hackatime.hackclub.com/api/v1/authenticated/me");

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    let status = response.status();

    if !status.is_success() {
        let text = response.text().await?;
        return Err(anyhow!("Request failed: {} - {}", status, text,));
    }
    let body = response.json::<Me>().await?;
    // println!("Me: {:?}", body);
    Ok(body)
}
async fn fetch_project_list(token: &str) -> anyhow::Result<Vec<ProjectListItem>> {
    let client = reqwest::Client::new();

    let url = format!("https://hackatime.hackclub.com/api/v1/authenticated/projects");

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    let status = response.status();

    if !status.is_success() {
        let text = response.text().await?;
        return Err(anyhow!("Request failed: {} - {}", status, text,));
    }
    let body = response.json::<ProjectListResponse>().await?;
    Ok(body.projects)
}
async fn fetch_project_details(
    token: &str,
    username: String,
) -> anyhow::Result<Vec<ProjectDetails>> {
    let client = reqwest::Client::new();

    let url = format!(
        "https://hackatime.hackclub.com/api/v1/users/{}/projects/details",
        username
    );

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    let status = response.status();

    if !status.is_success() {
        let text = response.text().await?;
        return Err(anyhow!("Request failed: {} - {}", status, text,));
    }
    let body = response.json::<ProjectDetailsResponse>().await?;
    Ok(body.projects)
}
pub async fn fetch_projects(token: &str, user_id: u64) -> anyhow::Result<Vec<Project>> {
    let mut projects: Vec<Project> = fetch_project_list(&token)
        .await?
        .into_iter()
        .map(|p| Project {
            name: p.name,
            total_seconds: p.total_seconds,
            languages: p.languages,
            archived: p.archived,
            repo_url: None,
        })
        .collect();

    let project_details = fetch_project_details(&token, user_id.to_string()).await?;
    for (project, detail) in projects.iter_mut().zip(project_details.iter()) {
        project.repo_url = detail.repo_url.clone();
    }
    
    Ok(projects)
}
