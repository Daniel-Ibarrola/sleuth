# Mission

**sleuth** helps engineers diagnose CI/CD failures faster by analyzing GitHub Actions workflow runs and producing a focused root-cause report.

## What it does

Given a GitHub Actions run ID, sleuth fetches the run's metadata and logs, then uses an LLM agent (with tool use) to produce a short report:

- The failing job/step
- The likely root cause, in one sentence
- The evidence from the logs supporting that cause
- Two or three concrete suggested fixes

It runs as a CLI:

```
sleuth analyze run <RUN_ID> [--repo owner/name] [--json]
```

## Who it's for

The primary user is the author — a Rust learner exploring AI agents. The secondary user is any engineer who has stared at a wall of CI logs and wished a teammate would skim it for them.

## Why it exists

1. **Learning Rust** — exercise async, error handling, traits, CLI design, persistence, HTTP, and project structure on a real domain.
2. **Learning AI agents** — build an agent loop with tool use against the Claude API directly, no high-level framework.
3. **Maybe useful** — if the diagnoses turn out to be good, it's a real tool.

## v1 scope

In scope:

- GitHub Actions only.
- Public repos with no token; private repos with `GITHUB_TOKEN`.
- One LLM provider: Claude (Haiku tier for cost).
- Agent loop with a small set of tools (fetch run, list jobs, fetch log, fetch recent runs of the same workflow).
- Pretty terminal output and `--json`.
- Local SQLite store of past analyses.

Explicitly out of v1:

- Other CI providers (GitLab, CircleCI, Jenkins).
- A daemon, web UI, or Slack/PR integrations.
- Auto-fixing or opening PRs.
- A second LLM provider implementation (kept behind a trait so it's easy to add later).
- RAG over historical runs. The agent can request recent runs as a tool, but no embeddings.

## Success criteria for v1

- For a real failing GitHub Actions run, sleuth produces a report whose root cause matches what an experienced engineer would conclude after reading the logs.
- It does this in a single command, in under ~30 seconds, for under ~10 cents per analysis.
- The codebase is something the author can read in a year and still understand.
