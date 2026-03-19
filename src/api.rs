use anyhow::anyhow;
use serde::Deserialize;

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
impl WakatimeDash {
    pub fn fetch_me(&mut self) -> anyhow::Result<Me> {
        self.data.is_loading = true;
        let client = reqwest::blocking::Client::new();
        let token = &self
            .data
            .token
            .as_ref()
            .ok_or(anyhow::anyhow!("Token not found"))?
            .access_token;
        // println!("{}", token);

        let url = format!("https://hackatime.hackclub.com/api/v1/authenticated/me");

        let response = client.get(&url).header("Authorization", format!("Bearer {}", token)).send()?;
        
        let status = response.status();
        
        if !status.is_success() {
            let text = response.text()?;
            return Err(anyhow!(
                "Request failed: {} - {}",
                status,
                text,
            ));
        }
        let body = response.json::<Me>()?;
        println!("Me: {:?}", body);
        Ok(body)
    }
    fn fetch_project_list(&mut self) -> anyhow::Result<Vec<ProjectListItem>> {
        let client = reqwest::blocking::Client::new();
        let token = &self
            .data
            .token
            .as_ref()
            .ok_or(anyhow::anyhow!("Token not found"))?
            .access_token;

        let url = format!("https://hackatime.hackclub.com/api/v1/authenticated/projects");

        let response = client.get(&url).header("Authorization", format!("Bearer {}", token)).send()?;
        
        let status = response.status();
        
        if !status.is_success() {
            let text = response.text()?;
            return Err(anyhow!(
                "Request failed: {} - {}",
                status,
                text,
            ));
        }
        let body = response.json::<ProjectListResponse>()?;
        Ok(body.projects)
    }
    fn fetch_project_details(&mut self, username: String) -> anyhow::Result<Vec<ProjectDetails>> {
        let client = reqwest::blocking::Client::new();
        let token = &self
            .data
            .token
            .as_ref()
            .ok_or(anyhow::anyhow!("Token not found"))?
            .access_token;

        let url = format!("https://hackatime.hackclub.com/api/v1/users/{}/projects/details", username);

        let response = client.get(&url).header("Authorization", format!("Bearer {}", token)).send()?;
        
        let status = response.status();
        
        if !status.is_success() {
            let text = response.text()?;
            return Err(anyhow!(
                "Request failed: {} - {}",
                status,
                text,
            ));
        }
        let body = response.json::<ProjectDetailsResponse>()?;
        Ok(body.projects)
    }
    pub fn fetch_projects(&mut self) -> anyhow::Result<Vec<Project>> {
        let mut projects: Vec<Project> = self.fetch_project_list()?.into_iter().map(|p| {
            Project {
                name: p.name,
                total_seconds: p.total_seconds,
                languages: p.languages,
                archived: p.archived,
                repo_url: None,
            }
        }).collect();
        
        if let Some(user) = &self.data.user {
            let project_details = self.fetch_project_details(user.id.to_string())?;
            for (project, detail) in projects.iter_mut().zip(project_details.iter()) {
                project.repo_url = detail.repo_url.clone();
            }
        }
        println!("{:?}", projects);
        Ok(projects)
    }
    
}
