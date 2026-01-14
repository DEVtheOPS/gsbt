# Gameserver Backup Tool (GSBT)

I am wanting to make a modular game backup tool, it will have "connectors" such as `ftp` or `scp` or `nitrado` to provide ways to connect to servers. These connectors will augment what the server entry config looks like for that gameserver.

I am unsure if we should use golang or rust but I want it to be a compiled binary that is released.

The tool should support both cli arguments or a config file or both. cli arguments would override config file settings.

## Config Examples

```yml

default:
  backup_type: single # Valide values: single, monthly, weekly, daily, hourly
  backup_location: /src/gameserver_backups/
  prune_age: 30 # Age in days
  nitrado_api_key: 42342rf23f23r23rf2f23f2
  ignore_patterns:
    - "*.backup" # Should support glob patterns.

servers:
  - description: ark server 1
    backup_type: hourly
    prune_age: 7
    ignore_patterns:
      - "*.backup"
    connection:
      type: ftp
      username: myuser
      password: mypassword
      remote_path: arksa/
      host: 127.0.0.1
      port: 990
      ssl: true
  - description: ark server 2
    backup_type: hourly
    prune_age: 7
    backup_location: /some/other/location/
    connection:
      type: nitrado
      api_key: 4234523rtfg2q3g2
      service_id: 18341077
```
