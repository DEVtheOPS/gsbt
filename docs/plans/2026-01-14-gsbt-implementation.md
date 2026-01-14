# GSBT Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan in the current session with parallel subagents.

**Goal:** Build a modular gameserver backup CLI tool in Go with FTP, SFTP, and Nitrado connectors.

**Architecture:** CLI built with Cobra, connectors implement a common interface, config via YAML with env var substitution. Backups stored as timestamped .tar.gz archives. Parallel execution by default.

**Tech Stack:** Go 1.21+, Cobra (CLI), jlaffaye/ftp, pkg/sftp, yaml.v3, godotenv

---

## 🚨 CRITICAL: Test Requirements

**EVERY task MUST have passing tests before marking as complete:**

1. **Test-First Always:** Write tests BEFORE implementation code
2. **Verify Failure:** Run tests and confirm they FAIL before implementing
3. **Verify Success:** Run tests and confirm they PASS after implementing
4. **No Exceptions:** Do NOT mark a task complete if tests are failing
5. **Coverage Mandate:** All public functions must have test coverage

**Exit Criteria for Each Task:**
```bash
go test ./... -v        # Must show PASS for relevant packages
go build ./cmd/gsbt/... # Must compile without errors
```

**If tests fail:** Debug and fix. NEVER claim completion with failing tests.

---

## Epic 1: Project Setup

### Task 1.1: Initialize Go Module

**Files:**
- Create: `go.mod`
- Create: `go.sum`
- Create: `cmd/gsbt/main.go`

**Step 1: Initialize Go module**

```bash
cd /srv/gameserver-backup-tool
go mod init github.com/digitalfiz/gsbt
```

**Step 2: Create minimal main.go**

```go
// cmd/gsbt/main.go
package main

import "fmt"

func main() {
    fmt.Println("gsbt - gameserver backup tool")
}
```

**Step 3: Verify it compiles and runs**

```bash
go run cmd/gsbt/main.go
```

Expected: `gsbt - gameserver backup tool`

**Step 4: Commit**

```bash
git add go.mod cmd/
git commit -m "feat: initialize go module and main entry point"
```

---

### Task 1.2: Add Core Dependencies

**Files:**
- Modify: `go.mod`

**Step 1: Add dependencies**

```bash
go get github.com/spf13/cobra@latest
go get gopkg.in/yaml.v3@latest
go get github.com/joho/godotenv@latest
go get github.com/jlaffaye/ftp@latest
go get github.com/pkg/sftp@latest
go get golang.org/x/crypto/ssh@latest
```

**Step 2: Tidy modules**

```bash
go mod tidy
```

**Step 3: Verify dependencies**

```bash
go list -m all | head -10
```

**Step 4: Commit**

```bash
git add go.mod go.sum
git commit -m "feat: add core dependencies"
```

---

## Epic 2: CLI Framework

### Task 2.1: Setup Cobra Root Command

**Files:**
- Create: `internal/cli/root.go`
- Modify: `cmd/gsbt/main.go`

**Step 1: Create root command with global flags**

```go
// internal/cli/root.go
package cli

import (
    "os"

    "github.com/spf13/cobra"
)

var (
    cfgFile    string
    outputFmt  string
    verbose    bool
    quiet      bool
)

var rootCmd = &cobra.Command{
    Use:   "gsbt",
    Short: "Gameserver Backup Tool",
    Long:  `A modular backup tool for game servers supporting FTP, SFTP, and Nitrado.`,
}

func Execute() {
    err := rootCmd.Execute()
    if err != nil {
        os.Exit(1)
    }
}

func init() {
    rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "", "config file path")
    rootCmd.PersistentFlags().StringVar(&outputFmt, "output", "text", "output format (text, json)")
    rootCmd.PersistentFlags().BoolVarP(&verbose, "verbose", "v", false, "verbose output")
    rootCmd.PersistentFlags().BoolVarP(&quiet, "quiet", "q", false, "suppress non-error output")
}

// GetConfigFile returns the config file path from flags
func GetConfigFile() string {
    return cfgFile
}

// GetOutputFormat returns the output format
func GetOutputFormat() string {
    return outputFmt
}

// IsVerbose returns verbose flag state
func IsVerbose() bool {
    return verbose
}

// IsQuiet returns quiet flag state
func IsQuiet() bool {
    return quiet
}
```

**Step 2: Update main.go**

```go
// cmd/gsbt/main.go
package main

import "github.com/digitalfiz/gsbt/internal/cli"

func main() {
    cli.Execute()
}
```

**Step 3: Verify CLI runs**

```bash
go run cmd/gsbt/main.go --help
```

Expected: Shows help with global flags

**Step 4: Commit**

```bash
git add internal/cli/root.go cmd/gsbt/main.go
git commit -m "feat: setup cobra root command with global flags"
```

---

### Task 2.2: Add Version Command

**Files:**
- Create: `internal/cli/version.go`
- Create: `internal/version/version.go`

**Step 1: Create version package**

```go
// internal/version/version.go
package version

var (
    Version   = "dev"
    Commit    = "none"
    BuildDate = "unknown"
)
```

**Step 2: Create version command**

```go
// internal/cli/version.go
package cli

import (
    "encoding/json"
    "fmt"

    "github.com/digitalfiz/gsbt/internal/version"
    "github.com/spf13/cobra"
)

var versionCmd = &cobra.Command{
    Use:   "version",
    Short: "Print version information",
    Run: func(cmd *cobra.Command, args []string) {
        if GetOutputFormat() == "json" {
            data := map[string]string{
                "version":    version.Version,
                "commit":     version.Commit,
                "build_date": version.BuildDate,
            }
            out, _ := json.MarshalIndent(data, "", "  ")
            fmt.Println(string(out))
        } else {
            fmt.Printf("gsbt %s\n", version.Version)
            fmt.Printf("  commit: %s\n", version.Commit)
            fmt.Printf("  built:  %s\n", version.BuildDate)
        }
    },
}

func init() {
    rootCmd.AddCommand(versionCmd)
}
```

**Step 3: Verify version command**

```bash
go run cmd/gsbt/main.go version
go run cmd/gsbt/main.go --output=json version
```

**Step 4: Commit**

```bash
git add internal/version/ internal/cli/version.go
git commit -m "feat: add version command"
```

---

### Task 2.3: Add Command Stubs

**Files:**
- Create: `internal/cli/backup.go`
- Create: `internal/cli/prune.go`
- Create: `internal/cli/list.go`
- Create: `internal/cli/restore.go`

**Step 1: Create backup command stub**

```go
// internal/cli/backup.go
package cli

import (
    "fmt"

    "github.com/spf13/cobra"
)

var (
    backupServer     string
    backupSequential bool
)

var backupCmd = &cobra.Command{
    Use:   "backup",
    Short: "Backup gameserver files",
    Long:  `Download and archive files from configured gameservers.`,
    Run: func(cmd *cobra.Command, args []string) {
        fmt.Println("backup command - not yet implemented")
    },
}

func init() {
    backupCmd.Flags().StringVar(&backupServer, "server", "", "backup specific server only")
    backupCmd.Flags().BoolVar(&backupSequential, "sequential", false, "run backups sequentially")
    rootCmd.AddCommand(backupCmd)
}
```

**Step 2: Create prune command stub**

```go
// internal/cli/prune.go
package cli

import (
    "fmt"

    "github.com/spf13/cobra"
)

var (
    pruneServer string
    pruneDryRun bool
)

var pruneCmd = &cobra.Command{
    Use:   "prune",
    Short: "Remove old backups",
    Long:  `Delete backups older than the configured prune_age.`,
    Run: func(cmd *cobra.Command, args []string) {
        fmt.Println("prune command - not yet implemented")
    },
}

func init() {
    pruneCmd.Flags().StringVar(&pruneServer, "server", "", "prune specific server only")
    pruneCmd.Flags().BoolVar(&pruneDryRun, "dry-run", false, "show what would be deleted")
    rootCmd.AddCommand(pruneCmd)
}
```

**Step 3: Create list command stub**

```go
// internal/cli/list.go
package cli

import (
    "fmt"

    "github.com/spf13/cobra"
)

var listServer string

var listCmd = &cobra.Command{
    Use:   "list",
    Short: "List configured servers",
    Long:  `Show configured servers and their backup status.`,
    Run: func(cmd *cobra.Command, args []string) {
        fmt.Println("list command - not yet implemented")
    },
}

func init() {
    listCmd.Flags().StringVar(&listServer, "server", "", "show specific server details")
    rootCmd.AddCommand(listCmd)
}
```

**Step 4: Create restore command stub**

```go
// internal/cli/restore.go
package cli

import (
    "fmt"

    "github.com/spf13/cobra"
)

var (
    restoreServer string
    restoreLocal  string
    restoreDryRun bool
    restoreForce  bool
)

var restoreCmd = &cobra.Command{
    Use:   "restore <backup-file>",
    Short: "Restore a backup",
    Long:  `Restore a backup to a server or extract locally.`,
    Args:  cobra.ExactArgs(1),
    Run: func(cmd *cobra.Command, args []string) {
        fmt.Printf("restore command - not yet implemented (file: %s)\n", args[0])
    },
}

func init() {
    restoreCmd.Flags().StringVar(&restoreServer, "server", "", "restore to server")
    restoreCmd.Flags().StringVar(&restoreLocal, "local", "", "extract to local path")
    restoreCmd.Flags().BoolVar(&restoreDryRun, "dry-run", false, "show what would be restored")
    restoreCmd.Flags().BoolVar(&restoreForce, "force", false, "skip confirmation prompt")
    rootCmd.AddCommand(restoreCmd)
}
```

**Step 5: Verify all commands**

```bash
go run cmd/gsbt/main.go --help
go run cmd/gsbt/main.go backup --help
go run cmd/gsbt/main.go prune --help
go run cmd/gsbt/main.go list --help
go run cmd/gsbt/main.go restore --help
```

**Step 6: Commit**

```bash
git add internal/cli/backup.go internal/cli/prune.go internal/cli/list.go internal/cli/restore.go
git commit -m "feat: add command stubs for backup, prune, list, restore"
```

---

## Epic 3: Config System

### Task 3.1: Define Config Structs

**Files:**
- Create: `internal/config/types.go`
- Create: `internal/config/types_test.go`

**Step 1: Write test for config struct**

```go
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
```

**Step 2: Run test to verify it fails**

```bash
go test ./internal/config/... -v
```

Expected: FAIL (types not defined)

**Step 3: Create config types**

```go
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
```

**Step 4: Run test to verify it passes**

```bash
go test ./internal/config/... -v
```

Expected: PASS

**Step 5: Commit**

```bash
git add internal/config/
git commit -m "feat: define config structs with yaml parsing"
```

---

### Task 3.2: Environment Variable Substitution

**Files:**
- Create: `internal/config/envsubst.go`
- Create: `internal/config/envsubst_test.go`

**Step 1: Write test for env substitution**

