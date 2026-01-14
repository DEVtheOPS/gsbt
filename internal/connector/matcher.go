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
