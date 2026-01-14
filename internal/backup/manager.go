// internal/backup/manager.go
package backup

import (
	"context"
	"fmt"
	"io"
	"os"
	"path/filepath"

	"github.com/digitalfiz/gsbt/internal/connector"
)

// Manager coordinates backup operations for a single server.
type Manager struct {
	TempDir        string
	BackupLocation string
}

// Backup pulls files via connector, archives them, and writes to backup location.
func (m *Manager) Backup(ctx context.Context, conn connector.Connector) (string, error) {
	if conn == nil {
		return "", fmt.Errorf("connector is required")
	}

	if m.BackupLocation == "" {
		return "", fmt.Errorf("backup location is required")
	}

	tempDir := m.TempDir
	if tempDir == "" {
		tempDir = filepath.Join(m.BackupLocation, ".tmp")
	}

	if err := os.MkdirAll(tempDir, 0o755); err != nil {
		return "", fmt.Errorf("create temp dir: %w", err)
	}

	if err := conn.Connect(ctx); err != nil {
		return "", err
	}
	defer conn.Close()

	// List files
	files, err := conn.List(ctx)
	if err != nil {
		return "", fmt.Errorf("list: %w", err)
	}

	// Download files to temp dir
	for _, file := range files {
		localPath := filepath.Join(tempDir, file.Path)
		if file.IsDir {
			// ensure directories exist for completeness
			os.MkdirAll(localPath, 0o755)
			continue
		}

		if err := os.MkdirAll(filepath.Dir(localPath), 0o755); err != nil {
			return "", fmt.Errorf("mkdir for %s: %w", file.Path, err)
		}

		f, err := os.Create(localPath)
		if err != nil {
			return "", fmt.Errorf("create %s: %w", file.Path, err)
		}

		if err := conn.Download(ctx, file.Path, f); err != nil {
			f.Close()
			return "", fmt.Errorf("download %s: %w", file.Path, err)
		}
		f.Close()
	}

	// Build archive path
	archiveDir := m.BackupLocation
	if err := os.MkdirAll(archiveDir, 0o755); err != nil {
		return "", fmt.Errorf("create backup dir: %w", err)
	}

	archivePath := filepath.Join(archiveDir, TimestampedFilename())
	if err := CreateArchive(tempDir, archivePath); err != nil {
		return "", fmt.Errorf("create archive: %w", err)
	}

	return archivePath, nil
}

// Restore uploads an archive's contents back to the remote via connector.
// Not implemented yet; placeholder for future work.
func (m *Manager) Restore(ctx context.Context, conn connector.Connector, r io.Reader) error {
	return fmt.Errorf("restore not implemented")
}