```go
// internal/config/envsubst_test.go
package config

import (
    "os"
    "testing"
)

func TestExpandEnvVars(t *testing.T) {
    os.Setenv("TEST_VAR", "test_value")
    os.Setenv("ANOTHER_VAR", "another")
    defer os.Unsetenv("TEST_VAR")
    defer os.Unsetenv("ANOTHER_VAR")

    tests := []struct {
        input    string
        expected string
    }{
        {"${TEST_VAR}", "test_value"},
        {"$TEST_VAR", "test_value"},
        {"prefix_${TEST_VAR}_suffix", "prefix_test_value_suffix"},
        {"${UNSET_VAR:-default}", "default"},
        {"${TEST_VAR:-default}", "test_value"},
        {"${UNSET_VAR}", ""},
        {"no variables", "no variables"},
        {"${TEST_VAR}_${ANOTHER_VAR}", "test_value_another"},
    }

    for _, tt := range tests {
        result := ExpandEnvVars(tt.input)
        if result != tt.expected {
            t.Errorf("ExpandEnvVars(%q) = %q, want %q", tt.input, result, tt.expected)
        }
    }
}
```

**Step 2: Run test to verify it fails**

```bash
go test ./internal/config/... -v -run TestExpandEnvVars
```

Expected: FAIL (function not defined)

**Step 3: Implement env substitution**

```go
// internal/config/envsubst.go
package config

import (
    "os"
    "regexp"
    "strings"
)

var (
    // Matches ${VAR} or ${VAR:-default}
    envVarBraceRegex = regexp.MustCompile(`\$\{([^}:]+)(?::-([^}]*))?\}`)
    // Matches $VAR (not followed by {)
    envVarSimpleRegex = regexp.MustCompile(`\$([A-Za-z_][A-Za-z0-9_]*)`)
)

// ExpandEnvVars expands environment variables in a string
// Supports: ${VAR}, $VAR, and ${VAR:-default}
func ExpandEnvVars(s string) string {
    // First expand ${VAR} and ${VAR:-default} patterns
    result := envVarBraceRegex.ReplaceAllStringFunc(s, func(match string) string {
        parts := envVarBraceRegex.FindStringSubmatch(match)
        if len(parts) < 2 {
            return match
        }

        varName := parts[1]
        defaultVal := ""
        if len(parts) >= 3 {
            defaultVal = parts[2]
        }

        if val, exists := os.LookupEnv(varName); exists {
            return val
        }
        return defaultVal
    })

    // Then expand simple $VAR patterns (but not if already expanded)
    result = envVarSimpleRegex.ReplaceAllStringFunc(result, func(match string) string {
        varName := strings.TrimPrefix(match, "$")
        if val, exists := os.LookupEnv(varName); exists {
            return val
        }
        return match
    })

    return result
}

// ExpandEnvVarsInConfig recursively expands env vars in all string fields
func ExpandEnvVarsInConfig(cfg *Config) {
    cfg.Defaults.BackupLocation = ExpandEnvVars(cfg.Defaults.BackupLocation)
    cfg.Defaults.TempDir = ExpandEnvVars(cfg.Defaults.TempDir)
    cfg.Defaults.EnvFile = ExpandEnvVars(cfg.Defaults.EnvFile)
    cfg.Defaults.NitradoAPIKey = ExpandEnvVars(cfg.Defaults.NitradoAPIKey)

    for i := range cfg.Servers {
        cfg.Servers[i].BackupLocation = ExpandEnvVars(cfg.Servers[i].BackupLocation)
        cfg.Servers[i].Connection.Host = ExpandEnvVars(cfg.Servers[i].Connection.Host)
        cfg.Servers[i].Connection.Username = ExpandEnvVars(cfg.Servers[i].Connection.Username)
        cfg.Servers[i].Connection.Password = ExpandEnvVars(cfg.Servers[i].Connection.Password)
        cfg.Servers[i].Connection.KeyFile = ExpandEnvVars(cfg.Servers[i].Connection.KeyFile)
        cfg.Servers[i].Connection.APIKey = ExpandEnvVars(cfg.Servers[i].Connection.APIKey)
        cfg.Servers[i].Connection.RemotePath = ExpandEnvVars(cfg.Servers[i].Connection.RemotePath)
    }
}
```

**Step 4: Run test to verify it passes**

```bash
go test ./internal/config/... -v -run TestExpandEnvVars
```

Expected: PASS

**Step 5: Commit**

```bash
git add internal/config/envsubst.go internal/config/envsubst_test.go
git commit -m "feat: add environment variable substitution"
```

---

### Task 3.3: Config Loading and Discovery

**Files:**
- Create: `internal/config/loader.go`
- Create: `internal/config/loader_test.go`

**Step 1: Write test for config discovery**

```go
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
}
```

**Step 2: Run test to verify it fails**

```bash
go test ./internal/config/... -v -run "TestFindConfigFile|TestLoadConfig"
```

Expected: FAIL

**Step 3: Implement config loader**

```go
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
```

**Step 4: Run tests to verify they pass**

```bash
go test ./internal/config/... -v
```

Expected: PASS

**Step 5: Commit**

```bash
git add internal/config/loader.go internal/config/loader_test.go
git commit -m "feat: add config loading and discovery"
```

---

## Epic 4: Connector System

### Task 4.1: Define Connector Interface

**Files:**
- Create: `internal/connector/connector.go`

**Step 1: Create connector interface**

```go
// internal/connector/connector.go
package connector

import (
    "context"
    "io"
    "time"
)

// FileInfo represents a remote file
type FileInfo struct {
    Path    string
    Size    int64
    ModTime time.Time
    IsDir   bool
}

// Connector defines the interface for all backup connectors
type Connector interface {
    // Connect establishes connection to the remote server
    Connect(ctx context.Context) error

    // List returns files matching patterns at remote_path
    List(ctx context.Context) ([]FileInfo, error)

    // Download retrieves a file and writes to the provided writer
    Download(ctx context.Context, remotePath string, w io.Writer) error

    // Upload reads from reader and writes to remote path
    Upload(ctx context.Context, r io.Reader, remotePath string) error

    // Close terminates the connection
    Close() error

    // Name returns a human-readable name for logging
    Name() string
}

// Config holds common connector configuration
type Config struct {
    Type       string
    Host       string
    Port       int
    Username   string
    Password   string
    KeyFile    string
    Passive    bool
    TLS        bool
    APIKey     string
    ServiceID  string
    RemotePath string
    Include    []string
    Exclude    []string

    // Retry settings
    RetryAttempts int
    RetryDelay    int
    RetryBackoff  bool
}
```

**Step 2: Verify it compiles**

```bash
go build ./internal/connector/...
```

**Step 3: Commit**

```bash
git add internal/connector/connector.go
git commit -m "feat: define connector interface"
```

---

### Task 4.2: Implement File Matching

**Files:**
- Create: `internal/connector/matcher.go`
- Create: `internal/connector/matcher_test.go`

**Step 1: Write test for file matching**

```go
// internal/connector/matcher_test.go
package connector

import "testing"

func TestMatchesPatterns(t *testing.T) {
    tests := []struct {
        path     string
        include  []string
        exclude  []string
        expected bool
    }{
        // Include all, no exclude
        {"file.txt", []string{"*"}, nil, true},

        // Include all, exclude logs
        {"file.txt", []string{"*"}, []string{"*.log"}, true},
        {"debug.log", []string{"*"}, []string{"*.log"}, false},

        // Exclude directories
        {"Logs/debug.log", []string{"*"}, []string{"Logs/"}, false},
        {"saves/game.sav", []string{"*"}, []string{"Logs/"}, true},

        // Multiple excludes
        {"temp.tmp", []string{"*"}, []string{"*.log", "*.tmp"}, false},

        // Specific includes
        {"game.sav", []string{"*.sav"}, nil, true},
        {"config.ini", []string{"*.sav"}, nil, false},
    }

    for _, tt := range tests {
        result := MatchesPatterns(tt.path, tt.include, tt.exclude)
        if result != tt.expected {
            t.Errorf("MatchesPatterns(%q, %v, %v) = %v, want %v",
                tt.path, tt.include, tt.exclude, result, tt.expected)
        }
    }
}
```

**Step 2: Run test to verify it fails**

```bash
go test ./internal/connector/... -v -run TestMatchesPatterns
```

Expected: FAIL

**Step 3: Implement file matching**

```go
// internal/connector/matcher.go
package connector

import (
    "path/filepath"
    "strings"
)

// MatchesPatterns checks if a path matches include patterns and doesn't match exclude patterns
func MatchesPatterns(path string, include, exclude []string) bool {
    // Default include to ["*"]
    if len(include) == 0 {
        include = []string{"*"}
    }

    // Check excludes first
    for _, pattern := range exclude {
        // Directory pattern (ends with /)
        if strings.HasSuffix(pattern, "/") {
            dir := strings.TrimSuffix(pattern, "/")
            if strings.HasPrefix(path, dir+"/") || strings.HasPrefix(path, dir+"\\") {
                return false
            }
            if path == dir {
                return false
            }
        } else {
            // File pattern
            base := filepath.Base(path)
            if matched, _ := filepath.Match(pattern, base); matched {
                return false
            }
            // Also try matching the full path
            if matched, _ := filepath.Match(pattern, path); matched {
                return false
            }
        }
    }

    // Check includes
    for _, pattern := range include {
        if pattern == "*" {
            return true
        }
        base := filepath.Base(path)
        if matched, _ := filepath.Match(pattern, base); matched {
            return true
        }
        if matched, _ := filepath.Match(pattern, path); matched {
            return true
        }
    }

    return false
}
```

**Step 4: Run test to verify it passes**

```bash
go test ./internal/connector/... -v -run TestMatchesPatterns
```

Expected: PASS

**Step 5: Commit**

```bash
git add internal/connector/matcher.go internal/connector/matcher_test.go
git commit -m "feat: add file pattern matching for include/exclude"
```

---

### Task 4.3: Implement FTP Connector

**Files:**
- Create: `internal/connector/ftp.go`
- Create: `internal/connector/ftp_test.go`

**Step 1: Write test for FTP connector**

```go
// internal/connector/ftp_test.go
package connector

import (
    "testing"
)

func TestNewFTPConnector(t *testing.T) {
    cfg := Config{
        Type:       "ftp",
        Host:       "localhost",
        Port:       21,
        Username:   "user",
        Password:   "pass",
        RemotePath: "/saves/",
        Include:    []string{"*"},
        Exclude:    []string{"*.log"},
    }

    conn := NewFTPConnector(cfg)
    if conn == nil {
        t.Fatal("NewFTPConnector returned nil")
    }

    if conn.Name() != "ftp://localhost:21" {
        t.Errorf("unexpected name: %s", conn.Name())
    }
}
```

**Step 2: Run test to verify it fails**

```bash
go test ./internal/connector/... -v -run TestNewFTPConnector
```

Expected: FAIL

