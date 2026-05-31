use crate::errors::SleuthError;
use octocrab::{models, params::workflows::Filter};

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
}

async fn fetch_run(owner: &str, repo: &str, run_id: u64) -> Result<WorkflowRun, octocrab::Error> {
    let octocrab = octocrab::instance();

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
    })
}

// Get run metadata and logs and print nice report
pub async fn get_run_data(repo: &str, run_id: u64) -> Result<WorkflowRun, SleuthError> {
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

    let run = fetch_run(owner, repo, run_id).await?;
    Ok(run)
}

pub fn print_run(run: WorkflowRun) {
    println!();
    println!("GitHub Actions Run");
    println!("==================");
    println!("Run ID:      {}", run.id);
    println!("Name:        {}", run.name);
    println!("Workflow ID: {}", run.workflow_id);
    println!(
        "Status:      {}",
        format!("{}", run.status).to_lowercase()
    );
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

        println!();
    }
}

fn format_conclusion(conclusion: Option<&octocrab::models::workflows::Conclusion>) -> String {
    conclusion
        .map(|conclusion| format!("{conclusion:?}").to_lowercase())
        .unwrap_or_else(|| "unknown".to_string())
}

fn marker_for_conclusion(
    conclusion: Option<&octocrab::models::workflows::Conclusion>,
) -> &'static str {
    match conclusion {
        Some(octocrab::models::workflows::Conclusion::Success) => "✅",
        Some(octocrab::models::workflows::Conclusion::Failure) => "❌",
        Some(octocrab::models::workflows::Conclusion::Skipped) => "⏭️",
        Some(octocrab::models::workflows::Conclusion::Cancelled) => "🚫",
        Some(octocrab::models::workflows::Conclusion::TimedOut) => "⏱️",
        Some(octocrab::models::workflows::Conclusion::ActionRequired) => "⚠️",
        Some(octocrab::models::workflows::Conclusion::Neutral) => "➖",
        Some(_) => "➖",
        None => "•",
    }
}

// ... existing code ...
