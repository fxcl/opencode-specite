# Specite

## Project Overview

Specite is a Rust companion CLI for OpenCode that installs bundled slash-command templates into projects and manages a spec-driven iteration workflow under `.specite/`.

## Structure Map

```text
specite/
|- src/                        # Rust source for the `specite` CLI binary
|  |- main.rs                  # clap entrypoint and command dispatch
|  |- lib.rs                   # Public module surface
|  |- init.rs                  # Project initialization, managed command install, legacy cleanup
|  |- iterations.rs            # Iteration metadata, id resolution, stage updates, path helpers
|  |- project.rs               # Upward project-root discovery and display-path helpers
|  |- prompts.rs               # Prompt generation for command and delegated subagent flows
|  |- error.rs                 # Domain error types
|  \- assets.rs                # Compile-time embedded assets (include_str!)
|- assets/                     # Bundled templates (commands, agents, templates, prompts)
|  |- commands/                # Markdown templates copied into `.opencode/commands/`
|  |- agents/                  # Markdown templates copied into `.opencode/agents/`
|  |- command_prompts/         # Markdown prompt templates rendered for top-level command flows
|  |- subagent_prompts/        # Markdown prompt templates rendered for delegated subagent tasks
|  \- templates/               # SPEC.md and PLAN.md templates copied into `.specite/templates/`
|- tests/                      # Integration tests (cli_smoke.rs)
|- Cargo.toml                  # Rust package metadata and binary entrypoint
|- README.md                   # User-facing install and workflow documentation
|- AGENTS.md                   # Project guidance for coding agents
```

## Development Guide

- Primary workflow: update Rust source in `src/` and bundled assets in `assets/` together.
- Installation model: `cargo build --release` for a self-contained binary; end-user workflow is `specite init`, then `specite ...` inside initialized projects.
- Verification: run `cargo test` after changes; manually validate `specite init`, `specite new`, `specite list`, `specite update`, and `specite prompt ...` in a temp project when behavior changes.
- Prompt templates: keep markdown concise; command prompts live in `assets/command_prompts/`, delegated-agent prompts live in `assets/subagent_prompts/`, and OpenCode expands `$` placeholders plus inline shell snippets before sending the final prompt to the model.
- Assets are embedded at compile time via `include_str!` in `src/assets.rs`; add new files there when adding templates.
- Scope: managed runtime behavior lives in Rust modules, not copied `.opencode/scripts/` helpers; preserve backward-compatible init migration behavior when changing project setup.
