# gsbt - Gameserver Backup Tool

CLI tool to back up gameserver files via pluggable connectors (FTP, SFTP, Nitrado → FTP) into timestamped `.tar.gz` archives.

## Features (current state)
- Connectors: FTP, SFTP, Nitrado (fetches FTP creds via API).
- Backup command downloads matched files, archives them, and stores per-server backups with timestamps.
- Progress options: `text` (default), `rich` (single live bar per server, TTY), `json` (quiet machine output).
- Config discovery: `--config` > `$GSBT_CONFIG` > `./.gsbt-config.yml` > `~/.config/gsbt/config.yml`.

> Note: Prune/list/restore commands are stubbed; only `backup` is functional right now.

## Install

### From release (recommended)
Download binaries from GitHub Releases (published via GoReleaser when tagging `v*.*.*`). Place `gsbt` on your `$PATH`.

### From source
```bash
go install github.com/digitalfiz/gsbt/cmd/gsbt@latest
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
gsbt backup --config /path/to/config.yml
```
Options:
- `--server name` – target a single server.
- `--output rich` – live progress bar (TTY only). `text` (default) or `json` also available.
- `--sequential` – reserved; currently backups run sequentially.

Archives are stored at `{backup_location}/{timestamp}.tar.gz` with temp files under `{backup_location}/.tmp/`.

### Notes on Nitrado
- Provide `service_id` and an API key (`connection.api_key` or `defaults.nitrado_api_key`).
- Connector fetches FTP creds then reuses the FTP pipeline.

## Development
- Tests: `go test ./...`
- Taskfile: `task build`, `task test`, `task run -- --help`
- Release: tag `vX.Y.Z`; GitHub Actions will build/publish via GoReleaser.

## Roadmap
- Implement prune/list/restore commands
- Parallel backups with multi-bar rich output
- Retry/backoff polish and integration tests
