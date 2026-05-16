# Tech stack

Conservative picks. Default to widely-used crates with good docs. Avoid frameworks that hide the parts worth learning.

## Language and runtime

- **Rust**, edition 2024.
- **tokio** — async runtime, needed for `reqwest` and `octocrab`.

## CLI and ergonomics

- **clap** with derive macros — argument parsing.
- **anyhow** — application-level errors. No `thiserror` until something needs to be a library.
- **tracing** + **tracing-subscriber** — structured logging, controlled by `RUST_LOG`.
- **owo-colors** — terminal coloring. Lightweight, no global state.

## HTTP and JSON

- **reqwest** with `json` and `rustls-tls` features — HTTP client for the Claude API.
- **serde** + **serde_json** — used everywhere: API payloads, tool args, config, persisted records.

## GitHub

- **octocrab** — typed GitHub API client. Handles auth, pagination, and log download.
- Auth: anonymous for public repos; `GITHUB_TOKEN` env var for private repos or to lift rate limits.

## LLM

- **Provider**: Anthropic Claude. Default model: `claude-haiku-4-5`. Cost target ≤ $0.10 per analysis.
- **No SDK** — call the Messages API directly with `reqwest`. The point is to learn the protocol: messages, tool use, stop reasons, the agent loop.
- A small `LlmClient` trait wraps the calls so a second provider could be added later without rewriting the agent.
- Auth: `ANTHROPIC_API_KEY` env var.

## Persistence

- **rusqlite** with the `bundled` feature — embedded SQLite, simplest possible setup.
- One database file under the OS data dir (resolved via the `directories` crate), e.g. `~/.local/share/sleuth/sleuth.db` on Linux.
- Migrations: hand-rolled `CREATE TABLE IF NOT EXISTS` for v1. Add `refinery` or `rusqlite_migration` if/when the schema starts changing.
- `sqlx` is a tempting upgrade for compile-time checked queries, but heavier and async. Defer.

## Config

- Env vars for secrets: `ANTHROPIC_API_KEY`, `GITHUB_TOKEN`.
- Optional `~/.config/sleuth/config.toml` for defaults (default repo, model name). Parsed with **toml** + serde derive.
- The **directories** crate to find OS-appropriate config and data paths.

## Project layout

A target, not a constraint — phases of the roadmap will introduce modules as needed.

```
sleuth/
  Cargo.toml            # crate renamed to "sleuth"; [[bin]] name = "sleuth"
  specs/                # constitution lives here
  src/
    main.rs             # entry, wires up tracing + clap
    cli.rs              # clap definitions
    config.rs           # env + toml loading
    github/             # octocrab wrappers, log fetching
    llm/                # Claude client, message + tool-use protocol
    agent/              # agent loop, tool registry, tool implementations
    report/             # report types, pretty + json renderers
    store/              # sqlite schema, queries
```

## What we are deliberately NOT using (yet)

- **rig**, **langchain-rust**, or any agent framework — hides the lesson.
- **sqlx** — overkill for a single-user local DB.
- **ratatui** or a TUI — overkill for v1, colored output is enough.
- **thiserror** — application code, `anyhow` is enough.
- A second LLM provider — kept behind a trait but not implemented in v1.
