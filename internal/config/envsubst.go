// internal/config/envsubst.go
package config

import (
	"os"
	"regexp"
)

var (
	// envVarBraceRegex matches ${VAR} and ${VAR:-default} patterns
	envVarBraceRegex = regexp.MustCompile(`\$\{([A-Z_][A-Z0-9_]*)(:-([^}]*))?\}`)
	// envVarSimpleRegex matches $VAR pattern
	envVarSimpleRegex = regexp.MustCompile(`\$([A-Z_][A-Z0-9_]*)`)
)

// ExpandEnvVars expands environment variables in a string.
// Supports three patterns:
//   - ${VAR}          - replaced with value of VAR, empty string if not set
//   - ${VAR:-default} - replaced with value of VAR, or "default" if not set
//   - $VAR            - replaced with value of VAR, empty string if not set
func ExpandEnvVars(s string) string {
	// First, handle ${VAR} and ${VAR:-default} patterns
	result := envVarBraceRegex.ReplaceAllStringFunc(s, func(match string) string {
		// Extract variable name and default value
		matches := envVarBraceRegex.FindStringSubmatch(match)
		if len(matches) < 2 {
			return match
		}

		varName := matches[1]
		defaultValue := ""
		if len(matches) >= 4 {
			defaultValue = matches[3]
		}

		// Get environment variable value
		if value, exists := os.LookupEnv(varName); exists {
			return value
		}

		return defaultValue
	})

	// Then, handle simple $VAR patterns (but not $5.00 style)
	result = envVarSimpleRegex.ReplaceAllStringFunc(result, func(match string) string {
		matches := envVarSimpleRegex.FindStringSubmatch(match)
		if len(matches) < 2 {
			return match
		}

		varName := matches[1]
		if value, exists := os.LookupEnv(varName); exists {
			return value
		}

		return ""
	})

	return result
}

// ExpandEnvVarsInConfig expands all environment variables in a Config struct
func ExpandEnvVarsInConfig(cfg *Config) {
	// Expand defaults
	cfg.Defaults.BackupLocation = ExpandEnvVars(cfg.Defaults.BackupLocation)
	cfg.Defaults.TempDir = ExpandEnvVars(cfg.Defaults.TempDir)
	cfg.Defaults.EnvFile = ExpandEnvVars(cfg.Defaults.EnvFile)
	cfg.Defaults.NitradoAPIKey = ExpandEnvVars(cfg.Defaults.NitradoAPIKey)

	// Expand each server
	for i := range cfg.Servers {
		server := &cfg.Servers[i]
		server.Name = ExpandEnvVars(server.Name)
		server.Description = ExpandEnvVars(server.Description)
		server.BackupLocation = ExpandEnvVars(server.BackupLocation)

		// Expand connection fields
		conn := &server.Connection
		conn.Type = ExpandEnvVars(conn.Type)
		conn.Host = ExpandEnvVars(conn.Host)
		conn.Username = ExpandEnvVars(conn.Username)
		conn.Password = ExpandEnvVars(conn.Password)
		conn.KeyFile = ExpandEnvVars(conn.KeyFile)
		conn.APIKey = ExpandEnvVars(conn.APIKey)
		conn.ServiceID = ExpandEnvVars(conn.ServiceID)
		conn.RemotePath = ExpandEnvVars(conn.RemotePath)

		// Expand include/exclude patterns
		for j := range conn.Include {
			conn.Include[j] = ExpandEnvVars(conn.Include[j])
		}
		for j := range conn.Exclude {
			conn.Exclude[j] = ExpandEnvVars(conn.Exclude[j])
		}
	}
}
