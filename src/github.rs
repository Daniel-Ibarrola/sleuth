use crate::errors::SleuthError;
use octocrab::{Octocrab, models, params::workflows::Filter};
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;

pub(crate) const GITHUB_API_BASE: &str = "https://api.github.com";

#[derive(Debug)]
pub struct Step {
    pub name: String,
    pub status: models::workflows::Status,
    pub conclusion: Option<models::workflows::Conclusion>,
}

#[derive(Debug)]
pub struct WorkflowJob {
    pub id: u64,
    pub status: models::workflows::Status,
    pub conclusion: Option<models::workflows::Conclusion>,
    pub name: String,
    pub steps: Vec<Step>,
}

#[derive(Debug)]
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
    match std::env::var("GITHUB_TOKEN") {
        Ok(token) if !token.trim().is_empty() => {
            tracing::info!("GitHub: authenticated with GITHUB_TOKEN");
            Octocrab::builder().personal_token(token).build()
        }
        _ => {
            tracing::info!("GitHub: no GITHUB_TOKEN found, using anonymous access");
            Octocrab::builder().build()
        }
    }
}

async fn fetch_run(
    owner: &str,
    repo: &str,
    run_id: u64,
    octocrab: &Octocrab,
) -> Result<WorkflowRun, octocrab::Error> {
    tracing::info!(owner, repo, run_id, "fetching workflow run");
    let workflows = octocrab.workflows(owner, repo);
    let run = workflows.get(run_id.into()).await?;
    tracing::debug!(name = run.name, status = run.status, "run metadata received");

    tracing::info!(run_id, "fetching jobs");
    let jobs_page = workflows
        .list_jobs(run_id.into())
        .per_page(100)
        .filter(Filter::All)
        .send()
        .await?;

    let jobs: Vec<WorkflowJob> = jobs_page
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

    let job_count = jobs.len();
    tracing::info!(job_count, "jobs received");

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

async fn fetch_job_logs(
    owner: &str,
    repo: &str,
    job_id: u64,
    requests_client: &reqwest::Client,
    base_url: &str,
) -> Result<String, SleuthError> {
    // TODO: fall back to GET /actions/runs/{run_id}/logs (zip) when unauthenticated
    let authenticated = std::env::var("GITHUB_TOKEN")
        .map(|t| !t.trim().is_empty())
        .unwrap_or(false);
    tracing::info!(job_id, authenticated, "fetching job logs");

    let url = format!("{base_url}/repos/{owner}/{repo}/actions/jobs/{job_id}/logs");
    let mut request = requests_client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2026-03-10")
        .header("User-Agent", "sleuth");

    if authenticated {
        let token = std::env::var("GITHUB_TOKEN").unwrap();
        request = request.bearer_auth(token);
    }

    let response = request.send().await?.error_for_status()?;
    let log = response.text().await?;
    tracing::debug!(job_id, bytes = log.len(), "job log received");
    Ok(log)
}

pub async fn get_run_data(
    repo: &str,
    run_id: u64,
    github_client: &Octocrab,
    requests_client: &reqwest::Client,
    github_api_base: &str,
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

    let mut run = fetch_run(owner, repo, run_id, github_client).await?;
    let mut logs = HashMap::<u64, String>::new();
    let failed_jobs: Vec<_> = run
        .jobs
        .iter()
        .filter(|j| {
            j.conclusion == Some(models::workflows::Conclusion::Failure)
                || j.conclusion == Some(models::workflows::Conclusion::TimedOut)
        })
        .collect();

    tracing::info!(count = failed_jobs.len(), "fetching logs for failed jobs");
    for job in failed_jobs {
        let log = fetch_job_logs(owner, repo, job.id, requests_client, github_api_base).await?;
        logs.insert(job.id, log);
    }
    run.failed_jobs_logs = logs;

    Ok(run)
}

pub fn format_run(run: &WorkflowRun) -> String {
    let mut out = String::new();

    writeln!(out).unwrap();
    writeln!(out, "GitHub Actions Run").unwrap();
    writeln!(out, "==================").unwrap();
    writeln!(out, "Run ID:      {}", run.id).unwrap();
    writeln!(out, "Name:        {}", run.name).unwrap();
    writeln!(out, "Workflow ID: {}", run.workflow_id).unwrap();
    writeln!(out, "Status:      {}", run.status.to_lowercase()).unwrap();
    writeln!(
        out,
        "Conclusion:  {}",
        run.conclusion.as_deref().unwrap_or("unknown")
    )
    .unwrap();
    writeln!(out).unwrap();

    if run.jobs.is_empty() {
        writeln!(out, "Jobs: none found").unwrap();
        return out;
    }

    writeln!(out, "Jobs").unwrap();
    writeln!(out, "----").unwrap();

    for job in &run.jobs {
        let marker = marker_for_conclusion(job.conclusion.as_ref());

        writeln!(
            out,
            "{} {}  [{} / {}]",
            marker,
            job.name,
            format!("{:?}", job.status).to_lowercase(),
            format_conclusion(job.conclusion.as_ref()),
        )
        .unwrap();
        writeln!(out, "  Job ID: {}", job.id).unwrap();

        if job.steps.is_empty() {
            writeln!(out, "  Steps: none found").unwrap();
            writeln!(out).unwrap();
            continue;
        }

        writeln!(out, "  Steps:").unwrap();

        for step in &job.steps {
            let marker = marker_for_conclusion(step.conclusion.as_ref());

            writeln!(
                out,
                "    {} {}  [{} / {}]",
                marker,
                step.name,
                format!("{:?}", step.status).to_lowercase(),
                format_conclusion(step.conclusion.as_ref()),
            )
            .unwrap();
        }

        writeln!(out, "Failed jobs logs:").unwrap();
        writeln!(out, "-----------------").unwrap();
        // TODO: format logs and don't print map
        writeln!(out, "{:#?}", run.failed_jobs_logs).unwrap();
        writeln!(out).unwrap();
    }

    out
}

pub fn print_run(run: WorkflowRun) {
    print!("{}", format_run(&run));
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

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // ── JSON fixture helpers ──────────────────────────────────────────────────

    fn run_json(run_id: u64, conclusion: Option<&str>) -> serde_json::Value {
        serde_json::json!({
            "id": run_id,
            "workflow_id": 10,
            "node_id": "WFR_abc",
            "name": "CI",
            "head_branch": "main",
            "head_sha": "abc123",
            "run_number": 1,
            "event": "push",
            "status": "completed",
            "conclusion": conclusion,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "url": format!("http://localhost/repos/owner/repo/actions/runs/{run_id}"),
            "html_url": format!("http://localhost/owner/repo/actions/runs/{run_id}"),
            "jobs_url": format!("http://localhost/repos/owner/repo/actions/runs/{run_id}/jobs"),
            "logs_url": format!("http://localhost/repos/owner/repo/actions/runs/{run_id}/logs"),
            "check_suite_url": "http://localhost/repos/owner/repo/check-suites/1",
            "artifacts_url": format!("http://localhost/repos/owner/repo/actions/runs/{run_id}/artifacts"),
            "cancel_url": format!("http://localhost/repos/owner/repo/actions/runs/{run_id}/cancel"),
            "rerun_url": format!("http://localhost/repos/owner/repo/actions/runs/{run_id}/rerun"),
            "workflow_url": "http://localhost/repos/owner/repo/actions/workflows/10",
            "head_commit": {
                "id": "abc123",
                "tree_id": "def456",
                "message": "Test commit",
                "timestamp": "2024-01-01T00:00:00Z",
                "author": {"name": "User"},
                "committer": {"name": "User"}
            },
            "repository": {
                "id": 1,
                "name": "repo",
                "url": "http://localhost/repos/owner/repo"
            }
        })
    }

    fn job_json(
        job_id: u64,
        run_id: u64,
        name: &str,
        conclusion: Option<&str>,
        steps: Vec<serde_json::Value>,
    ) -> serde_json::Value {
        serde_json::json!({
            "id": job_id,
            "run_id": run_id,
            "workflow_name": "CI",
            "head_branch": "main",
            "run_url": format!("http://localhost/repos/owner/repo/actions/runs/{run_id}"),
            "run_attempt": 1,
            "node_id": "J_abc",
            "head_sha": "abc123",
            "url": format!("http://localhost/repos/owner/repo/actions/jobs/{job_id}"),
            "html_url": format!("http://localhost/owner/repo/actions/jobs/{job_id}"),
            "status": "completed",
            "conclusion": conclusion,
            "created_at": "2024-01-01T00:00:00Z",
            "started_at": "2024-01-01T00:00:10Z",
            "completed_at": "2024-01-01T00:01:00Z",
            "name": name,
            "steps": steps,
            "check_run_url": format!("http://localhost/repos/owner/repo/check-runs/{job_id}"),
            "labels": []
        })
    }

    fn jobs_page_json(jobs: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::json!({ "total_count": jobs.len(), "jobs": jobs })
    }

    async fn mock_run(server: &MockServer, run_id: u64, conclusion: Option<&str>) {
        Mock::given(method("GET"))
            .and(path(format!("/repos/owner/repo/actions/runs/{run_id}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(run_json(run_id, conclusion)))
            .mount(server)
            .await;
    }

    async fn mock_jobs(server: &MockServer, run_id: u64, jobs: Vec<serde_json::Value>) {
        Mock::given(method("GET"))
            .and(path(format!(
                "/repos/owner/repo/actions/runs/{run_id}/jobs"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(jobs_page_json(jobs)))
            .mount(server)
            .await;
    }

    async fn mock_job_logs(server: &MockServer, job_id: u64, log_text: &str) {
        Mock::given(method("GET"))
            .and(path(format!(
                "/repos/owner/repo/actions/jobs/{job_id}/logs"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_string(log_text.to_owned()))
            .mount(server)
            .await;
    }

    fn install_tls() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }

    fn octocrab_for(server: &MockServer) -> Octocrab {
        install_tls();
        Octocrab::builder()
            .base_uri(server.uri())
            .unwrap()
            .build()
            .unwrap()
    }

    fn plain_octocrab() -> Octocrab {
        install_tls();
        Octocrab::builder().build().unwrap()
    }

    // ── format_run unit tests ─────────────────────────────────────────────────

    fn run_no_jobs() -> WorkflowRun {
        WorkflowRun {
            id: 1001,
            name: "CI".to_string(),
            workflow_id: 456,
            status: "completed".to_string(),
            conclusion: Some("success".to_string()),
            jobs: vec![],
            failed_jobs_logs: HashMap::new(),
        }
    }

    fn run_with_jobs() -> WorkflowRun {
        WorkflowRun {
            id: 2002,
            name: "Test Suite".to_string(),
            workflow_id: 789,
            status: "completed".to_string(),
            conclusion: Some("failure".to_string()),
            jobs: vec![
                WorkflowJob {
                    id: 100,
                    name: "build".to_string(),
                    status: models::workflows::Status::Completed,
                    conclusion: Some(models::workflows::Conclusion::Success),
                    steps: vec![],
                },
                WorkflowJob {
                    id: 200,
                    name: "test".to_string(),
                    status: models::workflows::Status::Completed,
                    conclusion: Some(models::workflows::Conclusion::Failure),
                    steps: vec![Step {
                        name: "Run tests".to_string(),
                        status: models::workflows::Status::Completed,
                        conclusion: Some(models::workflows::Conclusion::Failure),
                    }],
                },
            ],
            failed_jobs_logs: HashMap::from([(200, "error: assertion failed\n".to_string())]),
        }
    }

    #[test]
    fn format_run_header_fields() {
        let out = format_run(&run_no_jobs());
        assert!(out.contains("GitHub Actions Run"));
        assert!(out.contains("Run ID:      1001"));
        assert!(out.contains("Name:        CI"));
        assert!(out.contains("Workflow ID: 456"));
        assert!(out.contains("Status:      completed"));
        assert!(out.contains("Conclusion:  success"));
        assert!(out.contains("Jobs: none found"));
    }

    #[test]
    fn format_run_missing_conclusion_shows_unknown() {
        let run = WorkflowRun {
            id: 1,
            name: "CI".to_string(),
            workflow_id: 1,
            status: "in_progress".to_string(),
            conclusion: None,
            jobs: vec![],
            failed_jobs_logs: HashMap::new(),
        };
        assert!(format_run(&run).contains("Conclusion:  unknown"));
    }

    #[test]
    fn format_run_job_markers_and_steps() {
        let out = format_run(&run_with_jobs());
        assert!(out.contains("✅ build"));
        assert!(out.contains("❌ test"));
        assert!(out.contains("Run tests"));
        assert!(out.contains("error: assertion failed"));
    }

    // ── get_run_data repo validation tests ────────────────────────────────────

    #[tokio::test]
    async fn get_run_data_rejects_repo_without_slash() {
        let err = get_run_data(
            "noslash",
            1,
            &plain_octocrab(),
            &reqwest::Client::new(),
            "http://localhost",
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("Invalid repository format"));
    }

    #[tokio::test]
    async fn get_run_data_rejects_empty_owner() {
        let err = get_run_data(
            "/repo",
            1,
            &plain_octocrab(),
            &reqwest::Client::new(),
            "http://localhost",
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("Invalid repository format"));
    }

    #[tokio::test]
    async fn get_run_data_rejects_multiple_slashes() {
        let err = get_run_data(
            "owner/repo/extra",
            1,
            &plain_octocrab(),
            &reqwest::Client::new(),
            "http://localhost",
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("Invalid repository format"));
    }

    // ── wiremock integration tests ────────────────────────────────────────────

    #[tokio::test]
    async fn get_run_data_returns_jobs_for_successful_run() {
        let server = MockServer::start().await;

        mock_run(&server, 123, Some("success")).await;
        mock_jobs(
            &server,
            123,
            vec![job_json(10, 123, "build", Some("success"), vec![])],
        )
        .await;

        let run = get_run_data(
            "owner/repo",
            123,
            &octocrab_for(&server),
            &reqwest::Client::new(),
            &server.uri(),
        )
        .await
        .unwrap();

        assert_eq!(run.id, 123);
        assert_eq!(run.name, "CI");
        assert_eq!(run.jobs.len(), 1);
        assert_eq!(run.jobs[0].name, "build");
        assert!(run.failed_jobs_logs.is_empty());
    }

    #[tokio::test]
    async fn get_run_data_fetches_logs_for_failed_jobs() {
        let server = MockServer::start().await;

        mock_run(&server, 456, Some("failure")).await;
        mock_jobs(
            &server,
            456,
            vec![job_json(99, 456, "test", Some("failure"), vec![])],
        )
        .await;
        mock_job_logs(&server, 99, "Error: assertion failed at line 42").await;

        let run = get_run_data(
            "owner/repo",
            456,
            &octocrab_for(&server),
            &reqwest::Client::new(),
            &server.uri(),
        )
        .await
        .unwrap();

        assert_eq!(run.jobs.len(), 1);
        assert_eq!(
            run.failed_jobs_logs.get(&99).unwrap(),
            "Error: assertion failed at line 42"
        );
    }

    #[tokio::test]
    async fn get_run_data_fetches_logs_for_timed_out_jobs() {
        let server = MockServer::start().await;

        mock_run(&server, 789, Some("timed_out")).await;
        mock_jobs(
            &server,
            789,
            vec![job_json(55, 789, "slow-test", Some("timed_out"), vec![])],
        )
        .await;
        mock_job_logs(
            &server,
            55,
            "##[error]The job running on runner ... exceeded the maximum execution time",
        )
        .await;

        let run = get_run_data(
            "owner/repo",
            789,
            &octocrab_for(&server),
            &reqwest::Client::new(),
            &server.uri(),
        )
        .await
        .unwrap();

        assert!(run.failed_jobs_logs.contains_key(&55));
    }
}
