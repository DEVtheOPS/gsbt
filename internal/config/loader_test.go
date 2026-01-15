// internal/config/loader_test.go
package config

import (
	"os"
	"path/filepath"
	"testing"
)

func TestFindConfigFile(t *testing.T) {
	// Create temp directory for test
	tmpDir, err := os.MkdirTemp("", "gsbt-test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	// Create test config file
	testConfig := filepath.Join(tmpDir, ".gsbt-config.yml")
	os.WriteFile(testConfig, []byte("defaults:\n  prune_age: 30\n"), 0644)

	// Test explicit flag
	found, err := FindConfigFile(testConfig)
	if err != nil {
		t.Errorf("FindConfigFile with explicit path failed: %v", err)
	}
	if found != testConfig {
		t.Errorf("expected %s, got %s", testConfig, found)
	}

	// Test env var
	os.Setenv("GSBT_CONFIG", testConfig)
	defer os.Unsetenv("GSBT_CONFIG")

	found, err = FindConfigFile("")
	if err != nil {
		t.Errorf("FindConfigFile with env var failed: %v", err)
	}
	if found != testConfig {
		t.Errorf("expected %s, got %s", testConfig, found)
	}
}

func TestLoadConfig(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "gsbt-test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	configContent := `
defaults:
  backup_location: /srv/backups/
  prune_age: 30

servers:
  - name: test
    connection:
      type: ftp
      host: localhost
      username: user
      password: pass
      remote_path: /saves/
`
	testConfig := filepath.Join(tmpDir, "config.yml")
	os.WriteFile(testConfig, []byte(configContent), 0644)

	cfg, err := LoadConfig(testConfig)
	if err != nil {
		t.Fatalf("LoadConfig failed: %v", err)
	}

	if cfg.Defaults.PruneAge != 30 {
		t.Errorf("expected prune_age 30, got %d", cfg.Defaults.PruneAge)
	}

	if cfg.Defaults.BackupLocation != "/srv/backups/" {
		t.Errorf("expected backup_location /srv/backups/, got %s", cfg.Defaults.BackupLocation)
	}

	if cfg.Servers[0].GetBackupLocation(cfg.Defaults) != filepath.Join(cfg.Defaults.BackupLocation, "test") {
		t.Errorf("expected derived backup_location %s, got %s", filepath.Join(cfg.Defaults.BackupLocation, "test"), cfg.Servers[0].GetBackupLocation(cfg.Defaults))
	}

	if len(cfg.Servers) != 1 {
		t.Fatalf("expected 1 server, got %d", len(cfg.Servers))
	}

	if cfg.Servers[0].Name != "test" {
		t.Errorf("expected server name test, got %s", cfg.Servers[0].Name)
	}
}

func TestLoadConfigWithDefaults(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "gsbt-test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	// Minimal config - should apply defaults
	configContent := `
defaults:
  backup_location: /srv/backups/

servers:
  - name: test
    connection:
      type: ftp
      host: localhost
`
	testConfig := filepath.Join(tmpDir, "config.yml")
	os.WriteFile(testConfig, []byte(configContent), 0644)

	cfg, err := LoadConfig(testConfig)
	if err != nil {
		t.Fatalf("LoadConfig failed: %v", err)
	}

	// Check defaults are applied
	if cfg.Defaults.RetryAttempts != 3 {
		t.Errorf("expected default retry_attempts 3, got %d", cfg.Defaults.RetryAttempts)
	}
	if cfg.Defaults.RetryDelay != 5 {
		t.Errorf("expected default retry_delay 5, got %d", cfg.Defaults.RetryDelay)
	}
	if cfg.Defaults.PruneAge != 30 {
		t.Errorf("expected default prune_age 30, got %d", cfg.Defaults.PruneAge)
	}
}

func TestLoadConfigWithEnvVars(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "gsbt-test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	// Set test env vars
	os.Setenv("TEST_BACKUP_PATH", "/test/backups")
	os.Setenv("TEST_FTP_HOST", "test.example.com")
	defer os.Unsetenv("TEST_BACKUP_PATH")
	defer os.Unsetenv("TEST_FTP_HOST")

	configContent := `
defaults:
  backup_location: ${TEST_BACKUP_PATH}

servers:
  - name: test
    connection:
      type: ftp
      host: ${TEST_FTP_HOST}
`
	testConfig := filepath.Join(tmpDir, "config.yml")
	os.WriteFile(testConfig, []byte(configContent), 0644)

	cfg, err := LoadConfig(testConfig)
	if err != nil {
		t.Fatalf("LoadConfig failed: %v", err)
	}

	if cfg.Defaults.BackupLocation != "/test/backups" {
		t.Errorf("expected backup_location /test/backups, got %s", cfg.Defaults.BackupLocation)
	}

	if cfg.Servers[0].Connection.Host != "test.example.com" {
		t.Errorf("expected host test.example.com, got %s", cfg.Servers[0].Connection.Host)
	}
}