**Step 3: Implement FTP connector**

```go
// internal/connector/ftp.go
package connector

import (
    "context"
    "fmt"
    "io"
    "path"
    "time"

    "github.com/jlaffaye/ftp"
)

// FTPConnector implements Connector for FTP servers
type FTPConnector struct {
    config Config
    conn   *ftp.ServerConn
}

// NewFTPConnector creates a new FTP connector
func NewFTPConnector(cfg Config) *FTPConnector {
    if cfg.Port == 0 {
        cfg.Port = 21
    }
    return &FTPConnector{config: cfg}
}

// Name returns the connector name for logging
func (f *FTPConnector) Name() string {
    return fmt.Sprintf("ftp://%s:%d", f.config.Host, f.config.Port)
}

// Connect establishes FTP connection
func (f *FTPConnector) Connect(ctx context.Context) error {
    addr := fmt.Sprintf("%s:%d", f.config.Host, f.config.Port)

    var opts []ftp.DialOption
    opts = append(opts, ftp.DialWithContext(ctx))
    opts = append(opts, ftp.DialWithTimeout(30*time.Second))

    if f.config.TLS {
        opts = append(opts, ftp.DialWithExplicitTLS(nil))
    }

    conn, err := ftp.Dial(addr, opts...)
    if err != nil {
        return fmt.Errorf("failed to connect to FTP: %w", err)
    }

    if err := conn.Login(f.config.Username, f.config.Password); err != nil {
        conn.Quit()
        return fmt.Errorf("FTP login failed: %w", err)
    }

    f.conn = conn
    return nil
}

// List returns files at remote_path matching include/exclude patterns
func (f *FTPConnector) List(ctx context.Context) ([]FileInfo, error) {
    if f.conn == nil {
        return nil, fmt.Errorf("not connected")
    }

    var files []FileInfo
    err := f.walkDir(f.config.RemotePath, &files)
    if err != nil {
        return nil, err
    }

    // Filter by patterns
    var filtered []FileInfo
    for _, file := range files {
        if !file.IsDir {
            relPath := file.Path
            if MatchesPatterns(relPath, f.config.Include, f.config.Exclude) {
                filtered = append(filtered, file)
            }
        }
    }

    return filtered, nil
}

func (f *FTPConnector) walkDir(dir string, files *[]FileInfo) error {
    entries, err := f.conn.List(dir)
    if err != nil {
        return fmt.Errorf("failed to list %s: %w", dir, err)
    }

    for _, entry := range entries {
        if entry.Name == "." || entry.Name == ".." {
            continue
        }

        fullPath := path.Join(dir, entry.Name)
        relPath := fullPath
        if len(f.config.RemotePath) > 0 {
            relPath = fullPath[len(f.config.RemotePath):]
            if len(relPath) > 0 && relPath[0] == '/' {
                relPath = relPath[1:]
            }
        }

        info := FileInfo{
            Path:    relPath,
            Size:    int64(entry.Size),
            ModTime: entry.Time,
            IsDir:   entry.Type == ftp.EntryTypeFolder,
        }
        *files = append(*files, info)

        if entry.Type == ftp.EntryTypeFolder {
            if err := f.walkDir(fullPath, files); err != nil {
                return err
            }
        }
    }

    return nil
}

// Download retrieves a file from FTP
func (f *FTPConnector) Download(ctx context.Context, remotePath string, w io.Writer) error {
    if f.conn == nil {
        return fmt.Errorf("not connected")
    }

    fullPath := path.Join(f.config.RemotePath, remotePath)
    resp, err := f.conn.Retr(fullPath)
    if err != nil {
        return fmt.Errorf("failed to download %s: %w", remotePath, err)
    }
    defer resp.Close()

    _, err = io.Copy(w, resp)
    return err
}

// Upload sends a file to FTP
func (f *FTPConnector) Upload(ctx context.Context, r io.Reader, remotePath string) error {
    if f.conn == nil {
        return fmt.Errorf("not connected")
    }

    fullPath := path.Join(f.config.RemotePath, remotePath)

    // Ensure parent directory exists
    dir := path.Dir(fullPath)
    f.conn.MakeDir(dir) // Ignore error, may already exist

    return f.conn.Stor(fullPath, r)
}

// Close terminates the FTP connection
func (f *FTPConnector) Close() error {
    if f.conn != nil {
        return f.conn.Quit()
    }
    return nil
}
```

**Step 4: Run test to verify it passes**

```bash
go test ./internal/connector/... -v -run TestNewFTPConnector
```

Expected: PASS

**Step 5: Commit**

```bash
git add internal/connector/ftp.go internal/connector/ftp_test.go
git commit -m "feat: implement FTP connector"
```

---

### Task 4.4: Implement SFTP Connector

**Files:**
- Create: `internal/connector/sftp.go`
- Create: `internal/connector/sftp_test.go`

**Step 1: Write test for SFTP connector**

```go
// internal/connector/sftp_test.go
package connector

import (
    "testing"
)

func TestNewSFTPConnector(t *testing.T) {
    cfg := Config{
        Type:       "sftp",
        Host:       "localhost",
        Port:       22,
        Username:   "user",
        Password:   "pass",
        RemotePath: "/home/user/saves/",
        Include:    []string{"*"},
        Exclude:    []string{"*.log"},
    }

    conn := NewSFTPConnector(cfg)
    if conn == nil {
        t.Fatal("NewSFTPConnector returned nil")
    }

    if conn.Name() != "sftp://localhost:22" {
        t.Errorf("unexpected name: %s", conn.Name())
    }
}

func TestNewSFTPConnectorWithKeyFile(t *testing.T) {
    cfg := Config{
        Type:       "sftp",
        Host:       "localhost",
        Port:       22,
        Username:   "user",
        KeyFile:    "/home/user/.ssh/id_rsa",
        RemotePath: "/saves/",
    }

    conn := NewSFTPConnector(cfg)
    if conn == nil {
        t.Fatal("NewSFTPConnector returned nil")
    }
}
```

**Step 2: Run test to verify it fails**

```bash
go test ./internal/connector/... -v -run TestNewSFTPConnector
```

Expected: FAIL

**Step 3: Implement SFTP connector**

```go
// internal/connector/sftp.go
package connector

import (
    "context"
    "fmt"
    "io"
    "os"
    "path"
    "path/filepath"
    "time"

    "github.com/pkg/sftp"
    "golang.org/x/crypto/ssh"
)

// SFTPConnector implements Connector for SFTP servers
type SFTPConnector struct {
    config     Config
    sshClient  *ssh.Client
    sftpClient *sftp.Client
}

// NewSFTPConnector creates a new SFTP connector
func NewSFTPConnector(cfg Config) *SFTPConnector {
    if cfg.Port == 0 {
        cfg.Port = 22
    }
    return &SFTPConnector{config: cfg}
}

// Name returns the connector name for logging
func (s *SFTPConnector) Name() string {
    return fmt.Sprintf("sftp://%s:%d", s.config.Host, s.config.Port)
}

// Connect establishes SFTP connection
func (s *SFTPConnector) Connect(ctx context.Context) error {
    var authMethods []ssh.AuthMethod

    // Try key file authentication first
    if s.config.KeyFile != "" {
        key, err := os.ReadFile(s.config.KeyFile)
        if err != nil {
            return fmt.Errorf("failed to read key file: %w", err)
        }

        signer, err := ssh.ParsePrivateKey(key)
        if err != nil {
            return fmt.Errorf("failed to parse key file: %w", err)
        }

        authMethods = append(authMethods, ssh.PublicKeys(signer))
    }

    // Fall back to password authentication
    if s.config.Password != "" {
        authMethods = append(authMethods, ssh.Password(s.config.Password))
    }

    if len(authMethods) == 0 {
        return fmt.Errorf("no authentication method provided (need password or key_file)")
    }

    sshConfig := &ssh.ClientConfig{
        User:            s.config.Username,
        Auth:            authMethods,
        HostKeyCallback: ssh.InsecureIgnoreHostKey(), // TODO: proper host key verification
        Timeout:         30 * time.Second,
    }

    addr := fmt.Sprintf("%s:%d", s.config.Host, s.config.Port)
    sshClient, err := ssh.Dial("tcp", addr, sshConfig)
    if err != nil {
        return fmt.Errorf("failed to connect to SSH: %w", err)
    }
    s.sshClient = sshClient

    sftpClient, err := sftp.NewClient(sshClient)
    if err != nil {
        sshClient.Close()
        return fmt.Errorf("failed to create SFTP client: %w", err)
    }
    s.sftpClient = sftpClient

    return nil
}

// List returns files at remote_path matching include/exclude patterns
func (s *SFTPConnector) List(ctx context.Context) ([]FileInfo, error) {
    if s.sftpClient == nil {
        return nil, fmt.Errorf("not connected")
    }

    var files []FileInfo
    err := s.walkDir(s.config.RemotePath, &files)
    if err != nil {
        return nil, err
    }

    // Filter by patterns
    var filtered []FileInfo
    for _, file := range files {
        if !file.IsDir {
            if MatchesPatterns(file.Path, s.config.Include, s.config.Exclude) {
                filtered = append(filtered, file)
            }
        }
    }

    return filtered, nil
}

func (s *SFTPConnector) walkDir(dir string, files *[]FileInfo) error {
    entries, err := s.sftpClient.ReadDir(dir)
    if err != nil {
        return fmt.Errorf("failed to list %s: %w", dir, err)
    }

    for _, entry := range entries {
        if entry.Name() == "." || entry.Name() == ".." {
            continue
        }

        fullPath := filepath.Join(dir, entry.Name())
        relPath := fullPath
        if len(s.config.RemotePath) > 0 {
            relPath, _ = filepath.Rel(s.config.RemotePath, fullPath)
        }

        info := FileInfo{
            Path:    relPath,
            Size:    entry.Size(),
            ModTime: entry.ModTime(),
            IsDir:   entry.IsDir(),
        }
        *files = append(*files, info)

        if entry.IsDir() {
            if err := s.walkDir(fullPath, files); err != nil {
                return err
            }
        }
    }

    return nil
}

// Download retrieves a file from SFTP
func (s *SFTPConnector) Download(ctx context.Context, remotePath string, w io.Writer) error {
    if s.sftpClient == nil {
        return fmt.Errorf("not connected")
    }

    fullPath := path.Join(s.config.RemotePath, remotePath)
    f, err := s.sftpClient.Open(fullPath)
    if err != nil {
        return fmt.Errorf("failed to open %s: %w", remotePath, err)
    }
    defer f.Close()

    _, err = io.Copy(w, f)
    return err
}

// Upload sends a file to SFTP
func (s *SFTPConnector) Upload(ctx context.Context, r io.Reader, remotePath string) error {
    if s.sftpClient == nil {
        return fmt.Errorf("not connected")
    }

    fullPath := path.Join(s.config.RemotePath, remotePath)

    // Ensure parent directory exists
    dir := path.Dir(fullPath)
    s.sftpClient.MkdirAll(dir)

    f, err := s.sftpClient.Create(fullPath)
    if err != nil {
        return fmt.Errorf("failed to create %s: %w", remotePath, err)
    }
    defer f.Close()

    _, err = io.Copy(f, r)
    return err
}

// Close terminates the SFTP connection
func (s *SFTPConnector) Close() error {
    if s.sftpClient != nil {
        s.sftpClient.Close()
    }
    if s.sshClient != nil {
        s.sshClient.Close()
    }
    return nil
}
```

