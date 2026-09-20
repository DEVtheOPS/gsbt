# Context & Instructions

## Project Overview

**gsbt** (Gameserver Backup Tool) is a CLI utility written in Rust for backing up game server files. It supports pluggable connectors (FTP, SFTP, Nitrado) and creates timestamped `.tar.gz` archives.

### Key Features

* **Connectors:** FTP, SFTP, Nitrado (API wrapper around FTP).
* **Output Modes:** Text (default), JSON (structured logging).
* **Configuration:** YAML-based with environment variable substitution support.
* **Architecture:** Modular design with `src/` modules for backup logic, connectors, logging, and configuration.

## Building and Running

This project uses [Task](https://taskfile.dev/) for automation.

| Command              | Description                                                            |
| :------------------- | :--------------------------------------------------------------------- |
| `task build`         | Builds the `gsbt` binary (release) and regenerates the schema.         |
| `task test`          | Runs all unit tests (`cargo test --all-targets`).                      |
| `task run -- [args]` | Runs the application from source. Example: `task run -- backup --help` |
| `task fmt`           | Formats the source (`cargo fmt`).                                      |
| `task lint`          | Runs clippy with warnings denied.                                      |
| `task tidy`          | Updates dependencies (`cargo update`).                                 |

### Manual Commands

If `task` is not available:

* **Build:** `cargo build --release`
* **Test:** `cargo test --all-targets`
* **Run:** `cargo run -- [args]`

### Code Structure

* `src/main.rs`: Binary entry point.
* `src/lib.rs`: Library root exposing the modules.
* `src/cli/`: Command line interface (`clap`).
* `src/connector/`: FTP, SFTP, and Nitrado connector implementations.
* `src/backup/`: Core backup logic (archive creation, file transfer).
* `src/config/`: Configuration loading and parsing.
* `src/log.rs`: Custom logger with markup support.
* `src/progress.rs`: Progress reporting.
* `src/bin/schema_gen.rs`: JSON schema generator (`schemars`).

### Configuration

Configuration is typically located at `~/.config/gsbt/config.yml` or `.gsbt-config.yml`.
Refer to `README.md` for a sample configuration structure.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **PUSH TO REMOTE** - This is MANDATORY:

   ```bash
   git pull --rebase
   git push
   git status  # MUST show "up to date with origin"
   ```

4. **Clean up** - Clear stashes, prune remote branches
5. **Verify** - All changes committed AND pushed
6. **Hand off** - Provide context for next session

**CRITICAL RULES:**

* Work is NOT complete until `git push` succeeds
* NEVER stop before pushing - that leaves work stranded locally
* NEVER say "ready to push when you are" - YOU must push
* If push fails, resolve and retry until it succeeds
