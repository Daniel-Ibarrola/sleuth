# Requirements: Phase 1 — Fetch a GitHub Actions run

## Phase

Phase 1 — Fetch a GitHub Actions run.

## Goal

Add the first real GitHub integration so a user can inspect a GitHub Actions workflow run from the CLI without invoking the LLM.

The command:

```shell
bash sleuth fetch run <RUN_ID> --repo owner/name
```

prints a human-readable summary of the workflow run, its jobs, and the failing job log text.

## Scope

In scope:

1. Add GitHub API support using `octocrab`.
2. Use `reqwest` where needed for log download behavior.
3. Authenticate anonymously by default.
4. Use `GITHUB_TOKEN` when present.
5. Implement a GitHub-facing function to fetch workflow run metadata, jobs, and failing jobs.
6. Implement a GitHub-facing function to fetch raw job logs.
7. Implement `sleuth fetch run <RUN_ID> --repo owner/name`.
8. Print human-readable output.
9. Cover core behavior with automated tests that do not require live GitHub access.
10. Support manual validation against a real public failing workflow run.

Out of scope:

1. LLM analysis.
2. Root-cause reporting.
3. JSON output for `fetch`.
4. SQLite persistence.
5. Config-file default repo behavior.
6. Fixture repository creation.
7. CI workflow setup.
8. Support for CI providers other than GitHub Actions.

## User-facing behavior

The user can run:

```shell
bash sleuth fetch run 12345 --repo owner/name
```


The command should print:

1. The repository.
2. The run ID.
3. Workflow metadata where available.
4. Run status and conclusion.
5. A list of jobs with names, statuses, and conclusions.
6. The raw log text for failing job or jobs.

If no jobs failed, the command should clearly say that no failing jobs were found.

If the run or repository cannot be fetched, the command should fail with a clear error message.

## CLI decisions

1. `sleuth fetch run` is a supported user-facing v1 command.
2. `--repo owner/name` is required for this phase.
3. Human-readable output is the only required output format for this phase.
4. JSON output is deferred to a later phase.

## Authentication decisions

1. Public repositories should work without credentials.
2. If `GITHUB_TOKEN` is set, it should be used automatically.
3. Missing `GITHUB_TOKEN` is not an error for public repositories.
4. Authentication and authorization failures should produce actionable errors.

## Testing decisions

1. Automated tests should avoid live GitHub calls.
2. GitHub API access should be behind a mockable boundary.
3. A manual validation step should cover a real public failing GitHub Actions run.
4. Tests should focus on behavior and output, not exact third-party API internals.

## Context

This phase supports the larger goal of diagnosing CI/CD failures by establishing the project’s ability to fetch GitHub Actions data directly. Later phases will reuse this functionality for LLM-assisted analysis.