# Rules for agents in this repo

Everything you need to work here is in this repo. The full pipeline and rules (`PIPELINE.md`, `RULES.md`, `RUBRIC.md`, `roles/`) live in the owner's private pipeline repo; this is the short copy.

1. **Find the stage** in `PIPELINE.md`. The spec is `SPEC.md`, the plan `docs/ROADMAP.md`, the code map `docs/code-map.md`.
2. **Work from an issue.** Take one that is not `in-progress` and not blocked; add `in-progress` before you start. Its body is your brief; an Easy issue has a "Qwen brief" section with the expected results and the code map snippet.
3. **Build and check.** This project is Linux-only (SPEC, R04); say whether you built on the host or in the build container. Run `tools/check.sh` (it exports a temporary `XDG_RUNTIME_DIR`); the docs check forbids private details.
4. **Open the PR** after the gate passes.
5. **Label the PR** `by:<machine>` and `model:<model>`, and write `Closes #N`, or `Refs #N (code half)` when the WP has a device Q row; the issue closes when that device session passes. Every PR body has `## Verified` listing each done-check command and exit status, and `## Not verified` listing each host skip and its device Q row; never claim a run that was not done.
6. **A failed gate** is reported in the PR; issue edits are handled by the owner or project manager.
7. **Use the project's words** from `SPEC.md` and the rulings in `docs/notes/DECISIONS.md`.
8. **No private details** in code, commits or PR text: no names, e-mail addresses, home paths or machine names. Commit as the no-reply identity.

## This project

- Project notes and run sheets live in `docs/`; research scripts in `tools/`.
- Only games you own; no game files in the repo.
