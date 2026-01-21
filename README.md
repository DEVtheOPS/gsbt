# gsbt - Gameserver Backup Tool

CLI tool to back up gameserver files via pluggable connectors (FTP, SFTP, Nitrado → FTP) into timestamped `.tar.gz` archives.

## Features (current state)
- **Connectors**: FTP, SFTP, Nitrado (fetches FTP creds via API)
- **Backup command** downloads matched files, archives them, and stores per-server backups with timestamps
- **Output modes**:
  - `text` (default): Plain text
  - `json`: Structured JSON for programmatic consumption
- **Metadata**: Optional structured context data (shown in verbose mode or JSON)
- **Config discovery**: `--config` > `$GSBT_CONFIG` > `./.gsbt-config.yml` > `~/.config/gsbt/config.yml`

> Note: Prune/list/restore commands are stubbed; only `backup` is functional right now.

## Install

### From release (recommended)
Download binaries from GitHub Releases (published via GoReleaser when tagging `v*.*.*`). Place `gsbt` on your `$PATH`.

### From source
```bash
go install github.com/devtheops/gsbt/cmd/gsbt@latest
```
Or build locally:
```bash
go build ./cmd/gsbt
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

## Development

### Quick Start

- **Tests**: `go test ./...`
- **Taskfile**: `task build`, `task test`, `task run -- --help`
- **Release**: tag `vX.Y.Z`; GitHub Actions will build/publish via GoReleaser

### Architecture

**Packages:**

- `internal/log` - Standardized logging with markup support (stripped for text/json)
  - Two modes: text (plain), json (structured)
  - Metadata support for structured context
- `internal/progress` - Progress reporting interface
  - `nullProgress` (quiet/json), `simpleProgress` (text)
  - Integrates with logger for consistency
- `internal/connector` - Pluggable connector interface
  - FTP, SFTP, Nitrado implementations
  - Pattern matching for include/exclude
- `internal/backup` - Backup orchestration
  - Archive creation, download management
  - Progress reporting integration
- `internal/config` - Configuration loading
  - YAML parsing, env var substitution
  - Config file discovery

**Adding a new connector:**

1. Implement `connector.Connector` interface
2. Add factory case in `connector.NewConnector()`
3. Follow existing patterns (FTP, SFTP examples)

**Adding markup to logs:**

The logger supports markup tags like `[green]`, but they are currently stripped in all output modes.

```go
logger.Info("[green]Success![/green] Operation completed") // Output: Success! Operation completed
```

## Roadmap

- Implement prune/list/restore commands
- Retry/backoff polish and integration tests
