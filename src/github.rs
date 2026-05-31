use crate::errors::SleuthError;
use octocrab::{Octocrab, models, params::workflows::Filter};
use std::collections::HashMap;

pub struct Step {
    pub name: String,
    pub status: models::workflows::Status,
    pub conclusion: Option<models::workflows::Conclusion>,
}

pub struct WorkflowJob {
    pub id: u64,
    pub status: models::workflows::Status,
    pub conclusion: Option<models::workflows::Conclusion>,
    pub name: String,
    pub steps: Vec<Step>,
}

pub struct WorkflowRun {
    pub id: u64,
    pub name: String,
    pub workflow_id: u64,
    pub status: String,
    pub conclusion: Option<String>,
    pub jobs: Vec<WorkflowJob>,
    // TODO: should we store failed logs in this struct?
    pub failed_jobs_logs: HashMap<u64, String>,
}

pub fn get_github_client() -> Result<Octocrab, octocrab::Error> {
    // TODO: add logging
    match std::env::var("GITHUB_TOKEN") {
        Ok(token) if !token.trim().is_empty() => Octocrab::builder().personal_token(token).build(),
        _ => Octocrab::builder().build(),
    }
}

async fn fetch_run(
    owner: &str,
    repo: &str,
    run_id: u64,
    octocrab: &Octocrab,
) -> Result<WorkflowRun, octocrab::Error> {
    let workflows = octocrab.workflows(owner, repo);
    let run = workflows.get(run_id.into()).await?;
    let jobs_page = workflows
        .list_jobs(run_id.into())
        .per_page(100)
        .filter(Filter::All)
        .send()
        .await?;

    let jobs = jobs_page
        .into_iter()
        .map(|job| WorkflowJob {
            id: job.id.into_inner(),
            status: job.status,
            conclusion: job.conclusion,
            name: job.name,
            steps: job
                .steps
                .into_iter()
                .map(|step| Step {
                    name: step.name,
                    status: step.status,
                    conclusion: step.conclusion,
                })
                .collect(),
        })
        .collect();

    Ok(WorkflowRun {
        id: run.id.into_inner(),
        name: run.name,
        workflow_id: run.workflow_id.into_inner(),
        status: run.status,
        conclusion: run.conclusion,
        jobs,
        failed_jobs_logs: HashMap::new(),
    })
}

pub async fn fetch_job_logs(
    owner: &str,
    repo: &str,
    job_id: u64,
    requests_client: &reqwest::Client,
) -> Result<String, SleuthError> {
    // TODO: this endpoint requires authentication. We should fallback to
    //  GET repos/OWNER/REPO/actions/runs/RUN_ID/logs if the user is not authenticated.
    //  This downloads all logs in a zip file
    let url = format!("https://api.github.com/repos/{owner}/{repo}/actions/jobs/{job_id}/logs");
    let mut request = requests_client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2026-03-10")
        .header("User-Agent", "sleuth");

    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.trim().is_empty() {
            request = request.bearer_auth(token);
        }
    }

    let response = request.send().await?.error_for_status()?;

    Ok(response.text().await?)
}

// Get run metadata and logs and print nice report
pub async fn get_run_data(
    repo: &str,
    run_id: u64,
    github_client: &Octocrab,
    requests_client: &reqwest::Client,
) -> Result<WorkflowRun, SleuthError> {
    let (owner, repo) = repo.split_once('/').ok_or_else(|| {
        SleuthError::InvalidFormat(String::from(
            "Invalid repository format, expected 'owner/repo'",
        ))
    })?;

    if owner.is_empty() || repo.is_empty() || repo.contains('/') {
        return Err(SleuthError::InvalidFormat(String::from(
            "Invalid repository format, expected 'owner/repo'",
        )));
    }

    let mut run = fetch_run(owner, repo, run_id, &github_client).await?;
    let mut logs = HashMap::<u64, String>::new();
    for job in &run.jobs {
        if job.conclusion == Some(models::workflows::Conclusion::Failure)
            || job.conclusion == Some(models::workflows::Conclusion::TimedOut)
        {
            let log = fetch_job_logs(owner, repo, job.id, &requests_client).await?;
            logs.insert(job.id, log);
        }
    }
    run.failed_jobs_logs = logs;

    Ok(run)
}

pub fn print_run(run: WorkflowRun) {
    println!();
    println!("GitHub Actions Run");
    println!("==================");
    println!("Run ID:      {}", run.id);
    println!("Name:        {}", run.name);
    println!("Workflow ID: {}", run.workflow_id);
    println!("Status:      {}", format!("{}", run.status).to_lowercase());
    println!(
        "Conclusion:  {}",
        run.conclusion.unwrap_or_else(|| "unknown".to_string())
    );
    println!();

    if run.jobs.is_empty() {
        println!("Jobs: none found");
        return;
    }

    println!("Jobs");
    println!("----");

    for job in &run.jobs {
        let marker = marker_for_conclusion(job.conclusion.as_ref());

        println!(
            "{} {}  [{} / {}]",
            marker,
            job.name,
            format!("{:?}", job.status).to_lowercase(),
            format_conclusion(job.conclusion.as_ref())
        );
        println!("  Job ID: {}", job.id);

        if job.steps.is_empty() {
            println!("  Steps: none found");
            println!();
            continue;
        }

        println!("  Steps:");

        for step in &job.steps {
            let marker = marker_for_conclusion(step.conclusion.as_ref());

            println!(
                "    {} {}  [{} / {}]",
                marker,
                step.name,
                format!("{:?}", step.status).to_lowercase(),
                format_conclusion(step.conclusion.as_ref())
            );
        }

        println!("Failed jobs logs:");
        println!("-----------------");
        println!("{:#?}", run.failed_jobs_logs);
        println!()
    }
}

fn format_conclusion(conclusion: Option<&models::workflows::Conclusion>) -> String {
    conclusion
        .map(|conclusion| format!("{conclusion:?}").to_lowercase())
        .unwrap_or_else(|| "unknown".to_string())
}

fn marker_for_conclusion(conclusion: Option<&models::workflows::Conclusion>) -> &'static str {
    match conclusion {
        Some(models::workflows::Conclusion::Success) => "✅",
        Some(models::workflows::Conclusion::Failure) => "❌",
        Some(models::workflows::Conclusion::Skipped) => "⏭️",
        Some(models::workflows::Conclusion::Cancelled) => "🚫",
        Some(models::workflows::Conclusion::TimedOut) => "⏱️",
        Some(models::workflows::Conclusion::ActionRequired) => "⚠️",
        Some(models::workflows::Conclusion::Neutral) => "➖",
        Some(_) => "➖",
        None => "•",
    }
}