**Step 4: Run test to verify it passes**

```bash
go test ./internal/connector/... -v -run TestNewSFTPConnector
```

Expected: PASS

**Step 5: Commit**

```bash
git add internal/connector/sftp.go internal/connector/sftp_test.go
git commit -m "feat: implement SFTP connector"
```

---

### Task 4.5: Implement Nitrado Connector

**Files:**
- Create: `internal/connector/nitrado.go`
- Create: `internal/connector/nitrado_test.go`

**Step 1: Write test for Nitrado connector**

```go
// internal/connector/nitrado_test.go
package connector

import (
    "testing"
)

func TestNewNitradoConnector(t *testing.T) {
    cfg := Config{
        Type:       "nitrado",
        APIKey:     "test-api-key",
        ServiceID:  "12345",
        RemotePath: "/games/ark/",
        Include:    []string{"*"},
        Exclude:    []string{"*.log"},
    }

    conn := NewNitradoConnector(cfg)
    if conn == nil {
        t.Fatal("NewNitradoConnector returned nil")
    }

    if conn.Name() != "nitrado://12345" {
        t.Errorf("unexpected name: %s", conn.Name())
    }
}
```

**Step 2: Run test to verify it fails**

```bash
go test ./internal/connector/... -v -run TestNewNitradoConnector
```

Expected: FAIL

**Step 3: Implement Nitrado connector**

```go
// internal/connector/nitrado.go
package connector

import (
    "context"
    "encoding/json"
    "fmt"
    "io"
    "net/http"
    "time"
)

const nitradoAPIBase = "https://api.nitrado.net"

// NitradoConnector implements Connector for Nitrado game servers
// It fetches FTP credentials from Nitrado API and delegates to FTPConnector
type NitradoConnector struct {
    config    Config
    ftp       *FTPConnector
    apiKey    string
    serviceID string
}

// nitradoFTPResponse represents the Nitrado API response for FTP credentials
type nitradoFTPResponse struct {
    Status  string `json:"status"`
    Data    struct {
        FTP struct {
            Hostname string `json:"hostname"`
            Port     int    `json:"port"`
            Username string `json:"username"`
            Password string `json:"password"`
        } `json:"ftp"`
    } `json:"data"`
    Message string `json:"message"`
}

// NewNitradoConnector creates a new Nitrado connector
func NewNitradoConnector(cfg Config) *NitradoConnector {
    return &NitradoConnector{
        config:    cfg,
        apiKey:    cfg.APIKey,
        serviceID: cfg.ServiceID,
    }
}

// Name returns the connector name for logging
func (n *NitradoConnector) Name() string {
    return fmt.Sprintf("nitrado://%s", n.serviceID)
}

// Connect fetches FTP credentials from Nitrado API and establishes FTP connection
func (n *NitradoConnector) Connect(ctx context.Context) error {
    // Fetch FTP credentials from Nitrado API
    creds, err := n.fetchFTPCredentials(ctx)
    if err != nil {
        return fmt.Errorf("failed to get Nitrado FTP credentials: %w", err)
    }

    // Create FTP connector with retrieved credentials
    ftpConfig := Config{
        Type:       "ftp",
        Host:       creds.Hostname,
        Port:       creds.Port,
        Username:   creds.Username,
        Password:   creds.Password,
        RemotePath: n.config.RemotePath,
        Include:    n.config.Include,
        Exclude:    n.config.Exclude,
        Passive:    true,

        RetryAttempts: n.config.RetryAttempts,
        RetryDelay:    n.config.RetryDelay,
        RetryBackoff:  n.config.RetryBackoff,
    }

    n.ftp = NewFTPConnector(ftpConfig)
    return n.ftp.Connect(ctx)
}

type ftpCredentials struct {
    Hostname string
    Port     int
    Username string
    Password string
}

func (n *NitradoConnector) fetchFTPCredentials(ctx context.Context) (*ftpCredentials, error) {
    url := fmt.Sprintf("%s/services/%s/gameservers", nitradoAPIBase, n.serviceID)

    req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
    if err != nil {
        return nil, err
    }

    req.Header.Set("Authorization", "Bearer "+n.apiKey)
    req.Header.Set("Accept", "application/json")

    client := &http.Client{Timeout: 30 * time.Second}
    resp, err := client.Do(req)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()

    // Handle rate limiting
    if resp.StatusCode == 429 {
        retryAfter := resp.Header.Get("Retry-After")
        return nil, fmt.Errorf("rate limited by Nitrado API (retry after: %s)", retryAfter)
    }

    if resp.StatusCode != 200 {
        body, _ := io.ReadAll(resp.Body)
        return nil, fmt.Errorf("Nitrado API error (status %d): %s", resp.StatusCode, string(body))
    }

    var apiResp nitradoFTPResponse
    if err := json.NewDecoder(resp.Body).Decode(&apiResp); err != nil {
        return nil, fmt.Errorf("failed to parse Nitrado response: %w", err)
    }

    if apiResp.Status != "success" {
        return nil, fmt.Errorf("Nitrado API returned error: %s", apiResp.Message)
    }

    return &ftpCredentials{
        Hostname: apiResp.Data.FTP.Hostname,
        Port:     apiResp.Data.FTP.Port,
        Username: apiResp.Data.FTP.Username,
        Password: apiResp.Data.FTP.Password,
    }, nil
}

// List delegates to FTP connector
func (n *NitradoConnector) List(ctx context.Context) ([]FileInfo, error) {
    if n.ftp == nil {
        return nil, fmt.Errorf("not connected")
    }
    return n.ftp.List(ctx)
}

// Download delegates to FTP connector
func (n *NitradoConnector) Download(ctx context.Context, remotePath string, w io.Writer) error {
    if n.ftp == nil {
        return fmt.Errorf("not connected")
    }
    return n.ftp.Download(ctx, remotePath, w)
}

// Upload delegates to FTP connector
func (n *NitradoConnector) Upload(ctx context.Context, r io.Reader, remotePath string) error {
    if n.ftp == nil {
        return fmt.Errorf("not connected")
    }
    return n.ftp.Upload(ctx, r, remotePath)
}

// Close terminates the FTP connection
func (n *NitradoConnector) Close() error {
    if n.ftp != nil {
        return n.ftp.Close()
    }
    return nil
}
```

**Step 4: Run test to verify it passes**

```bash
go test ./internal/connector/... -v -run TestNewNitradoConnector
```

Expected: PASS

**Step 5: Commit**

```bash
git add internal/connector/nitrado.go internal/connector/nitrado_test.go
git commit -m "feat: implement Nitrado connector (FTP wrapper)"
```

---

### Task 4.6: Connector Factory

**Files:**
- Create: `internal/connector/factory.go`
- Create: `internal/connector/factory_test.go`

**Step 1: Write test for connector factory**

```go
// internal/connector/factory_test.go
package connector

import (
    "testing"
)

func TestNewConnector(t *testing.T) {
    tests := []struct {
        connType string
        wantType string
        wantErr  bool
    }{
        {"ftp", "*connector.FTPConnector", false},
        {"sftp", "*connector.SFTPConnector", false},
        {"nitrado", "*connector.NitradoConnector", false},
        {"unknown", "", true},
    }

    for _, tt := range tests {
        cfg := Config{
            Type:       tt.connType,
            Host:       "localhost",
            Username:   "user",
            Password:   "pass",
            RemotePath: "/",
        }

        conn, err := NewConnector(cfg)
        if tt.wantErr {
            if err == nil {
                t.Errorf("NewConnector(%q) expected error, got nil", tt.connType)
            }
        } else {
            if err != nil {
                t.Errorf("NewConnector(%q) unexpected error: %v", tt.connType, err)
            }
            if conn == nil {
                t.Errorf("NewConnector(%q) returned nil", tt.connType)
            }
        }
    }
}
```

**Step 2: Run test to verify it fails**

```bash
go test ./internal/connector/... -v -run TestNewConnector
```

Expected: FAIL

**Step 3: Implement connector factory**

```go
// internal/connector/factory.go
package connector

import "fmt"

// NewConnector creates a connector based on the config type
func NewConnector(cfg Config) (Connector, error) {
    switch cfg.Type {
    case "ftp":
        return NewFTPConnector(cfg), nil
    case "sftp":
        return NewSFTPConnector(cfg), nil
    case "nitrado":
        return NewNitradoConnector(cfg), nil
    default:
        return nil, fmt.Errorf("unknown connector type: %s", cfg.Type)
    }
}
```

**Step 4: Run test to verify it passes**

```bash
go test ./internal/connector/... -v -run TestNewConnector
```

Expected: PASS

**Step 5: Commit**

```bash
git add internal/connector/factory.go internal/connector/factory_test.go
git commit -m "feat: add connector factory"
```

---

## Epic 5: Backup System

### Task 5.1: Archive Creation

**Files:**
- Create: `internal/backup/archive.go`
- Create: `internal/backup/archive_test.go`

**Step 1: Write test for archive creation**

```go
// internal/backup/archive_test.go
package backup

import (
    "archive/tar"
    "compress/gzip"
    "os"
    "path/filepath"
    "testing"
)

func TestCreateArchive(t *testing.T) {
    // Create temp directory with test files
    tmpDir, err := os.MkdirTemp("", "gsbt-test")
    if err != nil {
        t.Fatal(err)
    }
    defer os.RemoveAll(tmpDir)

    // Create test files
    srcDir := filepath.Join(tmpDir, "source")
    os.MkdirAll(filepath.Join(srcDir, "subdir"), 0755)
    os.WriteFile(filepath.Join(srcDir, "file1.txt"), []byte("content1"), 0644)
    os.WriteFile(filepath.Join(srcDir, "subdir", "file2.txt"), []byte("content2"), 0644)

    // Create archive
    archivePath := filepath.Join(tmpDir, "test.tar.gz")
    err = CreateArchive(srcDir, archivePath)
    if err != nil {
        t.Fatalf("CreateArchive failed: %v", err)
    }

    // Verify archive exists
    if _, err := os.Stat(archivePath); err != nil {
        t.Fatalf("archive not created: %v", err)
    }

    // Verify archive contents
    f, _ := os.Open(archivePath)
    defer f.Close()
    gz, _ := gzip.NewReader(f)
    tr := tar.NewReader(gz)

    fileCount := 0
    for {
        _, err := tr.Next()
        if err != nil {
            break
        }
        fileCount++
    }

    if fileCount < 2 {
        t.Errorf("expected at least 2 files in archive, got %d", fileCount)
    }
}
```

