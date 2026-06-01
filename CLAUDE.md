# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

**sleuth** — a CLI tool that diagnoses GitHub Actions failures by fetching run metadata and logs, then running an LLM agent to produce a root-cause report. The project's dual purpose is (1) learning Rust and (2) learning how to build an AI agent loop from scratch.

## Commands

```bash
cargo build
cargo run -- fetch run <RUN_ID> --repo owner/name
cargo test
cargo fmt
cargo clippy -- -D warnings     # warnings are errors in CI
RUST_LOG=debug cargo run -- ...  # enable tracing output
```

## Required environment variables

- `ANTHROPIC_API_KEY` — required for any `analyze` command (LLM calls).
- `GITHUB_TOKEN` — optional; needed for private repos and to avoid API rate limits. Also required to fetch job logs (the per-job log endpoint requires auth).

The `.env.sh` file in the repo root can `export` these for local development — source it before running, don't commit credentials.

## Architecture

The codebase is early-stage. Current modules:

- `main.rs` — clap CLI definition (`Cli`, `Command`, `AnalyzeTarget`, `FetchTarget`), wires up tracing and dispatches subcommands.
- `github.rs` — octocrab wrappers. `get_run_data()` fetches run metadata + jobs + logs for failing jobs. `print_run()` formats to stdout. Log fetching hits the GitHub REST API directly via `reqwest` because octocrab doesn't expose the per-job log endpoint.
- `errors.rs` — `SleuthError` enum with `From` impls for `octocrab::Error` and `reqwest::Error`.

Planned modules (not yet created — add them as roadmap phases require):

```
src/
  config.rs        # env + toml loading, replaces hardcoded default_repo in main.rs
  github/          # expand github.rs into a directory when it grows
  llm/             # reqwest-based Claude Messages API client + LlmClient trait
  agent/           # agent loop, tool registry, tool implementations
  report/          # report types, pretty + JSON renderers
  store/           # rusqlite schema and queries
```

## Key design constraints

- **No agent framework** — the Claude API is called directly via `reqwest`. The point is to learn the messages/tool-use/stop-reason protocol by hand.
- **No SDK** — `reqwest` + `serde_json` only for LLM calls.
- **`anyhow` for application errors, `SleuthError` for typed errors** that need `From` conversions between library error types.
- **Default model: `claude-haiku-4-5`**. Cost target ≤ $0.10 per analysis.
- **Persistence**: rusqlite with `bundled` feature, DB at `~/.local/share/sleuth/sleuth.db` (use the `directories` crate). Not yet implemented.
- **Auth**: anonymous octocrab client unless `GITHUB_TOKEN` is set; same pattern for raw reqwest calls.

## Specs and roadmap

The `specs/` directory is the project constitution — read it before making structural decisions:

- `specs/mission.md` — goals, v1 scope, success criteria.
- `specs/tech-stack.md` — authoritative list of allowed crates and what's explicitly deferred.
- `specs/roadmap.md` — phased delivery plan. Each phase has a `Status` field; update it as work proceeds.
- `specs/cli.md` — full v1 CLI surface (subcommands, flags, env vars, exit codes).

## Test fixture repo

`Daniel-Ibarrola/sleuth-fixtures` — a public Python repo with deliberately broken CI workflows covering common failure modes (dependency install failure, service container not ready, flaky test, etc.). Use it as the default `--repo` target during development.
