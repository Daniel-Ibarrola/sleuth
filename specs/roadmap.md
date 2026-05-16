# Roadmap

Each phase is small enough to finish in one or two sittings and ends with something a user (or developer) can do that they couldn't before. Phases describe user-visible functionality and outcomes; the specific modules, types, and functions used to deliver them are decided at implementation time.

Each phase carries a status: `todo`, `in progress`, or `completed`. Update it as work proceeds — a phase is only `completed` once its exit criteria are met.

## Phase 0 — Scaffold

**Status:** completed

- Rename the crate to `sleuth` in `Cargo.toml`, set `[[bin]] name = "sleuth"`.
- Add baseline deps: `clap`, `tokio`, `anyhow`, `tracing`, `tracing-subscriber`, `serde`, `serde_json`.
- `sleuth --help` works and prints the planned subcommands.
- `RUST_LOG=info sleuth ...` produces tracing output.

Exit criteria: `cargo run -- analyze run 1` prints "not implemented yet" and exits cleanly.

## Phase 1 — Fetch a GitHub Actions run

**Status:** todo

- Add `octocrab` and `reqwest`.
- Implement `github::fetch_run(repo, run_id)` returning workflow run metadata, the list of jobs, and the failing job(s).
- Implement `github::fetch_job_logs(repo, job_id)` returning the raw log text.
- Anonymous for public repos; pick up `GITHUB_TOKEN` if present.
- `sleuth fetch run <ID> --repo owner/name` dumps a summary of jobs and the failing log to stdout.

Exit criteria: against a real public failing run, the tool prints job names, statuses, and the failing log.

## Phase 2 — Continuous integration

**Status:** todo

Before adding more features, every pull request should automatically run formatting, linting, and tests, so changes can't silently rot the codebase.

A GitHub Actions workflow runs on every PR and on pushes to `main`, checking formatting (`cargo fmt`), lints (`cargo clippy`, warnings treated as errors), and the test suite (`cargo test`). The workflow's status is required to merge.

Exit criteria: opening a PR with a formatting error, a clippy warning, or a failing test gets a red check.

## Phase 3 — A public test-fixture repo

**Status:** todo

To develop and demo sleuth we need reliable failing GitHub Actions runs. We set up a separate public repo — **`Daniel-Ibarola/sleuth-fixtures`**, a small Python project — whose CI is deliberately broken in different recognizable ways, so there are always fresh failures to point sleuth at.

The fixture repo should cover, at minimum:

- Dependency install failure (missing or incompatible package).
- Service container not ready (e.g. Postgres not healthy before tests start).
- Flaky test.
- Test assertion failure.
- Timeout or hung step.
- Misconfigured environment variable.

Each failure mode is triggerable on demand (e.g. by re-running the workflow) so a fresh run ID is always available. The repo URL is recorded in this project's README as the canonical end-to-end target.

Exit criteria: from this point on, `sleuth ... --repo Daniel-Ibarola/sleuth-fixtures` is the default thing to point the tool at during development.

## Phase 4 — Basic analysis

**Status:** todo

The user can run `sleuth analyze run <ID> --repo owner/name` and get a written report containing the failing job, a likely root cause, supporting evidence from the logs, and a short list of suggested fixes.

This phase introduces the LLM in its simplest form: the model sees the logs we already know how to fetch and produces a free-form text answer. No back-and-forth, no investigation — just the simplest pipeline that produces a useful report.

Exit criteria: against several different failure modes in the fixture repo, the reports name the correct failing job and give a plausible cause and fixes.

## Phase 5 — Self-directed investigation

**Status:** todo

The user runs the same command and gets noticeably better reports, because the assistant now decides what to look at. It can pull the list of jobs, fetch a specific job's log on demand, look up a run's metadata, and check recent runs of the same workflow to see what changed since the last green.

The user-visible change: reports become more focused and start citing things only an investigator could have found — e.g. "this workflow was green yesterday; the only meaningful change in the failing run was X."

Exit criteria: on the fixture repo's "what changed since green?" style failures, the report identifies the relevant change. Cost per analysis stays under the budget set in `mission.md`.

## Phase 6 — Machine-readable output

**Status:** todo

`sleuth analyze` gains a `--json` flag that emits the report as a stable JSON document for other tools to consume (piped into `jq`, posted to chat by a script, etc.). The pretty terminal output remains the default.

Exit criteria: `sleuth analyze run <ID> --json | jq .likely_cause` returns a sensible string for a known failing run.

## Phase 7 — History

**Status:** todo

Past analyses are remembered locally and can be revisited without re-running the LLM.

- `sleuth history` lists recent analyses, most recent first.
- `sleuth show <analysis_id>` re-renders an old report.
- Every `analyze` invocation records a row automatically.

Exit criteria: after analyzing two runs, `sleuth history` shows both, and `sleuth show <id>` reproduces a past report identically.

## Phase 8 — Config defaults

**Status:** todo

A user who works on the same repo most of the time shouldn't need to type `--repo` every time. An optional `~/.config/sleuth/config.toml` provides defaults (default repo, default model), with CLI flags and env vars taking precedence.

Exit criteria: with `default_repo` set in the config file, `sleuth analyze run <ID>` works without `--repo`.

## Later (post-v1, do not start before v1 is done)

- Second LLM provider (OpenAI, local via Ollama).
- More investigation tools: PR diff for the failing run, commit author, retry history.
- `sleuth watch` daemon mode against a repo's failed runs.
- A small TUI for browsing history.
- Support for other CI providers (GitLab, CircleCI, Jenkins).