**Step 2: Run test to verify it fails**

```bash
go test ./internal/backup/... -v -run TestCreateArchive
```

Expected: FAIL

**Step 3: Implement archive creation**

```go
// internal/backup/archive.go
package backup

import (
    "archive/tar"
    "compress/gzip"
    "fmt"
    "io"
    "os"
    "path/filepath"
)

// CreateArchive creates a .tar.gz archive from a source directory
func CreateArchive(srcDir, destPath string) error {
    // Create output file
    outFile, err := os.Create(destPath)
    if err != nil {
        return fmt.Errorf("failed to create archive file: %w", err)
    }
    defer outFile.Close()

    // Create gzip writer
    gzWriter := gzip.NewWriter(outFile)
    defer gzWriter.Close()

    // Create tar writer
    tarWriter := tar.NewWriter(gzWriter)
    defer tarWriter.Close()

    // Walk the source directory
    return filepath.Walk(srcDir, func(path string, info os.FileInfo, err error) error {
        if err != nil {
            return err
        }

        // Get relative path
        relPath, err := filepath.Rel(srcDir, path)
        if err != nil {
            return err
        }

        // Skip the root directory itself
        if relPath == "." {
            return nil
        }

        // Create tar header
        header, err := tar.FileInfoHeader(info, "")
        if err != nil {
            return err
        }
        header.Name = relPath

        // Write header
        if err := tarWriter.WriteHeader(header); err != nil {
            return err
        }

        // If it's a file, write contents
        if !info.IsDir() {
            file, err := os.Open(path)
            if err != nil {
                return err
            }
            defer file.Close()

            if _, err := io.Copy(tarWriter, file); err != nil {
                return err
            }
        }

        return nil
    })
}

// ExtractArchive extracts a .tar.gz archive to a destination directory
func ExtractArchive(archivePath, destDir string) error {
    // Open archive file
    file, err := os.Open(archivePath)
    if err != nil {
        return fmt.Errorf("failed to open archive: %w", err)
    }
    defer file.Close()

    // Create gzip reader
    gzReader, err := gzip.NewReader(file)
    if err != nil {
        return fmt.Errorf("failed to create gzip reader: %w", err)
    }
    defer gzReader.Close()

    // Create tar reader
    tarReader := tar.NewReader(gzReader)

    // Extract files
    for {
        header, err := tarReader.Next()
        if err == io.EOF {
            break
        }
        if err != nil {
            return fmt.Errorf("failed to read tar header: %w", err)
        }

        targetPath := filepath.Join(destDir, header.Name)

        switch header.Typeflag {
        case tar.TypeDir:
            if err := os.MkdirAll(targetPath, os.FileMode(header.Mode)); err != nil {
                return err
            }
        case tar.TypeReg:
            // Ensure parent directory exists
            if err := os.MkdirAll(filepath.Dir(targetPath), 0755); err != nil {
                return err
            }

            outFile, err := os.Create(targetPath)
            if err != nil {
                return err
            }

            if _, err := io.Copy(outFile, tarReader); err != nil {
                outFile.Close()
                return err
            }
            outFile.Close()

            if err := os.Chmod(targetPath, os.FileMode(header.Mode)); err != nil {
                return err
            }
        }
    }

    return nil
}
```

**Step 4: Run test to verify it passes**

```bash
go test ./internal/backup/... -v -run TestCreateArchive
```

Expected: PASS

**Step 5: Commit**

```bash
git add internal/backup/
git commit -m "feat: implement tar.gz archive creation and extraction"
```

---

### Task 5.2: Backup Orchestration

**Files:**
- Create: `internal/backup/backup.go`
- Modify: `internal/backup/backup_test.go`

**Step 1: Write test for backup manager**

```go
// internal/backup/backup_test.go (add to existing file)

func TestGenerateBackupPath(t *testing.T) {
    mgr := &Manager{
        BackupLocation: "/srv/backups",
    }

    path := mgr.GenerateBackupPath("test-server")

    // Should match pattern: /srv/backups/test-server/YYYY-MM-DD_HHMMSS.tar.gz
    if !filepath.IsAbs(path) {
        t.Errorf("expected absolute path, got %s", path)
    }

    if filepath.Ext(path) != ".gz" {
        t.Errorf("expected .gz extension, got %s", filepath.Ext(path))
    }
}
```

**Step 2: Run test to verify it fails**

```bash
go test ./internal/backup/... -v -run TestGenerateBackupPath
```

Expected: FAIL

**Step 3: Implement backup manager**

```go
// internal/backup/backup.go
package backup

import (
    "context"
    "fmt"
    "io"
    "os"
    "path/filepath"
    "time"

    "github.com/digitalfiz/gsbt/internal/connector"
)

// Manager handles backup operations
type Manager struct {
    BackupLocation string
    TempDir        string
}

// NewManager creates a new backup manager
func NewManager(backupLocation, tempDir string) *Manager {
    if tempDir == "" {
        tempDir = filepath.Join(backupLocation, ".tmp")
    }
    return &Manager{
        BackupLocation: backupLocation,
        TempDir:        tempDir,
    }
}

// GenerateBackupPath generates a timestamped backup path for a server
func (m *Manager) GenerateBackupPath(serverName string) string {
    timestamp := time.Now().UTC().Format("2006-01-02_150405")
    filename := fmt.Sprintf("%s.tar.gz", timestamp)
    return filepath.Join(m.BackupLocation, serverName, filename)
}

// BackupResult contains the result of a backup operation
type BackupResult struct {
    ServerName  string
    BackupPath  string
    FileCount   int
    TotalSize   int64
    Duration    time.Duration
    Error       error
}

// Backup performs a backup for a server using the provided connector
func (m *Manager) Backup(ctx context.Context, serverName string, conn connector.Connector) (*BackupResult, error) {
    start := time.Now()
    result := &BackupResult{
        ServerName: serverName,
    }

    // Create temp directory for this backup
    tempDir := filepath.Join(m.TempDir, serverName, fmt.Sprintf("%d", time.Now().UnixNano()))
    if err := os.MkdirAll(tempDir, 0755); err != nil {
        result.Error = fmt.Errorf("failed to create temp directory: %w", err)
        return result, result.Error
    }
    defer os.RemoveAll(tempDir)

    // Connect
    if err := conn.Connect(ctx); err != nil {
        result.Error = fmt.Errorf("connection failed: %w", err)
        return result, result.Error
    }
    defer conn.Close()

    // List files
    files, err := conn.List(ctx)
    if err != nil {
        result.Error = fmt.Errorf("failed to list files: %w", err)
        return result, result.Error
    }

    result.FileCount = len(files)

    // Download files
    for _, file := range files {
        localPath := filepath.Join(tempDir, file.Path)

        // Create parent directory
        if err := os.MkdirAll(filepath.Dir(localPath), 0755); err != nil {
            result.Error = fmt.Errorf("failed to create directory: %w", err)
            return result, result.Error
        }

        // Create local file
        localFile, err := os.Create(localPath)
        if err != nil {
            result.Error = fmt.Errorf("failed to create local file: %w", err)
            return result, result.Error
        }

        // Download
        if err := conn.Download(ctx, file.Path, localFile); err != nil {
            localFile.Close()
            result.Error = fmt.Errorf("failed to download %s: %w", file.Path, err)
            return result, result.Error
        }

        info, _ := localFile.Stat()
        result.TotalSize += info.Size()
        localFile.Close()
    }

    // Create backup directory
    backupPath := m.GenerateBackupPath(serverName)
    if err := os.MkdirAll(filepath.Dir(backupPath), 0755); err != nil {
        result.Error = fmt.Errorf("failed to create backup directory: %w", err)
        return result, result.Error
    }

    // Create archive
    if err := CreateArchive(tempDir, backupPath); err != nil {
        result.Error = fmt.Errorf("failed to create archive: %w", err)
        return result, result.Error
    }

    result.BackupPath = backupPath
    result.Duration = time.Since(start)

    return result, nil
}

// Restore extracts a backup and optionally uploads to a server
func (m *Manager) Restore(ctx context.Context, archivePath, localDest string, conn connector.Connector) error {
    // Create temp directory for extraction
    tempDir := filepath.Join(m.TempDir, "restore", fmt.Sprintf("%d", time.Now().UnixNano()))
    if err := os.MkdirAll(tempDir, 0755); err != nil {
        return fmt.Errorf("failed to create temp directory: %w", err)
    }
    defer os.RemoveAll(tempDir)

    // Extract archive
    if err := ExtractArchive(archivePath, tempDir); err != nil {
        return fmt.Errorf("failed to extract archive: %w", err)
    }

    // If local destination, just move files there
    if localDest != "" {
        if err := os.MkdirAll(localDest, 0755); err != nil {
            return err
        }
        return copyDir(tempDir, localDest)
    }

    // Otherwise, upload to server
    if conn == nil {
        return fmt.Errorf("no destination specified (use --local or --server)")
    }

    if err := conn.Connect(ctx); err != nil {
        return fmt.Errorf("connection failed: %w", err)
    }
    defer conn.Close()

    // Walk and upload files
    return filepath.Walk(tempDir, func(path string, info os.FileInfo, err error) error {
        if err != nil || info.IsDir() {
            return err
        }

        relPath, _ := filepath.Rel(tempDir, path)

        file, err := os.Open(path)
        if err != nil {
            return err
        }
        defer file.Close()

        return conn.Upload(ctx, file, relPath)
    })
}

func copyDir(src, dst string) error {
    return filepath.Walk(src, func(path string, info os.FileInfo, err error) error {
        if err != nil {
            return err
        }

        relPath, _ := filepath.Rel(src, path)
        targetPath := filepath.Join(dst, relPath)

        if info.IsDir() {
            return os.MkdirAll(targetPath, info.Mode())
        }

        srcFile, err := os.Open(path)
        if err != nil {
            return err
        }
        defer srcFile.Close()

        dstFile, err := os.Create(targetPath)
        if err != nil {
            return err
        }
        defer dstFile.Close()

        _, err = io.Copy(dstFile, srcFile)
        return err
    })
}
```

**Step 4: Run tests**

```bash
go test ./internal/backup/... -v
```

Expected: PASS

**Step 5: Commit**

```bash
git add internal/backup/backup.go internal/backup/backup_test.go
git commit -m "feat: implement backup manager with download and archive"
```

---

## Epic 6: Prune System

### Task 6.1: Implement Prune Logic

**Files:**
- Create: `internal/prune/prune.go`
- Create: `internal/prune/prune_test.go`

**Step 1: Write test for prune**

