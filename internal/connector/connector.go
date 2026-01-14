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
