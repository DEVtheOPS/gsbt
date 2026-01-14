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