```go
// internal/prune/prune_test.go
package prune

import (
    "os"
    "path/filepath"
    "testing"
    "time"
)

func TestFindOldBackups(t *testing.T) {
    tmpDir, err := os.MkdirTemp("", "gsbt-prune-test")
    if err != nil {
        t.Fatal(err)
    }
    defer os.RemoveAll(tmpDir)

    // Create test backup files with different ages
    serverDir := filepath.Join(tmpDir, "test-server")
    os.MkdirAll(serverDir, 0755)

    now := time.Now()

    // Recent backup (1 day old)
    recentFile := filepath.Join(serverDir, now.AddDate(0, 0, -1).Format("2006-01-02_150405")+".tar.gz")
    os.WriteFile(recentFile, []byte("recent"), 0644)

    // Old backup (10 days old)
    oldFile := filepath.Join(serverDir, now.AddDate(0, 0, -10).Format("2006-01-02_150405")+".tar.gz")
    os.WriteFile(oldFile, []byte("old"), 0644)

    // Find backups older than 7 days
    old, err := FindOldBackups(serverDir, 7)
    if err != nil {
        t.Fatalf("FindOldBackups failed: %v", err)
    }

    if len(old) != 1 {
        t.Errorf("expected 1 old backup, got %d", len(old))
    }

    if len(old) > 0 && filepath.Base(old[0].Path) != filepath.Base(oldFile) {
        t.Errorf("wrong file identified as old")
    }
}
```

**Step 2: Run test to verify it fails**

```bash
go test ./internal/prune/... -v
```

Expected: FAIL

**Step 3: Implement prune**

```go
// internal/prune/prune.go
package prune

import (
    "fmt"
    "os"
    "path/filepath"
    "regexp"
    "time"
)

// BackupFile represents a backup file with metadata
type BackupFile struct {
    Path      string
    Timestamp time.Time
    Size      int64
}

// PruneResult contains the result of a prune operation
type PruneResult struct {
    ServerName   string
    DeletedCount int
    FreedBytes   int64
    Errors       []error
}

var backupFileRegex = regexp.MustCompile(`^(\d{4}-\d{2}-\d{2}_\d{6})\.tar\.gz$`)

// ParseBackupTimestamp extracts timestamp from backup filename
func ParseBackupTimestamp(filename string) (time.Time, error) {
    matches := backupFileRegex.FindStringSubmatch(filename)
    if len(matches) != 2 {
        return time.Time{}, fmt.Errorf("invalid backup filename: %s", filename)
    }

    return time.Parse("2006-01-02_150405", matches[1])
}

// FindOldBackups returns backup files older than the specified age in days
func FindOldBackups(serverDir string, maxAgeDays int) ([]BackupFile, error) {
    var oldBackups []BackupFile
    cutoff := time.Now().AddDate(0, 0, -maxAgeDays)

    entries, err := os.ReadDir(serverDir)
    if err != nil {
        return nil, fmt.Errorf("failed to read directory: %w", err)
    }

    for _, entry := range entries {
        if entry.IsDir() {
            continue
        }

        timestamp, err := ParseBackupTimestamp(entry.Name())
        if err != nil {
            continue // Skip non-backup files
        }

        if timestamp.Before(cutoff) {
            info, err := entry.Info()
            if err != nil {
                continue
            }

            oldBackups = append(oldBackups, BackupFile{
                Path:      filepath.Join(serverDir, entry.Name()),
                Timestamp: timestamp,
                Size:      info.Size(),
            })
        }
    }

    return oldBackups, nil
}

// Prune deletes old backups for a server
func Prune(backupLocation, serverName string, maxAgeDays int, dryRun bool) (*PruneResult, error) {
    result := &PruneResult{
        ServerName: serverName,
    }

    serverDir := filepath.Join(backupLocation, serverName)
    oldBackups, err := FindOldBackups(serverDir, maxAgeDays)
    if err != nil {
        return result, err
    }

    for _, backup := range oldBackups {
        if dryRun {
            result.DeletedCount++
            result.FreedBytes += backup.Size
            continue
        }

        if err := os.Remove(backup.Path); err != nil {
            result.Errors = append(result.Errors, fmt.Errorf("failed to delete %s: %w", backup.Path, err))
        } else {
            result.DeletedCount++
            result.FreedBytes += backup.Size
        }
    }

    return result, nil
}

// ListBackups returns all backups for a server
func ListBackups(backupLocation, serverName string) ([]BackupFile, error) {
    serverDir := filepath.Join(backupLocation, serverName)

    if _, err := os.Stat(serverDir); os.IsNotExist(err) {
        return nil, nil
    }

    entries, err := os.ReadDir(serverDir)
    if err != nil {
        return nil, err
    }

    var backups []BackupFile
    for _, entry := range entries {
        if entry.IsDir() {
            continue
        }

        timestamp, err := ParseBackupTimestamp(entry.Name())
        if err != nil {
            continue
        }

        info, err := entry.Info()
        if err != nil {
            continue
        }

        backups = append(backups, BackupFile{
            Path:      filepath.Join(serverDir, entry.Name()),
            Timestamp: timestamp,
            Size:      info.Size(),
        })
    }

    return backups, nil
}
```

**Step 4: Run test to verify it passes**

```bash
go test ./internal/prune/... -v
```

Expected: PASS

**Step 5: Commit**

```bash
git add internal/prune/
git commit -m "feat: implement backup pruning by age"
```

---

## Epic 7: Output Formatting

### Task 7.1: Implement Output Formatters

**Files:**
- Create: `internal/output/output.go`
- Create: `internal/output/output_test.go`

**Step 1: Write test for output formatting**

```go
// internal/output/output_test.go
package output

import (
    "bytes"
    "encoding/json"
    "testing"
)

func TestJSONOutput(t *testing.T) {
    var buf bytes.Buffer
    w := NewWriter(&buf, "json", false)

    data := map[string]string{"status": "success", "message": "test"}
    w.WriteResult(data)

    var result map[string]string
    if err := json.Unmarshal(buf.Bytes(), &result); err != nil {
        t.Fatalf("invalid JSON output: %v", err)
    }

    if result["status"] != "success" {
        t.Errorf("expected status=success, got %s", result["status"])
    }
}

func TestTextOutput(t *testing.T) {
    var buf bytes.Buffer
    w := NewWriter(&buf, "text", false)

    w.Info("test message")

    if buf.String() != "test message\n" {
        t.Errorf("unexpected output: %q", buf.String())
    }
}

func TestQuietMode(t *testing.T) {
    var buf bytes.Buffer
    w := NewWriter(&buf, "text", true)

    w.Info("this should not appear")

    if buf.Len() != 0 {
        t.Errorf("expected no output in quiet mode, got: %q", buf.String())
    }
}
```

**Step 2: Run test to verify it fails**

```bash
go test ./internal/output/... -v
```

Expected: FAIL

**Step 3: Implement output formatters**

```go
// internal/output/output.go
package output

import (
    "encoding/json"
    "fmt"
    "io"
)

// Writer handles formatted output
type Writer struct {
    w      io.Writer
    format string
    quiet  bool
}

// NewWriter creates a new output writer
func NewWriter(w io.Writer, format string, quiet bool) *Writer {
    return &Writer{
        w:      w,
        format: format,
        quiet:  quiet,
    }
}

// Info writes an informational message (suppressed in quiet mode)
func (w *Writer) Info(format string, args ...interface{}) {
    if w.quiet {
        return
    }
    if w.format == "json" {
        return // JSON mode doesn't output informal messages
    }
    fmt.Fprintf(w.w, format+"\n", args...)
}

// Error writes an error message (never suppressed)
func (w *Writer) Error(format string, args ...interface{}) {
    if w.format == "json" {
        w.WriteResult(map[string]interface{}{
            "error": fmt.Sprintf(format, args...),
        })
        return
    }
    fmt.Fprintf(w.w, "ERROR: "+format+"\n", args...)
}

// ServerLog writes a server-prefixed log message
func (w *Writer) ServerLog(server, format string, args ...interface{}) {
    if w.quiet {
        return
    }
    if w.format == "json" {
        return
    }
    msg := fmt.Sprintf(format, args...)
    fmt.Fprintf(w.w, "[%s] %s\n", server, msg)
}

// WriteResult writes a structured result (always outputs in JSON mode)
func (w *Writer) WriteResult(data interface{}) {
    if w.format == "json" {
        enc := json.NewEncoder(w.w)
        enc.SetIndent("", "  ")
        enc.Encode(data)
    }
}

// WriteTable writes tabular data
func (w *Writer) WriteTable(headers []string, rows [][]string) {
    if w.quiet {
        return
    }
    if w.format == "json" {
        return
    }

    // Calculate column widths
    widths := make([]int, len(headers))
    for i, h := range headers {
        widths[i] = len(h)
    }
    for _, row := range rows {
        for i, cell := range row {
            if i < len(widths) && len(cell) > widths[i] {
                widths[i] = len(cell)
            }
        }
    }

    // Print headers
    for i, h := range headers {
        fmt.Fprintf(w.w, "%-*s  ", widths[i], h)
    }
    fmt.Fprintln(w.w)

    // Print rows
    for _, row := range rows {
        for i, cell := range row {
            if i < len(widths) {
                fmt.Fprintf(w.w, "%-*s  ", widths[i], cell)
            }
        }
        fmt.Fprintln(w.w)
    }
}
```

**Step 4: Run test to verify it passes**

```bash
go test ./internal/output/... -v
```

Expected: PASS

**Step 5: Commit**

```bash
git add internal/output/
git commit -m "feat: implement text and JSON output formatters"
```

---

## Epic 8: Wire Up Commands

### Task 8.1: Implement Backup Command

**Files:**
- Modify: `internal/cli/backup.go`

**Step 1: Update backup command implementation**

