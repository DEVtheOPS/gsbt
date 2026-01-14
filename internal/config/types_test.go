// internal/config/types_test.go
package config

import (
	"testing"

	"gopkg.in/yaml.v3"
)

func TestConfigParsing(t *testing.T) {
	yamlData := `
defaults:
  backup_location: /srv/backups/
  prune_age: 30
  retry_attempts: 3

servers:
  - name: test-server
    description: Test Server
    connection:
      type: ftp
      host: localhost
      username: user
      password: pass
      remote_path: /saves/
`
	var cfg Config
	err := yaml.Unmarshal([]byte(yamlData), &cfg)
	if err != nil {
		t.Fatalf("failed to parse yaml: %v", err)
	}

	if cfg.Defaults.BackupLocation != "/srv/backups/" {
		t.Errorf("expected backup_location /srv/backups/, got %s", cfg.Defaults.BackupLocation)
	}

	if len(cfg.Servers) != 1 {
		t.Fatalf("expected 1 server, got %d", len(cfg.Servers))
	}

	if cfg.Servers[0].Name != "test-server" {
		t.Errorf("expected server name test-server, got %s", cfg.Servers[0].Name)
	}

	if cfg.Servers[0].Connection.Type != "ftp" {
		t.Errorf("expected connection type ftp, got %s", cfg.Servers[0].Connection.Type)
	}
}
