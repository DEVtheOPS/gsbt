# Gameserver Backup Tool (gsbt) - Design Document

**Date:** 2026-01-14
**Status:** Approved

## Overview

gsbt is a Go CLI tool for backing up game server files via multiple connector types. It runs via external schedulers (cron/systemd) or ad-hoc, stores backups as compressed `.tar.gz` archives, and supports automatic pruning.

### Key Features

- **Modular connectors:** FTP, SFTP, Nitrado (v1)
- **Flexible config:** YAML with environment variable substitution
- **Multiple execution modes:** Scheduled via cron, ad-hoc with overrides, fully ad-hoc without config
- **Parallel execution:** Concurrent backups by default
- **Reliability:** Retries with exponential backoff, throttle-aware

## CLI Interface

### Global Flags

```bash
gsbt [global flags] <command> [command flags]

Global:
  --config=PATH        Config file path
  --output=FORMAT      Output format: text (default), json
  --verbose, -v        Verbose logging
  --quiet, -q          Suppress non-error output
```

### Commands

```bash
# Backup
gsbt backup                     # Back up all configured servers (parallel)
gsbt backup --server="ark1"     # Back up specific server
gsbt backup --sequential        # Run sequentially instead of parallel
gsbt backup --type=ftp --host=x --user=y --pass=z --remote-path=/saves
                                # Fully ad-hoc, no config needed

# Prune
gsbt prune                      # Remove backups older than prune_age
gsbt prune --server="ark1"      # Prune specific server only
gsbt prune --dry-run            # Show what would be deleted

# List
gsbt list                       # Show configured servers + last backup info
gsbt list --server="ark1"       # Specific server details

# Restore
gsbt restore <backup-file> --server="ark1"   # Restore to server
gsbt restore <backup-file> --local=/path/    # Extract locally
gsbt restore <backup-file> --server="ark1" --dry-run
gsbt restore <backup-file> --server="ark1" --force  # Skip confirmation
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Partial failure (some servers failed) |
| 2 | Total failure |

### Config Discovery Order

1. `--config` flag
2. `$GSBT_CONFIG` environment variable
3. `./.gsbt-config.yml` (current directory)
4. `~/.config/gsbt/config.yml` (user default)

First match wins.

## Configuration File

```yaml
# ~/.config/gsbt/config.yml

defaults:
  backup_location: /srv/gameserver_backups/
  temp_dir: ""                     # Empty = {backup_location}/.tmp/ (default)
  prune_age: 30                    # Days to keep backups
  retry_attempts: 3
  retry_delay: 5                   # Base delay in seconds
  retry_backoff: true              # Exponential backoff (default)
  env_file: ""                     # Optional: path to .env file to load
  nitrado_api_key: ${NITRADO_API_KEY}

servers:
  - name: ark-server-1             # Unique identifier (used in --server flag)
    description: "ARK PvE Cluster Node 1"
    prune_age: 7                   # Override default
    connection:
      type: ftp
      host: 192.168.1.100
      port: 21                     # Optional, defaults per type
      username: arkserver
      password: ${ARK_FTP_PASS}
      passive: true                # Default true
      remote_path: /ShooterGame/Saved/
      exclude: ["*.log", "Logs/", "*.tmp", "Cache/"]
      # include: ["*"]             # Implicit default

  - name: ark-server-2
    description: "ARK PvE Cluster Node 2"
    backup_location: /mnt/nas/ark-backups/   # Override default
    connection:
      type: sftp
      host: 192.168.1.101
      port: 22
      username: backup
      key_file: ${HOME}/.ssh/ark_backup_key  # SSH key auth
      remote_path: /home/ark/server/Saved/
      exclude: ["*.log"]

  - name: nitrado-ark
    description: "Nitrado Hosted ARK"
    connection:
      type: nitrado
      api_key: ${NITRADO_ARK_KEY}  # Override default key
      service_id: 18341077
      remote_path: /games/ark/saves/   # Optional override
      exclude: ["*.log"]