```go
// internal/cli/backup.go
package cli

import (
    "context"
    "fmt"
    "os"
    "sync"

    "github.com/digitalfiz/gsbt/internal/backup"
    "github.com/digitalfiz/gsbt/internal/config"
    "github.com/digitalfiz/gsbt/internal/connector"
    "github.com/digitalfiz/gsbt/internal/output"
    "github.com/spf13/cobra"
)

var (
    backupServer     string
    backupSequential bool

    // Ad-hoc flags
    backupType       string
    backupHost       string
    backupPort       int
    backupUsername   string
    backupPassword   string
    backupKeyFile    string
    backupRemotePath string
    backupOutput     string
)

var backupCmd = &cobra.Command{
    Use:   "backup",
    Short: "Backup gameserver files",
    Long:  `Download and archive files from configured gameservers.`,
    RunE:  runBackup,
}

func init() {
    backupCmd.Flags().StringVar(&backupServer, "server", "", "backup specific server only")
    backupCmd.Flags().BoolVar(&backupSequential, "sequential", false, "run backups sequentially")

    // Ad-hoc backup flags
    backupCmd.Flags().StringVar(&backupType, "type", "", "connector type (ftp, sftp, nitrado)")
    backupCmd.Flags().StringVar(&backupHost, "host", "", "server host")
    backupCmd.Flags().IntVar(&backupPort, "port", 0, "server port")
    backupCmd.Flags().StringVar(&backupUsername, "user", "", "username")
    backupCmd.Flags().StringVar(&backupPassword, "pass", "", "password")
    backupCmd.Flags().StringVar(&backupKeyFile, "key-file", "", "SSH key file")
    backupCmd.Flags().StringVar(&backupRemotePath, "remote-path", "", "remote path to backup")
    backupCmd.Flags().StringVar(&backupOutput, "backup-output", "", "backup output directory")

    rootCmd.AddCommand(backupCmd)
}

func runBackup(cmd *cobra.Command, args []string) error {
    ctx := context.Background()
    out := output.NewWriter(os.Stdout, GetOutputFormat(), IsQuiet())

    // Check for ad-hoc backup
    if backupType != "" {
        return runAdHocBackup(ctx, out)
    }

    // Load config
    cfgPath, err := config.FindConfigFile(GetConfigFile())
    if err != nil {
        out.Error("Config error: %v", err)
        return err
    }

    cfg, err := config.LoadConfig(cfgPath)
    if err != nil {
        out.Error("Failed to load config: %v", err)
        return err
    }

    // Determine which servers to backup
    var servers []config.Server
    if backupServer != "" {
        for _, s := range cfg.Servers {
            if s.Name == backupServer {
                servers = append(servers, s)
                break
            }
        }
        if len(servers) == 0 {
            out.Error("Server not found: %s", backupServer)
            return fmt.Errorf("server not found: %s", backupServer)
        }
    } else {
        servers = cfg.Servers
    }

    mgr := backup.NewManager(cfg.Defaults.BackupLocation, cfg.Defaults.TempDir)

    var results []*backup.BackupResult
    var exitCode int

    if backupSequential {
        results = runSequentialBackups(ctx, mgr, servers, cfg.Defaults, out)
    } else {
        results = runParallelBackups(ctx, mgr, servers, cfg.Defaults, out)
    }

    // Determine exit code
    failCount := 0
    for _, r := range results {
        if r.Error != nil {
            failCount++
        }
    }

    if failCount == len(results) {
        exitCode = 2
    } else if failCount > 0 {
        exitCode = 1
    }

    // Output results
    if GetOutputFormat() == "json" {
        out.WriteResult(map[string]interface{}{
            "results":   results,
            "exit_code": exitCode,
        })
    }

    if exitCode > 0 {
        os.Exit(exitCode)
    }

    return nil
}

func runSequentialBackups(ctx context.Context, mgr *backup.Manager, servers []config.Server, defaults config.Defaults, out *output.Writer) []*backup.BackupResult {
    var results []*backup.BackupResult

    for _, server := range servers {
        out.ServerLog(server.Name, "Starting backup...")
        result := backupServer(ctx, mgr, server, defaults, out)
        results = append(results, result)

        if result.Error != nil {
            out.ServerLog(server.Name, "FAILED: %v", result.Error)
        } else {
            out.ServerLog(server.Name, "Complete: %d files, %s", result.FileCount, formatBytes(result.TotalSize))
        }
    }

    return results
}

func runParallelBackups(ctx context.Context, mgr *backup.Manager, servers []config.Server, defaults config.Defaults, out *output.Writer) []*backup.BackupResult {
    results := make([]*backup.BackupResult, len(servers))
    var wg sync.WaitGroup

    for i, server := range servers {
        wg.Add(1)
        go func(idx int, srv config.Server) {
            defer wg.Done()
            out.ServerLog(srv.Name, "Starting backup...")
            results[idx] = backupServer(ctx, mgr, srv, defaults, out)

            if results[idx].Error != nil {
                out.ServerLog(srv.Name, "FAILED: %v", results[idx].Error)
            } else {
                out.ServerLog(srv.Name, "Complete: %d files, %s", results[idx].FileCount, formatBytes(results[idx].TotalSize))
            }
        }(i, server)
    }

    wg.Wait()
    return results
}

func backupServer(ctx context.Context, mgr *backup.Manager, server config.Server, defaults config.Defaults, out *output.Writer) *backup.BackupResult {
    connCfg := connector.Config{
        Type:          server.Connection.Type,
        Host:          server.Connection.Host,
        Port:          server.Connection.Port,
        Username:      server.Connection.Username,
        Password:      server.Connection.Password,
        KeyFile:       server.Connection.KeyFile,
        Passive:       server.Connection.IsPassive(),
        TLS:           server.Connection.TLS,
        APIKey:        server.Connection.APIKey,
        ServiceID:     server.Connection.ServiceID,
        RemotePath:    server.Connection.RemotePath,
        Include:       server.Connection.GetInclude(),
        Exclude:       server.Connection.Exclude,
        RetryAttempts: defaults.RetryAttempts,
        RetryDelay:    defaults.RetryDelay,
        RetryBackoff:  defaults.RetryBackoff,
    }

    // Use Nitrado default API key if not specified
    if connCfg.Type == "nitrado" && connCfg.APIKey == "" {
        connCfg.APIKey = defaults.NitradoAPIKey
    }

    conn, err := connector.NewConnector(connCfg)
    if err != nil {
        return &backup.BackupResult{ServerName: server.Name, Error: err}
    }

    return must(mgr.Backup(ctx, server.Name, conn))
}

func must(r *backup.BackupResult, err error) *backup.BackupResult {
    if err != nil && r.Error == nil {
        r.Error = err
    }
    return r
}

func runAdHocBackup(ctx context.Context, out *output.Writer) error {
    if backupRemotePath == "" {
        return fmt.Errorf("--remote-path is required for ad-hoc backup")
    }

    outputDir := backupOutput
    if outputDir == "" {
        outputDir = "."
    }

    connCfg := connector.Config{
        Type:       backupType,
        Host:       backupHost,
        Port:       backupPort,
        Username:   backupUsername,
        Password:   backupPassword,
        KeyFile:    backupKeyFile,
        RemotePath: backupRemotePath,
        Include:    []string{"*"},
    }

    conn, err := connector.NewConnector(connCfg)
    if err != nil {
        return err
    }

    mgr := backup.NewManager(outputDir, "")
    result, err := mgr.Backup(ctx, "adhoc", conn)
    if err != nil {
        out.Error("Backup failed: %v", err)
        return err
    }

    out.Info("Backup complete: %s", result.BackupPath)
    out.Info("Files: %d, Size: %s, Duration: %s", result.FileCount, formatBytes(result.TotalSize), result.Duration)

    if GetOutputFormat() == "json" {
        out.WriteResult(result)
    }

    return nil
}

func formatBytes(b int64) string {
    const unit = 1024
    if b < unit {
        return fmt.Sprintf("%d B", b)
    }
    div, exp := int64(unit), 0
    for n := b / unit; n >= unit; n /= unit {
        div *= unit
        exp++
    }
    return fmt.Sprintf("%.1f %cB", float64(b)/float64(div), "KMGTPE"[exp])
}
```

**Step 2: Verify it compiles**

```bash
go build ./cmd/gsbt/...
```

**Step 3: Commit**

```bash
git add internal/cli/backup.go
git commit -m "feat: implement backup command with parallel execution"
```

---

### Task 8.2: Implement List Command

**Files:**
- Modify: `internal/cli/list.go`

**Step 1: Update list command implementation**

```go
// internal/cli/list.go
package cli

import (
    "fmt"
    "os"
    "path/filepath"
    "time"

    "github.com/digitalfiz/gsbt/internal/config"
    "github.com/digitalfiz/gsbt/internal/output"
    "github.com/digitalfiz/gsbt/internal/prune"
    "github.com/spf13/cobra"
)

var listServer string

var listCmd = &cobra.Command{
    Use:   "list",
    Short: "List configured servers",
    Long:  `Show configured servers and their backup status.`,
    RunE:  runList,
}

func init() {
    listCmd.Flags().StringVar(&listServer, "server", "", "show specific server details")
    rootCmd.AddCommand(listCmd)
}

type serverInfo struct {
    Name           string    `json:"name"`
    Description    string    `json:"description"`
    ConnectionType string    `json:"connection_type"`
    Host           string    `json:"host,omitempty"`
    BackupPath     string    `json:"backup_path"`
    PruneAgeDays   int       `json:"prune_age_days"`
    LastBackup     *time.Time `json:"last_backup,omitempty"`
    BackupCount    int       `json:"backup_count"`
    TotalSizeBytes int64     `json:"total_size_bytes"`
}

func runList(cmd *cobra.Command, args []string) error {
    out := output.NewWriter(os.Stdout, GetOutputFormat(), IsQuiet())

    cfgPath, err := config.FindConfigFile(GetConfigFile())
    if err != nil {
        out.Error("Config error: %v", err)
        return err
    }

    cfg, err := config.LoadConfig(cfgPath)
    if err != nil {
        out.Error("Failed to load config: %v", err)
        return err
    }

    var servers []config.Server
    if listServer != "" {
        for _, s := range cfg.Servers {
            if s.Name == listServer {
                servers = append(servers, s)
                break
            }
        }
        if len(servers) == 0 {
            out.Error("Server not found: %s", listServer)
            return fmt.Errorf("server not found: %s", listServer)
        }
    } else {
        servers = cfg.Servers
    }

    var infos []serverInfo
    for _, server := range servers {
        info := buildServerInfo(server, cfg.Defaults)
        infos = append(infos, info)
    }

    if GetOutputFormat() == "json" {
        out.WriteResult(map[string]interface{}{
            "servers": infos,
        })
    } else {
        printServerList(out, infos)
    }

    return nil
}

func buildServerInfo(server config.Server, defaults config.Defaults) serverInfo {
    backupLocation := server.GetBackupLocation(defaults)
    serverDir := filepath.Join(backupLocation, server.Name)

    info := serverInfo{
        Name:           server.Name,
        Description:    server.Description,
        ConnectionType: server.Connection.Type,
        BackupPath:     serverDir,
        PruneAgeDays:   server.GetPruneAge(defaults),
    }

    // Get host for display
    if server.Connection.Type == "nitrado" {
        info.Host = fmt.Sprintf("service:%s", server.Connection.ServiceID)
    } else {
        info.Host = server.Connection.Host
    }

    // Get backup info
    backups, err := prune.ListBackups(backupLocation, server.Name)
    if err == nil && len(backups) > 0 {
        info.BackupCount = len(backups)

        // Find most recent and calculate total size
        var mostRecent time.Time
        for _, b := range backups {
            info.TotalSizeBytes += b.Size
            if b.Timestamp.After(mostRecent) {
                mostRecent = b.Timestamp
            }
        }
        info.LastBackup = &mostRecent
    }

    return info
}

func printServerList(out *output.Writer, infos []serverInfo) {
    out.Info("SERVERS (%d configured)\n", len(infos))

    for _, info := range infos {
        out.Info("  %s", info.Name)
        if info.Description != "" {
            out.Info("    Description:  %s", info.Description)
        }
        out.Info("    Connection:   %s://%s", info.ConnectionType, info.Host)
        out.Info("    Backup Path:  %s", info.BackupPath)
        out.Info("    Prune Age:    %d days", info.PruneAgeDays)

        if info.LastBackup != nil {
            ago := time.Since(*info.LastBackup).Round(time.Minute)
            out.Info("    Last Backup:  %s (%s ago)", info.LastBackup.Format("2006-01-02 15:04:05"), ago)
        } else {
            out.Info("    Last Backup:  never")
        }

        out.Info("    Backup Count: %d", info.BackupCount)
        if info.TotalSizeBytes > 0 {
            out.Info("    Total Size:   %s", formatBytes(info.TotalSizeBytes))
        }
        out.Info("")
    }
}
```

