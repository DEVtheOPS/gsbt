// internal/config/types.go
package config

// Config is the root configuration structure
type Config struct {
	Defaults Defaults `yaml:"defaults"`
	Servers  []Server `yaml:"servers"`
}

// Defaults holds default values for all servers
type Defaults struct {
	BackupLocation string `yaml:"backup_location"`
	TempDir        string `yaml:"temp_dir"`
	PruneAge       int    `yaml:"prune_age"`
	RetryAttempts  int    `yaml:"retry_attempts"`
	RetryDelay     int    `yaml:"retry_delay"`
	RetryBackoff   bool   `yaml:"retry_backoff"`
	EnvFile        string `yaml:"env_file"`
	NitradoAPIKey  string `yaml:"nitrado_api_key"`
}

// Server represents a single gameserver configuration
type Server struct {
	Name           string     `yaml:"name"`
	Description    string     `yaml:"description"`
	BackupLocation string     `yaml:"backup_location"`
	PruneAge       int        `yaml:"prune_age"`
	Connection     Connection `yaml:"connection"`
}

// Connection holds connector-specific configuration
type Connection struct {
	Type       string   `yaml:"type"`
	Host       string   `yaml:"host"`
	Port       int      `yaml:"port"`
	Username   string   `yaml:"username"`
	Password   string   `yaml:"password"`
	KeyFile    string   `yaml:"key_file"`
	Passive    *bool    `yaml:"passive"`
	TLS        bool     `yaml:"tls"`
	APIKey     string   `yaml:"api_key"`
	ServiceID  string   `yaml:"service_id"`
	RemotePath string   `yaml:"remote_path"`
	Include    []string `yaml:"include"`
	Exclude    []string `yaml:"exclude"`
}

// GetBackupLocation returns server-specific or default backup location
func (s *Server) GetBackupLocation(defaults Defaults) string {
	if s.BackupLocation != "" {
		return s.BackupLocation
	}
	return defaults.BackupLocation
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
