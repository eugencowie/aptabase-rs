# Agent Instructions

## Development environment

This project uses mise to manage the development environment.

- Run `mise tasks` to see the available tasks.
- Use `mise run <task>` for standard operations.
- Run `mise ls -l` to see the managed tools.
- When invoking a managed tool directly, use `mise exec -- <command> [args]` rather than invoking the tool by its bare name.

## Upstream references

This repo is a fork of `aptabase/tauri-plugin-aptabase`. In commit messages, PR descriptions and comments, upstream PRs and issues should be cited as plain text by title or commit SHA. Avoid links and `#123` shorthand: GitHub cross-references those, which spams the upstream project with irrelevant notifications.

Open PRs only against `eugencowie/aptabase-rs`, unless the user explicitly asks for an upstream PR. Before running `gh pr create`, run `gh repo set-default eugencowie/aptabase-rs`. If you pass `--repo`, it must also be `eugencowie/aptabase-rs`. When opening a PR through the GitHub API or MCP tools, set the base repository to `eugencowie/aptabase-rs` explicitly.
