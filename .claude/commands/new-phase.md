Start a new phase of the project roadmap by following these steps in order:

1. Read `specs/roadmap.md` and identify the next phase that is not marked DONE. Use its title and tasks as the source of truth.

2. Derive from the roadmap entry:
   - A branch name in kebab-case from the phase title (e.g. `phase-5-api-client-integration`)
   - A directory name in the format `YYYY-MM-DD-short-name` using today's date (e.g. `2026-05-24-api-client-integration`)

3. Ask the user focused clarifying questions about the upcoming phase — things not already answered by the roadmap. For example: preferred patterns, constraints, anything ambiguous in the task list. Wait for their answers before proceeding.

4. Create and switch to the new branch.

5. Create the directory `specs/YYYY-MM-DD-short-name/` and scaffold three files inside it:

   **plan.md** — a series of numbered task groups derived from the roadmap tasks for this phase. Each group has a heading and bullet-point tasks. Incorporate any decisions from the user's answers in step 3.

   **requirements.md** — scope, decisions, and context for this phase. Populate it from the roadmap description and the user's answers. Include an `## Open Questions` section for anything still unresolved.

   **validation.md** — criteria for knowing the implementation succeeded and is ready to merge. Include sections for: manual smoke tests, automated test expectations, and a merge checklist.

6. Report back: show the branch name, the directory created, and a brief summary of what was scaffolded.
