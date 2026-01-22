// internal/config/types.go
package config

import "path/filepath"

//go:generate go run ../../cmd/schema-gen/main.go ../../gsbt.schema.json

// Config is the root configuration structure
type Config struct {
	Defaults Defaults `yaml:"defaults,omitempty"`
	Servers  []Server `yaml:"servers"`
}

// Defaults holds default values for all servers
type Defaults struct {
	BackupLocation string `yaml:"backup_location,omitempty"`
	TempDir        string `yaml:"temp_dir,omitempty"`
	PruneAge       int    `yaml:"prune_age,omitempty"`
	RetryAttempts  int    `yaml:"retry_attempts,omitempty"`
	RetryDelay     int    `yaml:"retry_delay,omitempty"`
	RetryBackoff   bool   `yaml:"retry_backoff,omitempty"`
	EnvFile        string `yaml:"env_file,omitempty"`
	NitradoAPIKey  string `yaml:"nitrado_api_key,omitempty"`
}

// Server represents a single gameserver configuration
type Server struct {
	Name           string     `yaml:"name"`
	Description    string     `yaml:"description,omitempty"`
	BackupLocation string     `yaml:"backup_location,omitempty"`
	PruneAge       int        `yaml:"prune_age,omitempty"`
	Connection     Connection `yaml:"connection"`
}

// Connection holds connector-specific configuration
type Connection struct {
	Type       string   `yaml:"type"`
	Host       string   `yaml:"host,omitempty"`
	Port       int      `yaml:"port,omitempty"`
	Username   string   `yaml:"username,omitempty"`
	Password   string   `yaml:"password,omitempty"`
	KeyFile    string   `yaml:"key_file,omitempty"`
	Passive    *bool    `yaml:"passive,omitempty"`
	TLS        bool     `yaml:"tls,omitempty"`
	APIKey     string   `yaml:"api_key,omitempty"`
	ServiceID  string   `yaml:"service_id,omitempty"`
	RemotePath string   `yaml:"remote_path,omitempty"`
	Include    []string `yaml:"include,omitempty"`
	Exclude    []string `yaml:"exclude,omitempty"`
}

// GetBackupLocation returns server-specific location, or the default with the server name appended
func (s *Server) GetBackupLocation(defaults Defaults) string {
	if s.BackupLocation != "" {
		return s.BackupLocation
	}
	if defaults.BackupLocation == "" {
		return ""
	}
	if s.Name == "" {
		return defaults.BackupLocation
	}
	return filepath.Join(defaults.BackupLocation, s.Name)
}

// GetPruneAge returns server-specific or default prune age
func (s *Server) GetPruneAge(defaults Defaults) int {
	if s.PruneAge > 0 {
		return s.PruneAge
	}
	return defaults.PruneAge
}

// GetInclude returns include patterns or default ["*"]
func (c *Connection) GetInclude() []string {
	if len(c.Include) > 0 {
		return c.Include
	}
	return []string{"*"}
}

// IsPassive returns passive mode setting (default true)
func (c *Connection) IsPassive() bool {
	if c.Passive != nil {
		return *c.Passive
	}
	return true
}