# CLI reference (v1)

The full surface of the `sleuth` CLI for v1. Anything not listed here is out of scope until v1 ships.

## Synopsis

```
sleuth <COMMAND> [OPTIONS]
```

## Commands

### `sleuth analyze run <RUN_ID>`

The main command. Fetches a GitHub Actions workflow run and produces a diagnostic report: failing job, likely root cause, supporting evidence from the logs, and a short list of suggested fixes.

Flags:

- `--repo OWNER/NAME` — the GitHub repo the run belongs to. Required unless `default_repo` is set in config.
- `--json` — emit the report as JSON instead of formatted terminal output.

Behavior:

- Authenticates with GitHub anonymously, or with `GITHUB_TOKEN` if set.
- Uses Claude (Haiku tier) for analysis. Requires `ANTHROPIC_API_KEY`.
- Records the analysis to the local history store.

### `sleuth fetch run <RUN_ID>`

Lower-level inspection command. Fetches a run and prints its metadata, job list, and logs to stdout without involving the LLM. Useful for debugging and for seeing what sleuth sees.

Flags:

- `--repo OWNER/NAME` — required unless set in config.

### `sleuth history`

Lists recent analyses recorded locally, most recent first. No GitHub or LLM calls.

Flags:

- `--repo OWNER/NAME` — filter to one repo.
- `--limit N` — maximum number of rows (default 20).

### `sleuth show <ANALYSIS_ID>`

Re-renders a past analysis from the local store. No GitHub or LLM calls.

Flags:

- `--json` — emit as JSON.

## Global flags

- `--help`, `-h` — usage.
- `--version`, `-V` — print version.

## Environment variables

- `ANTHROPIC_API_KEY` — required for any command that runs the LLM (`analyze`).
- `GITHUB_TOKEN` — optional; needed for private repos and to lift API rate limits.
- `RUST_LOG` — tracing filter, e.g. `RUST_LOG=info` or `RUST_LOG=sleuth=debug`.

## Configuration file

Optional file at `~/.config/sleuth/config.toml` (or the platform equivalent). Recognized keys for v1:

- `default_repo` — used when `--repo` is not given.
- `model` — overrides the default Claude model name.

Precedence (highest wins): CLI flag → env var → config file → built-in default.

## Exit codes

- `0` — success.
- `1` — generic error (network, GitHub API, LLM error, etc.).
- `2` — invalid arguments or usage.

## Examples

```
# Analyze a specific failing run
sleuth analyze run 12345 --repo myorg/myrepo

# Same, but produce JSON for piping
sleuth analyze run 12345 --repo myorg/myrepo --json | jq .likely_cause

# Inspect a run without involving the LLM
sleuth fetch run 12345 --repo myorg/myrepo

# Browse and re-display past analyses
sleuth history --limit 5
sleuth show 42
```
