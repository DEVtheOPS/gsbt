// internal/backup/manager.go
package backup

import (
	"context"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"time"

	"github.com/devtheops/gsbt/internal/connector"
	"github.com/devtheops/gsbt/internal/progress"
)

// Manager coordinates backup operations for a single server.
type Manager struct {
	TempDir        string
	BackupLocation string
	Progress       progress.Reporter
}

// Stats represents a summary of a backup run.
type Stats struct {
	Files    int
	Bytes    int64
	Duration time.Duration
}

// Backup pulls files via connector, archives them, and writes to backup location.
func (m *Manager) Backup(ctx context.Context, conn connector.Connector) (string, Stats, error) {
	start := time.Now()
	stats := Stats{}

	if conn == nil {
		return "", stats, fmt.Errorf("connector is required")
	}

	if m.BackupLocation == "" {
		return "", stats, fmt.Errorf("backup location is required")
	}

	tempDir := m.TempDir
	if tempDir == "" {
		tempDir = filepath.Join(m.BackupLocation, ".tmp")
	}

	if err := os.MkdirAll(tempDir, 0o755); err != nil {
		return "", stats, fmt.Errorf("create temp dir: %w", err)
	}

	if err := conn.Connect(ctx); err != nil {
		return "", stats, err
	}
	defer conn.Close()

	// List files
	files, err := conn.List(ctx)
	if err != nil {
		return "", stats, fmt.Errorf("list: %w", err)
	}

	var totalSize int64
	for _, f := range files {
		if !f.IsDir {
			stats.Files++
			stats.Bytes += f.Size
			totalSize += f.Size
		}
	}
	if m.Progress != nil {
		m.Progress.Start(totalSize, len(files))
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
			return "", stats, fmt.Errorf("mkdir for %s: %w", file.Path, err)
		}

		f, err := os.Create(localPath)
		if err != nil {
			return "", stats, fmt.Errorf("create %s: %w", file.Path, err)
		}

		if m.Progress != nil {
			m.Progress.FileStart(file.Path, file.Size)
		}

		// Wrap writer to report progress periodically
		pw := &progressWriter{w: f, cb: func(written int64) {
			if m.Progress != nil {
				m.Progress.FileProgress(file.Path, written, file.Size)
			}
		}}

		if err := conn.Download(ctx, file.Path, pw); err != nil {
			f.Close()
			return "", stats, fmt.Errorf("download %s: %w", file.Path, err)
		}
		f.Close()

		if m.Progress != nil {
			m.Progress.FileDone(file.Path)
		}
	}

	if m.Progress != nil {
		m.Progress.Close()
	}

	// Build archive path
	archiveDir := m.BackupLocation
	if err := os.MkdirAll(archiveDir, 0o755); err != nil {
		return "", stats, fmt.Errorf("create backup dir: %w", err)
	}

	archivePath := filepath.Join(archiveDir, TimestampedFilename())
	if err := CreateArchive(tempDir, archivePath); err != nil {
		return "", stats, fmt.Errorf("create archive: %w", err)
	}

	stats.Duration = time.Since(start)
	return archivePath, stats, nil
}

// progressWriter wraps an io.Writer to report incremental bytes written.
type progressWriter struct {
	w  io.Writer
	n  int64
	cb func(written int64)
}

func (p *progressWriter) Write(b []byte) (int, error) {
	n, err := p.w.Write(b)
	p.n += int64(n)
	if p.cb != nil {
		p.cb(p.n)
	}
	return n, err
}

// Restore uploads an archive's contents back to the remote via connector.
// Not implemented yet; placeholder for future work.
func (m *Manager) Restore(ctx context.Context, conn connector.Connector, r io.Reader) error {
	return fmt.Errorf("restore not implemented")
}
