# Context & Instructions

## Project Overview

**gsbt** (Gameserver Backup Tool) is a CLI utility written in Go for backing up game server files. It supports pluggable connectors (FTP, SFTP, Nitrado) and creates timestamped `.tar.gz` archives.

### Key Features

* **Connectors:** FTP, SFTP, Nitrado (API wrapper around FTP).
* **Output Modes:** Text (default), Rich (ANSI colors/progress), JSON (structured logging).
* **Configuration:** YAML-based with environment variable substitution support.
* **Architecture:** Modular design with `internal/` packages for backup logic, connectors, logging, and configuration.

## Building and Running

This project uses [Task](https://taskfile.dev/) for automation.

| Command              | Description                                                            |
| :------------------- | :--------------------------------------------------------------------- |
| `task build`         | Builds the `gsbt` binary to the current directory.                     |
| `task test`          | Runs all unit tests (`go test ./...`).                                 |
| `task run -- [args]` | Runs the application from source. Example: `task run -- backup --help` |
| `task tidy`          | Runs `go mod tidy` to clean up dependencies.                           |

### Manual Commands

If `task` is not available:

* **Build:** `go build ./cmd/gsbt`
* **Test:** `go test ./...`
* **Run:** `go run ./cmd/gsbt [args]`

## Development Conventions

### Workflows & Issue Tracking

This project uses **bd** (beads) for local issue tracking.

* **Start Work:** `bd ready` (find work), `bd update <id> --status in_progress` (claim).
* **Finish Work:** `bd close <id>`.
* **Sync:** `bd sync`.

### "Landing the Plane" (Session Completion)

**MANDATORY:** You must ensure all work is pushed before ending a session.

1. Run quality gates (`task test`, `task build`).
2. Update issue statuses (`bd close`, etc.).
3. **Push:**

    ```bash
    git pull --rebase
    bd sync
    git push
    ```

4. Verify `git status` shows up-to-date with origin.

### Code Structure

* `cmd/gsbt/`: Main entry point.
* `internal/connector/`: FTP, SFTP, and Nitrado connector implementations.
* `internal/backup/`: Core backup logic (archive creation, file transfer).
* `internal/config/`: Configuration loading and parsing.
* `internal/log/`: Custom logger with markup support.

### Configuration

Configuration is typically located at `~/.config/gsbt/config.yml` or `.gsbt-config.yml`.
Refer to `README.md` for a sample configuration structure.

## Agent Instructions

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
```

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:

   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```

5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**

* Work is NOT complete until `git push` succeeds
* NEVER stop before pushing - that leaves work stranded locally
* NEVER say "ready to push when you are" - YOU must push
* If push fails, resolve and retry until it succeeds
