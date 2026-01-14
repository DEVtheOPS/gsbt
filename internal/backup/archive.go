// internal/backup/archive.go
package backup

import (
	"archive/tar"
	"compress/gzip"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"time"
)

// CreateArchive compresses the contents of srcDir into a .tar.gz at destPath.
// The archive stores paths relative to srcDir.
func CreateArchive(srcDir, destPath string) error {
	if srcDir == "" {
		return fmt.Errorf("srcDir is required")
	}

	if destPath == "" {
		return fmt.Errorf("destPath is required")
	}

	if err := os.MkdirAll(filepath.Dir(destPath), 0o755); err != nil {
		return fmt.Errorf("failed to create archive directory: %w", err)
	}

	outFile, err := os.Create(destPath)
	if err != nil {
		return fmt.Errorf("failed to create archive file: %w", err)
	}
	defer outFile.Close()

	gzWriter := gzip.NewWriter(outFile)
	defer gzWriter.Close()

	tarWriter := tar.NewWriter(gzWriter)
	defer tarWriter.Close()

	// Walk the source directory and add files to the archive.
	return filepath.Walk(srcDir, func(path string, info os.FileInfo, walkErr error) error {
		if walkErr != nil {
			return walkErr
		}

		// Skip the root directory header
		if path == srcDir {
			return nil
		}

		relPath, err := filepath.Rel(srcDir, path)
		if err != nil {
			return err
		}

		header, err := tar.FileInfoHeader(info, "")
		if err != nil {
			return fmt.Errorf("create header for %s: %w", relPath, err)
		}
		header.Name = filepath.ToSlash(relPath)

		if err := tarWriter.WriteHeader(header); err != nil {
			return fmt.Errorf("write header for %s: %w", relPath, err)
		}

		if info.Mode().IsRegular() {
			file, err := os.Open(path)
			if err != nil {
				return fmt.Errorf("open %s: %w", relPath, err)
			}
			defer file.Close()

			if _, err := io.Copy(tarWriter, file); err != nil {
				return fmt.Errorf("copy %s: %w", relPath, err)
			}
		}

		return nil
	})
}

// TimestampedFilename returns a UTC timestamped filename in gsbt format.
func TimestampedFilename() string {
	return time.Now().UTC().Format("2006-01-02_150405") + ".tar.gz"
}
