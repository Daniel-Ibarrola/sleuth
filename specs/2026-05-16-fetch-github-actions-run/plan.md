# Plan: Phase 1 — Fetch a GitHub Actions run

## 1. Define the GitHub fetch contract
status: done

1. Write tests for parsing and validating repository identifiers in `owner/name` form.
2. Write tests for the expected run summary shape returned by the GitHub layer.
3. Write tests for identifying failing jobs from a fetched workflow run.
4. Define the data structures needed to represent:
    - workflow run metadata
    - jobs
    - failing jobs
    - raw job logs
5. Keep the public GitHub-facing API small enough for later reuse by `analyze`.

## 2. Add GitHub client dependencies and wiring
status: done

1. Add `octocrab` for GitHub API access.
2. Add or enable `reqwest` usage for downloading logs when needed.
3. At command startup, check for `GITHUB_TOKEN` before any network call. If absent, exit immediately with a message such as: `error: GITHUB_TOKEN is required. The GitHub log endpoint requires authentication even for public repositories. Set GITHUB_TOKEN to a token with at least read access.`
4. Create GitHub client construction that uses `GITHUB_TOKEN` for all API calls.
5. Ensure GitHub-related errors include enough context to diagnose bad repo names, missing runs, auth failures, and rate limits.

## 3. Implement workflow run fetching
status: done

1. Write unit tests for the run-fetching flow using mocked or fake GitHub client behavior.
2. Implement `fetch_run(repo, run_id)` to return:
    - workflow run metadata
    - the list of jobs
    - the failing job or jobs
3. Ensure the implementation handles:
    - successful runs with no failing jobs
    - failed runs with one failing job
    - failed runs with multiple failing jobs
    - missing or inaccessible runs
4. Add focused error messages for API failures.

## 4. Implement job log fetching
status: done

1. Write tests for fetching a job log through a mockable boundary.
2. Implement `fetch_job_logs(repo, job_id)` returning raw log text.
3. Ensure log fetching works for failed jobs selected from `fetch_run`.
4. Handle unavailable, empty, or expired logs gracefully.
5. Avoid truncating logs in the GitHub layer; any output truncation should be a presentation decision.

## 5. Implement the user-facing fetch command
status: done

1. Write CLI-level tests or integration tests for `sleuth fetch run <RUN_ID> --repo owner/name`.
2. Replace the placeholder behavior for `fetch run`.
3. Print human-readable output containing:
    - repository
    - run ID
    - workflow name if available
    - run status and conclusion
    - job names, statuses, and conclusions
    - failing job log text
4. Keep JSON output out of scope for this phase.
5. Ensure this remains a supported user-facing command, not a temporary debug-only command.

## 6. Add manual live validation path

1. Document how to run the command against a real public failing GitHub Actions run.
2. Keep automated tests isolated from live GitHub by default.
3. Allow a manual validation command that can use anonymous access or `GITHUB_TOKEN`.
4. Record any known-good public failing run used during development if one is available.

## 7. Polish and maintainability

1. Run formatting.
2. Run clippy with warnings treated as errors.
3. Run the automated test suite.
4. Review user-facing error messages.
5. Confirm the code remains understandable and aligned with the project’s learning goals.

## 8. Final validation before completion

1. Run every validation item listed in `validation.md`.
2. Confirm the Phase 1 roadmap exit criteria are met.
3. Only after all validation succeeds, update the roadmap phase status to `completed`.