**Step 2: Verify it compiles**

```bash
go build ./cmd/gsbt/...
```

**Step 3: Commit**

```bash
git add internal/cli/list.go
git commit -m "feat: implement list command showing servers and backup status"
```

---

### Task 8.3: Implement Prune Command

**Files:**
- Modify: `internal/cli/prune.go`

**Step 1: Update prune command implementation**

```go
// internal/cli/prune.go
package cli

import (
    "fmt"
    "os"

    "github.com/digitalfiz/gsbt/internal/config"
    "github.com/digitalfiz/gsbt/internal/output"
    "github.com/digitalfiz/gsbt/internal/prune"
    "github.com/spf13/cobra"
)

var (
    pruneServer string
    pruneDryRun bool
)

var pruneCmd = &cobra.Command{
    Use:   "prune",
    Short: "Remove old backups",
    Long:  `Delete backups older than the configured prune_age.`,
    RunE:  runPrune,
}

func init() {
    pruneCmd.Flags().StringVar(&pruneServer, "server", "", "prune specific server only")
    pruneCmd.Flags().BoolVar(&pruneDryRun, "dry-run", false, "show what would be deleted")
    rootCmd.AddCommand(pruneCmd)
}

func runPrune(cmd *cobra.Command, args []string) error {
    out := output.NewWriter(os.Stdout, GetOutputFormat(), IsQuiet())

    cfgPath, err := config.FindConfigFile(GetConfigFile())
    if err != nil {
        out.Error("Config error: %v", err)
        return err
    }

    cfg, err := config.LoadConfig(cfgPath)
    if err != nil {
        out.Error("Failed to load config: %v", err)
        return err
    }

    var servers []config.Server
    if pruneServer != "" {
        for _, s := range cfg.Servers {
            if s.Name == pruneServer {
                servers = append(servers, s)
                break
            }
        }
        if len(servers) == 0 {
            out.Error("Server not found: %s", pruneServer)
            return fmt.Errorf("server not found: %s", pruneServer)
        }
    } else {
        servers = cfg.Servers
    }

    if pruneDryRun {
        out.Info("DRY RUN - no files will be deleted\n")
    }

    var results []*prune.PruneResult
    var totalDeleted int
    var totalFreed int64

    for _, server := range servers {
        pruneAge := server.GetPruneAge(cfg.Defaults)
        backupLocation := server.GetBackupLocation(cfg.Defaults)

        out.ServerLog(server.Name, "Checking for backups older than %d days...", pruneAge)

        result, err := prune.Prune(backupLocation, server.Name, pruneAge, pruneDryRun)
        if err != nil {
            out.ServerLog(server.Name, "ERROR: %v", err)
            continue
        }

        results = append(results, result)
        totalDeleted += result.DeletedCount
        totalFreed += result.FreedBytes

        if result.DeletedCount > 0 {
            action := "Deleted"
            if pruneDryRun {
                action = "Would delete"
            }
            out.ServerLog(server.Name, "%s %d backups, freed %s", action, result.DeletedCount, formatBytes(result.FreedBytes))
        } else {
            out.ServerLog(server.Name, "No old backups to prune")
        }

        for _, err := range result.Errors {
            out.ServerLog(server.Name, "  Error: %v", err)
        }
    }

    out.Info("")
    if pruneDryRun {
        out.Info("Would delete %d backups, freeing %s", totalDeleted, formatBytes(totalFreed))
    } else {
        out.Info("Deleted %d backups, freed %s", totalDeleted, formatBytes(totalFreed))
    }

    if GetOutputFormat() == "json" {
        out.WriteResult(map[string]interface{}{
            "results":       results,
            "total_deleted": totalDeleted,
            "total_freed":   totalFreed,
            "dry_run":       pruneDryRun,
        })
    }

    return nil
}
```

**Step 2: Verify it compiles**

```bash
go build ./cmd/gsbt/...
```

**Step 3: Commit**

```bash
git add internal/cli/prune.go
git commit -m "feat: implement prune command with dry-run support"
```

---

### Task 8.4: Implement Restore Command

**Files:**
- Modify: `internal/cli/restore.go`

**Step 1: Update restore command implementation**

```go
// internal/cli/restore.go
package cli

import (
    "bufio"
    "context"
    "fmt"
    "os"
    "strings"

    "github.com/digitalfiz/gsbt/internal/backup"
    "github.com/digitalfiz/gsbt/internal/config"
    "github.com/digitalfiz/gsbt/internal/connector"
    "github.com/digitalfiz/gsbt/internal/output"
    "github.com/spf13/cobra"
)

var (
    restoreServer string
    restoreLocal  string
    restoreDryRun bool
    restoreForce  bool
)

var restoreCmd = &cobra.Command{
    Use:   "restore <backup-file>",
    Short: "Restore a backup",
    Long:  `Restore a backup to a server or extract locally.`,
    Args:  cobra.ExactArgs(1),
    RunE:  runRestore,
}

func init() {
    restoreCmd.Flags().StringVar(&restoreServer, "server", "", "restore to server")
    restoreCmd.Flags().StringVar(&restoreLocal, "local", "", "extract to local path")
    restoreCmd.Flags().BoolVar(&restoreDryRun, "dry-run", false, "show what would be restored")
    restoreCmd.Flags().BoolVar(&restoreForce, "force", false, "skip confirmation prompt")
    rootCmd.AddCommand(restoreCmd)
}

func runRestore(cmd *cobra.Command, args []string) error {
    ctx := context.Background()
    out := output.NewWriter(os.Stdout, GetOutputFormat(), IsQuiet())

    archivePath := args[0]

    // Verify archive exists
    if _, err := os.Stat(archivePath); err != nil {
        out.Error("Backup file not found: %s", archivePath)
        return err
    }

    // Must specify either --server or --local
    if restoreServer == "" && restoreLocal == "" {
        out.Error("Must specify either --server or --local")
        return fmt.Errorf("must specify either --server or --local")
    }

    // Handle local extraction
    if restoreLocal != "" {
        if restoreDryRun {
            out.Info("Would extract %s to %s", archivePath, restoreLocal)
            return nil
        }

        out.Info("Extracting %s to %s...", archivePath, restoreLocal)
        mgr := backup.NewManager("", "")
        if err := mgr.Restore(ctx, archivePath, restoreLocal, nil); err != nil {
            out.Error("Restore failed: %v", err)
            return err
        }
        out.Info("Restore complete")
        return nil
    }

    // Handle server restore
    cfgPath, err := config.FindConfigFile(GetConfigFile())
    if err != nil {
        out.Error("Config error: %v", err)
        return err
    }

    cfg, err := config.LoadConfig(cfgPath)
    if err != nil {
        out.Error("Failed to load config: %v", err)
        return err
    }

    // Find server
    var server *config.Server
    for _, s := range cfg.Servers {
        if s.Name == restoreServer {
            server = &s
            break
        }
    }
    if server == nil {
        out.Error("Server not found: %s", restoreServer)
        return fmt.Errorf("server not found: %s", restoreServer)
    }

    // Confirmation prompt
    if !restoreForce && !restoreDryRun {
        out.Info("WARNING: This will overwrite files on %s", server.Name)
        out.Info("Remote path: %s", server.Connection.RemotePath)
        fmt.Print("Continue? [y/N]: ")

        reader := bufio.NewReader(os.Stdin)
        response, _ := reader.ReadString('\n')
        response = strings.TrimSpace(strings.ToLower(response))

        if response != "y" && response != "yes" {
            out.Info("Restore cancelled")
            return nil
        }
    }

    if restoreDryRun {
        out.Info("Would restore %s to server %s (%s)", archivePath, server.Name, server.Connection.RemotePath)
        return nil
    }

    // Create connector
    connCfg := connector.Config{
        Type:       server.Connection.Type,
        Host:       server.Connection.Host,
        Port:       server.Connection.Port,
        Username:   server.Connection.Username,
        Password:   server.Connection.Password,
        KeyFile:    server.Connection.KeyFile,
        Passive:    server.Connection.IsPassive(),
        TLS:        server.Connection.TLS,
        APIKey:     server.Connection.APIKey,
        ServiceID:  server.Connection.ServiceID,
        RemotePath: server.Connection.RemotePath,
    }

    if connCfg.Type == "nitrado" && connCfg.APIKey == "" {
        connCfg.APIKey = cfg.Defaults.NitradoAPIKey
    }

    conn, err := connector.NewConnector(connCfg)
    if err != nil {
        out.Error("Failed to create connector: %v", err)
        return err
    }

    out.Info("Restoring %s to %s...", archivePath, server.Name)
    mgr := backup.NewManager(cfg.Defaults.BackupLocation, cfg.Defaults.TempDir)
    if err := mgr.Restore(ctx, archivePath, "", conn); err != nil {
        out.Error("Restore failed: %v", err)
        return err
    }

    out.Info("Restore complete")

    if GetOutputFormat() == "json" {
        out.WriteResult(map[string]interface{}{
            "status":  "success",
            "archive": archivePath,
            "server":  server.Name,
        })
    }

    return nil
}
```

**Step 2: Verify it compiles**

```bash
go build ./cmd/gsbt/...
```

**Step 3: Commit**

```bash
git add internal/cli/restore.go
git commit -m "feat: implement restore command with server upload and local extraction"
```

---

## Epic 9: Testing & Polish

### Task 9.1: Add Integration Tests

**Files:**
- Create: `tests/integration_test.go`

*Details omitted for brevity - would include mock FTP server tests*

### Task 9.2: Add Retry Logic

**Files:**
- Create: `internal/retry/retry.go`
- Modify: connector implementations to use retry

*Details omitted for brevity*

### Task 9.3: Build Configuration

**Files:**
- Create: `Makefile`
- Create: `.goreleaser.yml`

*Details omitted for brevity*

---

## Summary

| Epic | Tasks | Estimated Complexity |
|------|-------|---------------------|
| 1. Project Setup | 2 | Low |
| 2. CLI Framework | 3 | Low |
| 3. Config System | 3 | Medium |
| 4. Connector System | 6 | High |
| 5. Backup System | 2 | Medium |
| 6. Prune System | 1 | Low |
| 7. Output Formatting | 1 | Low |
| 8. Wire Up Commands | 4 | Medium |
| 9. Testing & Polish | 3 | Medium |

**Total: 25 tasks across 9 epics**
