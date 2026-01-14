// internal/config/envsubst_test.go
package config

import (
	"os"
	"testing"
)

func TestExpandEnvVars(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		envVars  map[string]string
		expected string
	}{
		{
			name:     "simple brace syntax",
			input:    "password: ${PASSWORD}",
			envVars:  map[string]string{"PASSWORD": "secret123"},
			expected: "password: secret123",
		},
		{
			name:     "simple dollar syntax",
			input:    "user: $USERNAME",
			envVars:  map[string]string{"USERNAME": "admin"},
			expected: "user: admin",
		},
		{
			name:     "default value used when var missing",
			input:    "key: ${API_KEY:-default_key}",
			envVars:  map[string]string{},
			expected: "key: default_key",
		},
		{
			name:     "default value ignored when var exists",
			input:    "key: ${API_KEY:-default_key}",
			envVars:  map[string]string{"API_KEY": "real_key"},
			expected: "key: real_key",
		},
		{
			name:     "multiple substitutions",
			input:    "host: ${HOST}, user: ${USER}",
			envVars:  map[string]string{"HOST": "localhost", "USER": "admin"},
			expected: "host: localhost, user: admin",
		},
		{
			name:     "mixed syntax",
			input:    "connect: $USER@${HOST}:${PORT:-22}",
			envVars:  map[string]string{"USER": "admin", "HOST": "192.168.1.1"},
			expected: "connect: admin@192.168.1.1:22",
		},
		{
			name:     "no substitution needed",
			input:    "plain: text",
			envVars:  map[string]string{},
			expected: "plain: text",
		},
		{
			name:     "empty string when var missing and no default",
			input:    "value: ${MISSING}",
			envVars:  map[string]string{},
			expected: "value: ",
		},
		{
			name:     "preserve literal dollar sign not followed by var",
			input:    "price: $5.00",
			envVars:  map[string]string{},
			expected: "price: $5.00",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Set up environment
			for k, v := range tt.envVars {
				os.Setenv(k, v)
				defer os.Unsetenv(k)
			}

			result := ExpandEnvVars(tt.input)
			if result != tt.expected {
				t.Errorf("ExpandEnvVars(%q) = %q, want %q", tt.input, result, tt.expected)
			}
		})
	}
}

func TestExpandEnvVarsInConfig(t *testing.T) {
	// Set up test environment
	os.Setenv("TEST_BACKUP_LOCATION", "/srv/backups")
	os.Setenv("TEST_FTP_PASSWORD", "ftp_pass123")
	os.Setenv("TEST_NITRADO_KEY", "nitrado_key_456")
	defer os.Unsetenv("TEST_BACKUP_LOCATION")
	defer os.Unsetenv("TEST_FTP_PASSWORD")
	defer os.Unsetenv("TEST_NITRADO_KEY")

	cfg := &Config{
		Defaults: Defaults{
			BackupLocation: "${TEST_BACKUP_LOCATION}",
			TempDir:        "${TEST_TEMP_DIR:-/tmp}",
			PruneAge:       30,
			NitradoAPIKey:  "${TEST_NITRADO_KEY}",
		},
		Servers: []Server{
			{
				Name:           "test-server",
				BackupLocation: "${TEST_BACKUP_LOCATION}/test",
				Connection: Connection{
					Type:     "ftp",
					Host:     "localhost",
					Username: "user",
					Password: "${TEST_FTP_PASSWORD}",
				},
			},
		},
	}

	ExpandEnvVarsInConfig(cfg)

	// Check defaults expanded correctly
	if cfg.Defaults.BackupLocation != "/srv/backups" {
		t.Errorf("expected BackupLocation=/srv/backups, got %s", cfg.Defaults.BackupLocation)
	}
	if cfg.Defaults.TempDir != "/tmp" {
		t.Errorf("expected TempDir=/tmp, got %s", cfg.Defaults.TempDir)
	}
	if cfg.Defaults.NitradoAPIKey != "nitrado_key_456" {
		t.Errorf("expected NitradoAPIKey=nitrado_key_456, got %s", cfg.Defaults.NitradoAPIKey)
	}

	// Check server config expanded correctly
	if cfg.Servers[0].BackupLocation != "/srv/backups/test" {
		t.Errorf("expected server BackupLocation=/srv/backups/test, got %s", cfg.Servers[0].BackupLocation)
	}
	if cfg.Servers[0].Connection.Password != "ftp_pass123" {
		t.Errorf("expected server Password=ftp_pass123, got %s", cfg.Servers[0].Connection.Password)
	}
}
