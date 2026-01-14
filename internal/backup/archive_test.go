// internal/backup/archive_test.go
package backup

import (
	"archive/tar"
	"compress/gzip"
	"io"
	"os"
	"path/filepath"
	"testing"
)

func TestCreateArchive(t *testing.T) {
	tmpDir := t.TempDir()
	src := filepath.Join(tmpDir, "src")
	os.MkdirAll(filepath.Join(src, "nested"), 0o755)
	os.WriteFile(filepath.Join(src, "root.txt"), []byte("root"), 0o644)
	os.WriteFile(filepath.Join(src, "nested", "child.txt"), []byte("child"), 0o644)

	dest := filepath.Join(tmpDir, "out.tar.gz")
	if err := CreateArchive(src, dest); err != nil {
		t.Fatalf("CreateArchive error: %v", err)
	}

	// Verify the archive contents
	f, err := os.Open(dest)
	if err != nil {
		t.Fatalf("open archive: %v", err)
	}
	defer f.Close()

	gz, err := gzip.NewReader(f)
	if err != nil {
		t.Fatalf("gzip reader: %v", err)
	}
	defer gz.Close()

	tr := tar.NewReader(gz)
	seen := map[string]string{}

	for {
		hdr, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			t.Fatalf("tar next: %v", err)
		}

		switch hdr.Name {
		case "root.txt":
			data, _ := io.ReadAll(tr)
			seen[hdr.Name] = string(data)
		case "nested/child.txt":
			data, _ := io.ReadAll(tr)
			seen[hdr.Name] = string(data)
		}
	}

	if got := seen["root.txt"]; got != "root" {
		t.Fatalf("root.txt content = %q", got)
	}
	if got := seen["nested/child.txt"]; got != "child" {
		t.Fatalf("nested/child.txt content = %q", got)
	}
}

func TestTimestampedFilename(t *testing.T) {
	name := TimestampedFilename()
	if filepath.Ext(name) != ".gz" {
		t.Fatalf("expected .tar.gz extension, got %s", name)
	}
	if len(name) != len("2006-01-02_150405.tar.gz") {
		t.Fatalf("unexpected timestamp length: %d", len(name))
	}
}
