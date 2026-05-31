# Validation: Phase 1 — Fetch a GitHub Actions run

## Automated validation

Run:

```shell
cargo fmt --check 
cargo clippy -- -D warnings
cargo test
```

All three commands must pass before the phase can be marked completed.

## Unit and integration expectations

Automated tests should verify:

1. Repository strings in `owner/name` form are accepted.
2. Invalid repository strings produce clear errors.
3. GitHub client setup uses anonymous access when `GITHUB_TOKEN` is absent.
4. GitHub client setup uses token-based access when `GITHUB_TOKEN` is present.
5. Workflow run data can be converted into the project’s run summary type.
6. Failed jobs are identified correctly.
7. Multiple failed jobs are handled correctly.
8. Runs with no failed jobs are handled clearly.
9. Job log fetching returns raw log text through a mockable boundary.
10. `sleuth fetch run <RUN_ID> --repo owner/name` prints the expected human-readable sections.

## Manual validation

Use a real public GitHub repository with a failing GitHub Actions run.

Run:
```shell
cargo run -- fetch run <RUN_ID> --repo owner/name
```

Expected result:

1. The command exits successfully.
2. The output includes the repository name.
3. The output includes the run ID.
4. The output includes job names.
5. The output includes job statuses and conclusions.
6. The output identifies failing job or jobs.
7. The output includes the failing job log text.

## Optional authenticated manual validation

If a token is available, run:

```shell
GITHUB_TOKEN=<token> cargo run -- fetch run <RUN_ID> --repo owner/name
```


Expected result:

1. The command exits successfully.
2. The behavior matches anonymous validation for public repositories.
3. Any private repository validation succeeds only when the token has sufficient access.

## Error validation

Validate clear failures for:

1. Missing `--repo`.
2. Invalid repository format.
3. Unknown repository.
4. Unknown run ID.
5. Authentication or authorization failure.
6. Missing or unavailable logs.

## Completion criteria

This phase is complete only when:

1. All automated validation commands pass.
2. Manual validation succeeds against a real public failing GitHub Actions run.
3. `sleuth fetch run <RUN_ID> --repo owner/name` prints job names, statuses, and the failing log.
4. The roadmap status for Phase 1 can be updated to `completed`.