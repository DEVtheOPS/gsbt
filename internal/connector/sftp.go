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
