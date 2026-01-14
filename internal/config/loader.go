// internal/config/loader.go
package config

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/joho/godotenv"
	"gopkg.in/yaml.v3"
)

// FindConfigFile locates the config file using discovery order:
// 1. Explicit path (from --config flag)
// 2. GSBT_CONFIG env var
// 3. ./.gsbt-config.yml (current directory)
// 4. ~/.config/gsbt/config.yml (user config)
func FindConfigFile(explicit string) (string, error) {
	// 1. Explicit path
	if explicit != "" {
		if _, err := os.Stat(explicit); err != nil {
			return "", fmt.Errorf("config file not found: %s", explicit)
		}
		return explicit, nil
	}

	// 2. GSBT_CONFIG env var
	if envPath := os.Getenv("GSBT_CONFIG"); envPath != "" {
		if _, err := os.Stat(envPath); err == nil {
			return envPath, nil
		}
	}

	// 3. Current directory
	localPath := ".gsbt-config.yml"
	if _, err := os.Stat(localPath); err == nil {
		abs, _ := filepath.Abs(localPath)
		return abs, nil
	}

	// 4. User config directory
	homeDir, err := os.UserHomeDir()
	if err == nil {
		userPath := filepath.Join(homeDir, ".config", "gsbt", "config.yml")
		if _, err := os.Stat(userPath); err == nil {
			return userPath, nil
		}
	}

	return "", fmt.Errorf("no config file found")
}

// LoadConfig loads and parses the config file
func LoadConfig(path string) (*Config, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read config: %w", err)
	}

	var cfg Config
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse config: %w", err)
	}

	// Load env file if specified
	if cfg.Defaults.EnvFile != "" {
		envPath := ExpandEnvVars(cfg.Defaults.EnvFile)
		if err := godotenv.Load(envPath); err != nil {
			// Non-fatal: env file is optional
		}
	}

	// Expand environment variables
	ExpandEnvVarsInConfig(&cfg)

	// Apply defaults
	applyDefaults(&cfg)

	return &cfg, nil
}

func applyDefaults(cfg *Config) {
	if cfg.Defaults.RetryAttempts == 0 {
		cfg.Defaults.RetryAttempts = 3
	}
	if cfg.Defaults.RetryDelay == 0 {
		cfg.Defaults.RetryDelay = 5
	}
	if cfg.Defaults.PruneAge == 0 {
		cfg.Defaults.PruneAge = 30
	}
}
