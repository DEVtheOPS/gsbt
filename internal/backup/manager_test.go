// internal/backup/manager_test.go
package backup

import (
	"context"
	"io"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/devtheops/gsbt/internal/connector"
)

// mockConnector is a simple in-memory connector for tests.
type mockConnector struct {
	files     []connector.FileInfo
	data      map[string]string
	connected bool
}

func (m *mockConnector) Connect(ctx context.Context) error {
	m.connected = true
	return nil
}

func (m *mockConnector) List(ctx context.Context) ([]connector.FileInfo, error) {
	return m.files, nil
}

func (m *mockConnector) Download(ctx context.Context, remotePath string, w io.Writer) error {
	if !m.connected {
		return io.ErrClosedPipe
	}
	data := m.data[remotePath]
	_, err := w.Write([]byte(data))
	return err
}

func (m *mockConnector) Upload(ctx context.Context, r io.Reader, remotePath string) error { return nil }
func (m *mockConnector) Close() error                                                     { m.connected = false; return nil }
func (m *mockConnector) Name() string                                                     { return "mock" }

func TestManagerBackup(t *testing.T) {
	ctx := context.Background()
	tmp := t.TempDir()

	mock := &mockConnector{
		files: []connector.FileInfo{
			{Path: "file1.txt", Size: 5, ModTime: time.Now()},
			{Path: "nested/file2.txt", Size: 5, ModTime: time.Now()},
		},
		data: map[string]string{
			"file1.txt":        "hello",
			"nested/file2.txt": "world",
		},
	}

	mgr := Manager{BackupLocation: tmp}
	archivePath, stats, err := mgr.Backup(ctx, mock)
	if err != nil {
		t.Fatalf("Backup error: %v", err)
	}

	if _, err := os.Stat(archivePath); err != nil {
		t.Fatalf("archive not created: %v", err)
	}

	// Archive should live in backup location
	if filepath.Dir(archivePath) != tmp {
		t.Fatalf("archive dir = %s, want %s", filepath.Dir(archivePath), tmp)
	}

	if stats.Files != 2 {
		t.Fatalf("stats files = %d, want 2", stats.Files)
	}
	if stats.Bytes != int64(len("hello")+len("world")) {
		t.Fatalf("stats bytes = %d, want %d", stats.Bytes, len("hello")+len("world"))
	}
}