```

### Environment Variable Substitution

Secrets are referenced via `${VAR}` or `${VAR:-default}` syntax:

```yaml
password: ${FTP_PASSWORD}
api_key: ${API_KEY:-default_key}
```

Optional `env_file` setting loads a `.env` file before processing config.

## Connector Architecture

### Interface

```go
type Connector interface {
    // Connect establishes connection to the remote server
    Connect(ctx context.Context) error

    // List returns files matching include/exclude patterns at remote_path
    List(ctx context.Context) ([]FileInfo, error)

    // Download retrieves a file to local destination
    Download(ctx context.Context, remotePath, localPath string) error

    // Upload sends a file to remote destination (for restore)
    Upload(ctx context.Context, localPath, remotePath string) error

    // Close terminates the connection
    Close() error
}
```

### V1 Connectors

| Connector | Auth Methods | Notes |
|-----------|--------------|-------|
| `ftp` | username/password | Passive mode default, explicit TLS supported |
| `sftp` | password, key_file | Standard SSH/SFTP |
| `nitrado` | API key | Credential resolver + FTP wrapper |

### Connector-Specific Config

**FTP:**
```yaml
type: ftp
host: required
port: 21 (default)
username: required
password: required
passive: true (default)
tls: false (default)       # Explicit FTPS
remote_path: required
include: ["*"] (default)
exclude: [] (default)
```

**SFTP:**
```yaml
type: sftp
host: required
port: 22 (default)
username: required
password: optional         # One of password or key_file required
key_file: optional
remote_path: required
include: ["*"] (default)
exclude: [] (default)
```

**Nitrado:**
```yaml
type: nitrado
api_key: required          # Falls back to defaults.nitrado_api_key
service_id: required
remote_path: optional      # Override Nitrado default path
include: ["*"] (default)
exclude: [] (default)
```

### Nitrado Implementation

Nitrado connector is a credential resolver that wraps FTP:

1. Call Nitrado API with `service_id` to get FTP credentials (host, username, password)
2. Initialize FTP connector with retrieved credentials
3. Delegate all file operations to FTP connector

Rate limiting (HTTP 429) is handled during credential fetch with exponential backoff.

## Backup Process

### Workflow

```
1. Connect to remote (via connector)
2. List files at remote_path matching include/exclude
3. Download files to temp directory
4. Create .tar.gz archive with timestamp
5. Move archive to backup_location/{server-name}/
6. Disconnect
7. Report success/failure
```

### Storage Layout

```
/srv/gameserver_backups/
├── .tmp/                        # Temp directory for in-progress backups
├── ark-server-1/
│   ├── 2024-01-15_143022.tar.gz
│   ├── 2024-01-15_160000.tar.gz
│   └── 2024-01-16_080000.tar.gz
├── ark-server-2/
│   └── 2024-01-16_080000.tar.gz
└── nitrado-ark/
    └── 2024-01-16_090000.tar.gz
```

**Filename format:** `YYYY-MM-DD_HHMMSS.tar.gz` (UTC timestamp)

**Archive contents:** Preserves directory structure from `remote_path` root.

### Parallel Execution

- Default: all servers concurrently
- `--sequential` flag: one at a time
- Per-server log prefixes: `[server-name] message`
- Temp directories are per-server to avoid conflicts

## Reliability

### Retry Strategy

```
Attempt 1: immediate
Attempt 2: wait (retry_delay) + jitter
Attempt 3: wait (retry_delay * 2) + jitter
Attempt 4: wait (retry_delay * 4) + jitter
...up to retry_attempts
```

Jitter: random 0-50% of delay to prevent thundering herd.

### Throttle Handling

- Respect `Retry-After` header if present
- Back off on HTTP 429, 503, 529 status codes
- FTP/SFTP: retry on "too many connections" errors

### Configuration

```yaml
defaults:
  retry_attempts: 3        # Default
  retry_delay: 5           # Base delay in seconds
  retry_backoff: true      # Exponential backoff enabled
