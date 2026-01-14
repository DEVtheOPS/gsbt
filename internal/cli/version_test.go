// internal/cli/version_test.go
package cli

import (
	"bytes"
	"encoding/json"
	"strings"
	"testing"

	"github.com/digitalfiz/gsbt/internal/version"
)

func TestVersionCommand(t *testing.T) {
	// Save original values
	origVersion := version.Version
	origCommit := version.Commit
	origBuildDate := version.BuildDate

	// Set test values
	version.Version = "1.2.3"
	version.Commit = "abc123"
	version.BuildDate = "2024-01-01"

	// Restore original values after test
	defer func() {
		version.Version = origVersion
		version.Commit = origCommit
		version.BuildDate = origBuildDate
	}()

	tests := []struct {
		name           string
		args           []string
		wantInOutput   []string
		checkJSON      bool
		jsonValidation func(t *testing.T, data map[string]string)
	}{
		{
			name: "text format default",
			args: []string{"version"},
			wantInOutput: []string{
				"gsbt 1.2.3",
				"commit: abc123",
				"built:  2024-01-01",
			},
			checkJSON: false,
		},
		{
			name: "json format",
			args: []string{"--output", "json", "version"},
			wantInOutput: []string{
				`"version"`,
				`"commit"`,
				`"build_date"`,
			},
			checkJSON: true,
			jsonValidation: func(t *testing.T, data map[string]string) {
				if data["version"] != "1.2.3" {
					t.Errorf("version = %q, want %q", data["version"], "1.2.3")
				}
				if data["commit"] != "abc123" {
					t.Errorf("commit = %q, want %q", data["commit"], "abc123")
				}
				if data["build_date"] != "2024-01-01" {
					t.Errorf("build_date = %q, want %q", data["build_date"], "2024-01-01")
				}
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			resetRootCmd()
			resetFlags()

			// Re-add version command since we reset root
			rootCmd.AddCommand(versionCmd)

			buf := new(bytes.Buffer)
			rootCmd.SetOut(buf)
			rootCmd.SetErr(buf)
			rootCmd.SetArgs(tt.args)

			err := rootCmd.Execute()
			if err != nil {
				t.Fatalf("version command failed: %v", err)
			}

			output := buf.String()

			if tt.checkJSON {
				// Validate JSON format
				var data map[string]string
				if err := json.Unmarshal([]byte(output), &data); err != nil {
					t.Fatalf("failed to parse JSON output: %v\nOutput: %s", err, output)
				}
				if tt.jsonValidation != nil {
					tt.jsonValidation(t, data)
				}
			}

			// Check expected strings are in output
			for _, expected := range tt.wantInOutput {
				if !strings.Contains(output, expected) {
					t.Errorf("output missing %q\nGot: %s", expected, output)
				}
			}
		})
	}
}

func TestVersionCommandMetadata(t *testing.T) {
	if versionCmd.Use != "version" {
		t.Errorf("versionCmd.Use = %q, want %q", versionCmd.Use, "version")
	}

	if versionCmd.Short != "Print version information" {
		t.Errorf("versionCmd.Short = %q, want %q", versionCmd.Short, "Print version information")
	}
}

func TestVersionCommandWithDevDefaults(t *testing.T) {
	// Test with default "dev" values
	resetRootCmd()
	resetFlags()
	rootCmd.AddCommand(versionCmd)

	buf := new(bytes.Buffer)
	rootCmd.SetOut(buf)
	rootCmd.SetErr(buf)
	rootCmd.SetArgs([]string{"version"})

	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("version command failed: %v", err)
	}

	output := buf.String()

	// Check that output contains the word "dev"
	if !strings.Contains(output, "dev") {
		t.Errorf("output should contain 'dev' when Version is not set\nGot: %s", output)
	}
}
