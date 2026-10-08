# gsbt - Gameserver Backup Tool

[![CI](https://img.shields.io/github/actions/workflow/status/devtheops/gameserver-backup-tool/ci.yml?branch=main&label=ci)](https://github.com/devtheops/gameserver-backup-tool/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/actions/workflow/status/devtheops/gameserver-backup-tool/release.yml?label=release)](https://github.com/devtheops/gameserver-backup-tool/actions/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/devtheops/gameserver-backup-tool)](https://github.com/devtheops/gameserver-backup-tool/releases)
[![License](https://img.shields.io/github/license/devtheops/gameserver-backup-tool)](LICENSE)

CLI tool to back up gameserver files via pluggable connectors (FTP, SFTP, Nitrado → FTP) into timestamped `.tar.gz` archives.

See `CONTRIBUTING.md` for development and contribution guidance, and `SECURITY.md` for responsible vulnerability reporting.

## Features (current state)
- **Connectors**: FTP (with optional explicit TLS), SFTP, Nitrado (fetches FTP creds via API)
- **Backup command** downloads matched files, archives them, and stores per-server backups with timestamps
- **Restore command** restores an archive to a configured server or extracts it locally, with plan-first dry runs, conflict-aware overwrite policy and remote confirmation
- **Output modes**:
  - `text` (default): Plain text
  - `json`: Structured JSON for programmatic consumption
- **Metadata**: Optional structured context data (shown in verbose mode or JSON)
- **Config discovery**: `--config` > `$GSBT_CONFIG` > `./.gsbt-config.yml` > `~/.config/gsbt/config.yml`

> Note: Prune/list commands are stubbed; `backup` and `restore` are functional.

## Install

### From release (recommended)
Download binaries from GitHub Releases. Place `gsbt` on your `$PATH`.

### From source
```bash
cargo build --release
# or install into ~/.cargo/bin
cargo install --path .
```

## Usage

### Sample config (`~/.config/gsbt/config.yml`)
```yaml
defaults:
  backup_location: /srv/gameserver_backups
  prune_age: 30
  retry_attempts: 3
  retry_delay: 5
  retry_backoff: true
  nitrado_api_key: ${NITRADO_API_KEY}

servers:
  - name: my-ftp
    connection:
      type: ftp
      host: ftp.example.com
      port: 21
      username: user
      password: ${FTP_PASS}
      remote_path: /game/saves
      include: ["*"]
      exclude: ["*.log", "Logs/"]

  - name: nitrado-ark
    connection:
      type: nitrado
      service_id: 18341077
      api_key: ${NITRADO_ARK_KEY}   # falls back to defaults.nitrado_api_key
      remote_path: /games/ark/saves
```

### Run a backup
```bash
# Basic usage
gsbt backup

# Backup single server
gsbt backup --server my-ftp

# JSON output for scripts
gsbt backup --output json

# Verbose mode (shows debug logs and metadata)
gsbt backup --verbose

# Quiet mode (errors only)
gsbt backup --quiet
```

**Options:**
- `--server name` – Target a single server
- `--output <mode>` – Output format: `text` (default), `json` (structured)
- `--verbose` / `-v` – Enable debug logging and show metadata
- `--quiet` / `-q` – Only show errors
- `--sequential` – Run backups one server at a time (default is parallel)

Archives are stored at `{backup_location}/{timestamp}.tar.gz` with temp files under `{backup_location}/.tmp/`.

### Output Modes

**Text mode** (default):
```
[server-name] starting backup
Files: 5, Total: 2.3 MB
- saves/world.dat (1.2 MB)
- config/server.ini (0.1 MB)
[server-name] saved /backups/2026-01-15_154500.tar.gz (5 files, 2.3 MB, 3.2s)
```

**JSON mode** (`--output json`):
```json
{"timestamp":"2026-01-15T15:45:00Z","level":"info","message":"starting backup","prefix":"server-name"}
{"timestamp":"2026-01-15T15:45:03Z","level":"info","message":"saved /backups/2026-01-15_154500.tar.gz","prefix":"server-name","metadata":{"archive_path":"/backups/2026-01-15_154500.tar.gz","files":5,"bytes":2400000,"duration_sec":3.2}}
```

### Notes on Nitrado
- Provide `service_id` and an API key (`connection.api_key` or `defaults.nitrado_api_key`).
- Connector fetches FTP creds then reuses the FTP pipeline.

### Restore a backup

Restore takes an archive and a target. Exactly one target mode is required:
`--server <name>` (remote) or `--local <dir>` (local extraction).

```bash
# Preview a local extraction (validates, plans and detects conflicts; writes nothing)
gsbt restore ./backups/2026-01-15_154500.tar.gz --local /srv/recover --dry-run

# Extract locally
gsbt restore <archive> --local /srv/recover

# Restore to a configured server (interactive confirmation unless --force)
gsbt restore <archive> --server my-ftp

# Non-interactive automation
gsbt restore <archive> --server my-ftp --force

# Replace existing files and drop the first path component
gsbt restore <archive> --local /srv/recover --overwrite --strip-components 1

# Include/exclude filtering (both repeatable)
gsbt restore <archive> --local /srv/recover --include '*.sav' --exclude '*.log'

# JSON output for scripts
gsbt restore <archive> --server my-ftp --dry-run --output json
```

**Restore options:**
- `--server <name>` / `--local <dir>` – exactly one target is required
- `--dry-run` – build and report the plan without writing anything
- `--overwrite` – replace existing files (default: skip existing files)
- `--force` – skip the interactive confirmation for remote restores
- `--include <glob>` / `--exclude <glob>` – repeatable member filters (include-first)
- `--strip-components <n>` – remove the first `n` path components (default `0`)

**Safety behavior:**
- Absolute paths and `..` traversal entries are rejected and never written.
- Symlink/hardlink entries are not restored: they are reported, skipped, and cause a non-zero exit.
- Remote restores require interactive confirmation unless `--force` is given; in a non-interactive environment without `--force` the command refuses to run.
- Per-file failures are reported and counted, and execution continues; the command exits non-zero if any entry failed.
- Duplicate paths within an archive resolve to the last entry, with a warning.
- `--no-overwrite` is intentionally not supported; the default is already safe.

## Development

### Quick Start

- **Tests**: `cargo test --all-targets`
- **Taskfile**: `task build`, `task test`, `task run -- --help`
- **Release**: tag `vX.Y.Z`; GitHub Actions builds/publishes release artifacts

### Architecture

Cargo workspace layout (`src/`):

- `log.rs` - Standardized logging with markup support (stripped for text/json)
  - Two modes: text (plain), json (structured)
  - Metadata support for structured context
- `progress.rs` - Progress reporting
  - `Reporter::Null` (quiet/json), `Reporter::Simple` (text)
  - Integrates with the logger for consistency
- `connector/` - Pluggable connector interface
  - `Connector` trait with FTP, SFTP, Nitrado implementations
  - Pattern matching for include/exclude
- `backup/` - Backup orchestration
  - Archive creation (`archive.rs`), download management (`manager.rs`)
  - Progress reporting integration
- `restore/` - Restore orchestration
  - Archive indexing (`archive_index.rs`), path safety (`path_rewrite.rs`)
  - Planning and conflict detection (`planner.rs`, `conflict_detector.rs`)
  - Local and remote executors (`executor_local.rs`, `executor_remote.rs`)
- `config/` - Configuration loading
  - YAML parsing (`serde`), env var substitution
  - Config file discovery
- `cli/` - Command line interface (`clap`)

**Adding a new connector:**

1. Implement the `connector::Connector` trait
2. Add a factory case in `connector::factory::new_connector()`
3. Follow existing patterns (`ftp.rs`, `sftp.rs`)

**Adding markup to logs:**

The logger supports markup tags like `[green]`, but they are currently stripped in all output modes.

```rust
logger.info("[green]Success![/green] Operation completed"); // Output: Success! Operation completed
```

## Roadmap

- Implement prune/list commands
- Retry/backoff polish and integration tests