```

## Prune Command

Removes backups older than `prune_age` days.

**Logic:**
```
For each server:
  1. List *.tar.gz in {backup_location}/{server-name}/
  2. Parse timestamp from filename
  3. Delete files older than server's prune_age (or default)
  4. Report: deleted N files, freed X bytes
```

## Restore Command

**Workflow:**
```
1. Extract archive to temp directory
2. If --local: move to destination, done
3. If --server: connect to server, upload files to remote_path
4. Report success/failure
```

**Safety:** Restore overwrites existing files. Prompts for confirmation unless `--force` or `--dry-run`.

## Output Formats

### Text (default)

Human-readable output with colors and formatting.

**List example:**
```
SERVERS (3 configured)

  ark-server-1
    Description:  ARK PvE Cluster Node 1
    Connection:   ftp://192.168.1.100:21
    Backup Path:  /srv/gameserver_backups/ark-server-1/
    Prune Age:    7 days
    Last Backup:  2024-01-15 14:30:22 (2 hours ago)
    Backup Count: 12
    Total Size:   4.2 GB
```

### JSON (`--output=json`)

Machine-readable output for scripting and monitoring.

```json
{
  "servers": [
    {
      "name": "ark-server-1",
      "description": "ARK PvE Cluster Node 1",
      "connection_type": "ftp",
      "host": "192.168.1.100",
      "backup_path": "/srv/gameserver_backups/ark-server-1/",
      "prune_age_days": 7,
      "last_backup": "2024-01-15T14:30:22Z",
      "backup_count": 12,
      "total_size_bytes": 4509715660
    }
  ]
}
```

## Project Structure

```
gsbt/
├── cmd/
│   └── gsbt/
│       └── main.go              # Entry point
├── internal/
│   ├── cli/                     # Command definitions
│   │   ├── root.go              # Global flags, config loading
│   │   ├── backup.go
│   │   ├── prune.go
│   │   ├── restore.go
│   │   └── list.go
│   ├── config/                  # Config parsing, env var expansion
│   │   ├── config.go
│   │   └── envsubst.go
│   ├── connector/               # Connector interface + implementations
│   │   ├── connector.go         # Interface definition
│   │   ├── ftp.go
│   │   ├── sftp.go
│   │   └── nitrado.go
│   ├── backup/                  # Backup orchestration
│   │   ├── backup.go
│   │   └── archive.go
│   ├── prune/
│   │   └── prune.go
│   └── output/                  # Text/JSON formatters
│       └── output.go
├── go.mod
├── go.sum
└── README.md
```

## Dependencies

| Library | Purpose |
|---------|---------|
| `github.com/spf13/cobra` | CLI framework |
| `github.com/jlaffaye/ftp` | FTP client |
| `github.com/pkg/sftp` | SFTP client |
| `golang.org/x/crypto/ssh` | SSH for SFTP |
| `gopkg.in/yaml.v3` | Config parsing |
| `github.com/joho/godotenv` | .env file loading |

## Summary

| Aspect | Decision |
|--------|----------|
| Language | Go |
| Commands | `backup`, `prune`, `list`, `restore` |
| Connectors (v1) | FTP, SFTP, Nitrado (FTP wrapper) |
| Backup format | `.tar.gz` archives with UTC timestamps |
| Storage layout | `{backup_location}/{server-name}/YYYY-MM-DD_HHMMSS.tar.gz` |
| Config discovery | `--config` → `$GSBT_CONFIG` → `./.gsbt-config.yml` → `~/.config/gsbt/config.yml` |
| Secrets | `${VAR}` env var substitution, optional `env_file` |
| Execution | Parallel by default, `--sequential` flag |
| Reliability | Retries with exponential backoff, throttle-aware |
| Output | Text default, `--output=json` global flag |
| Exit codes | 0 (success), 1 (partial), 2 (total failure) |
| Temp directory | `{backup_location}/.tmp/` default, configurable |
| Pruning | Age-based per `prune_age` setting |
