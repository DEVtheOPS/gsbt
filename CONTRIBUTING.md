# Contributing

Thanks for your interest in improving gsbt.

## Prerequisites

- Rust 1.85+ (MSRV); latest stable recommended
- [Task](https://taskfile.dev/) (recommended)

## Getting Started

```bash
git clone https://github.com/devtheops/gameserver-backup-tool
cd gameserver-backup-tool
cargo build
```

## Development Workflow

Use the Taskfile commands when available:

| Command | Description |
|---------|-------------|
| `task build` | Build `gsbt` locally (release) and regenerate the schema |
| `task test` | Run all tests |
| `task run -- backup --help` | Run from source |
| `task fmt` | Format the source |
| `task lint` | Run clippy with warnings denied |
| `task tidy` | Update dependencies |

Equivalent direct commands:

| Command | Description |
|---------|-------------|
| `cargo build --release` | Build binary |
| `cargo test --all-targets` | Run tests |
| `cargo run -- [args]` | Run from source |
| `cargo fmt --all` | Format |
| `cargo clippy --all-targets -- -D warnings` | Lint |

## Commit Messages

This project uses [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/). Release Please reads commit history to build changelogs and version bumps.

Examples:

```text
feat(connector): add retry support for SFTP downloads
fix(config): handle missing env var defaults safely
docs(readme): clarify JSON output examples
```

## Pull Requests

1. Create a branch from `main`
2. Make focused changes and add/update tests
3. Ensure `task test`, `task lint` and `task build` pass
4. Open a PR with a clear description and related issue links

## Releases

Releases are automated:

- `release-please` opens/updates release PRs from Conventional Commits
- Merging the release PR creates a version tag and GitHub Release
- The release workflow builds `gsbt` for Linux, macOS and Windows and publishes the artifacts
