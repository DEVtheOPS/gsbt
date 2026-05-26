# Contributing

Thanks for your interest in improving gsbt.

## Prerequisites

- Go 1.26+
- [Task](https://taskfile.dev/) (recommended)

## Getting Started

```bash
git clone https://github.com/devtheops/gameserver-backup-tool
cd gameserver-backup-tool
go mod download
```

## Development Workflow

Use the Taskfile commands when available:

| Command | Description |
|---------|-------------|
| `task build` | Build `gsbt` locally |
| `task test` | Run all tests |
| `task run -- backup --help` | Run from source |
| `task tidy` | Run `go mod tidy` |

Equivalent direct commands:

| Command | Description |
|---------|-------------|
| `go build ./cmd/gsbt` | Build binary |
| `go test ./...` | Run tests |
| `go run ./cmd/gsbt [args]` | Run from source |

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
3. Ensure `task test` and `task build` pass
4. Open a PR with a clear description and related issue links

## Releases

Releases are automated:

- `release-please` opens/updates release PRs from Conventional Commits
- Merging the release PR creates a version tag and GitHub Release
- The release workflow runs GoReleaser to publish release artifacts
