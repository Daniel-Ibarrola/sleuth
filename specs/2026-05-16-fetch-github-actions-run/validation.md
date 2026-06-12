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
3. GitHub client setup uses token-based access when `GITHUB_TOKEN` is present.
4. Workflow run data can be converted into the project’s run summary type.
5. Failed jobs are identified correctly.
6. Multiple failed jobs are handled correctly.
7. Runs with no failed jobs are handled clearly.
8. Job log fetching returns raw log text through a mockable boundary
9. `sleuth fetch run <RUN_ID> --repo owner/name` prints the expected human-readable sections.

## Manual validation

### Missing token (early-exit check)

Run without setting `GITHUB_TOKEN`:

```shell
cargo run -- fetch run <RUN_ID> --repo owner/name
```

Expected result:

1. The command exits immediately with a non-zero exit code.
2. The error message explains that `GITHUB_TOKEN` is required and how to set it.
3. No network calls are made.

### Authenticated fetch

Set `GITHUB_TOKEN` and point the command at a real public failing run:

```shell
GITHUB_TOKEN=<token> cargo run -- fetch run <RUN_ID> --repo owner/name
```

Expected result:

1. The command exits successfully.
2. The output includes the repository name.
3. The output includes the run ID.
4. The output includes job names.
5. The output includes job statuses and conclusions.
6. The output identifies failing job or jobs.
7. The output includes the failing job log text.

## Error validation

Validate clear failures for:

1. Missing `GITHUB_TOKEN` — must fail before any network call with an actionable message.
2. Missing `--repo`.
3. Invalid repository format.
4. Unknown repository.
5. Unknown run ID.
6. Authentication or authorization failure (bad token, insufficient scope).
7. Missing or unavailable logs.

## Completion criteria

This phase is complete only when:

1. All automated validation commands pass.
2. Manual validation succeeds against a real public failing GitHub Actions run.
3. `sleuth fetch run <RUN_ID> --repo owner/name` prints job names, statuses, and the failing log.
4. The roadmap status for Phase 1 can be updated to `completed`.