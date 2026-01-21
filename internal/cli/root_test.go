// internal/cli/root_test.go
package cli

import (
	"bytes"
	"fmt"
	"strings"
	"testing"

	"github.com/spf13/cobra"
)

// resetFlags resets all global flags to their default values
func resetFlags() {
	cfgFile = ""
	outputFmt = "text"
	verbose = false
	quiet = false
}

// resetRootCmd recreates the root command for testing
func resetRootCmd() {
	rootCmd = &cobra.Command{
		Use:   "gsbt",
		Short: "Gameserver Backup Tool",
		Long:  `A modular backup tool for game servers supporting FTP, SFTP, and Nitrado.`,
		PersistentPreRunE: func(cmd *cobra.Command, args []string) error {
			if verbose && quiet {
				return fmt.Errorf("--verbose and --quiet flags cannot be used together")
			}
			return nil
		},
		Run: func(cmd *cobra.Command, args []string) {
			// When no subcommand is provided, show help
			cmd.Help()
		},
	}
	rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "", "config file path")
	rootCmd.PersistentFlags().StringVar(&outputFmt, "output", "text", "output format (text, json)")
	rootCmd.PersistentFlags().BoolVarP(&verbose, "verbose", "v", false, "verbose output")
	rootCmd.PersistentFlags().BoolVarP(&quiet, "quiet", "q", false, "suppress non-error output")
}

func TestGetConfigFile(t *testing.T) {
	tests := []struct {
		name     string
		setValue string
		want     string
	}{
		{
			name:     "default empty",
			setValue: "",
			want:     "",
		},
		{
			name:     "custom path",
			setValue: "/etc/gsbt/config.yaml",
			want:     "/etc/gsbt/config.yaml",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			resetFlags()
			cfgFile = tt.setValue
			if got := GetConfigFile(); got != tt.want {
				t.Errorf("GetConfigFile() = %q, want %q", got, tt.want)
			}
		})
	}
}

func TestGetOutputFormat(t *testing.T) {
	tests := []struct {
		name     string
		setValue string
		want     string
	}{
		{
			name:     "default text",
			setValue: "text",
			want:     "text",
		},
		{
			name:     "json format",
			setValue: "json",
			want:     "json",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			resetFlags()
			outputFmt = tt.setValue
			if got := GetOutputFormat(); got != tt.want {
				t.Errorf("GetOutputFormat() = %q, want %q", got, tt.want)
			}
		})
	}
}

func TestIsVerbose(t *testing.T) {
	tests := []struct {
		name     string
		setValue bool
		want     bool
	}{
		{
			name:     "default false",
			setValue: false,
			want:     false,
		},
		{
			name:     "verbose enabled",
			setValue: true,
			want:     true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			resetFlags()
			verbose = tt.setValue
			if got := IsVerbose(); got != tt.want {
				t.Errorf("IsVerbose() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestIsQuiet(t *testing.T) {
	tests := []struct {
		name     string
		setValue bool
		want     bool
	}{
		{
			name:     "default false",
			setValue: false,
			want:     false,
		},
		{
			name:     "quiet enabled",
			setValue: true,
			want:     true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			resetFlags()
			quiet = tt.setValue
			if got := IsQuiet(); got != tt.want {
				t.Errorf("IsQuiet() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestRootCommandFlags(t *testing.T) {
	resetRootCmd()
	resetFlags()

	tests := []struct {
		name      string
		args      []string
		wantError bool
		checkFunc func(t *testing.T)
	}{
		{
			name:      "config flag",
			args:      []string{"--config", "/path/to/config.yaml"},
			wantError: false,
			checkFunc: func(t *testing.T) {
				if cfgFile != "/path/to/config.yaml" {
					t.Errorf("config flag not set correctly, got %q", cfgFile)
				}
			},
		},
		{
			name:      "output flag",
			args:      []string{"--output", "json"},
			wantError: false,
			checkFunc: func(t *testing.T) {
				if outputFmt != "json" {
					t.Errorf("output flag not set correctly, got %q", outputFmt)
				}
			},
		},
		{
			name:      "verbose flag short",
			args:      []string{"-v"},
			wantError: false,
			checkFunc: func(t *testing.T) {
				if !verbose {
					t.Error("verbose flag not set correctly")
				}
			},
		},
		{
			name:      "verbose flag long",
			args:      []string{"--verbose"},
			wantError: false,
			checkFunc: func(t *testing.T) {
				if !verbose {
					t.Error("verbose flag not set correctly")
				}
			},
		},
		{
			name:      "quiet flag short",
			args:      []string{"-q"},
			wantError: false,
			checkFunc: func(t *testing.T) {
				if !quiet {
					t.Error("quiet flag not set correctly")
				}
			},
		},
		{
			name:      "quiet flag long",
			args:      []string{"--quiet"},
			wantError: false,
			checkFunc: func(t *testing.T) {
				if !quiet {
					t.Error("quiet flag not set correctly")
				}
			},
		},
		{
			name:      "multiple flags",
			args:      []string{"--config", "test.yaml", "--output", "json", "-v"},
			wantError: false,
			checkFunc: func(t *testing.T) {
				if cfgFile != "test.yaml" {
					t.Errorf("config flag not set correctly, got %q", cfgFile)
				}
				if outputFmt != "json" {
					t.Errorf("output flag not set correctly, got %q", outputFmt)
				}
				if !verbose {
					t.Error("verbose flag not set correctly")
				}
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			resetRootCmd()
			resetFlags()

			rootCmd.SetArgs(tt.args)
			err := rootCmd.Execute()

			if tt.wantError && err == nil {
				t.Error("expected error but got none")
			}
			if !tt.wantError && err != nil {
				t.Errorf("unexpected error: %v", err)
			}

			if tt.checkFunc != nil {
				tt.checkFunc(t)
			}
		})
	}
}

func TestRootCommandHelp(t *testing.T) {
	resetRootCmd()
	resetFlags()

	buf := new(bytes.Buffer)
	rootCmd.SetOut(buf)
	rootCmd.SetErr(buf)

	// Get the help by calling Help() directly
	err := rootCmd.Help()
	if err != nil {
		t.Fatalf("help command failed: %v", err)
	}

	output := buf.String()

	// Check that help output contains expected elements
	expectedStrings := []string{
		"gsbt",
		"modular backup tool", // Part of the Long description
		"--config",
		"--output",
		"--verbose",
		"--quiet",
		"config file path",
		"output format",
		"verbose output",
		"suppress non-error output",
	}

	for _, expected := range expectedStrings {
		if !strings.Contains(output, expected) {
			t.Errorf("help output missing %q", expected)
		}
	}
}

func TestRootCommandMetadata(t *testing.T) {
	if rootCmd.Use != "gsbt" {
		t.Errorf("rootCmd.Use = %q, want %q", rootCmd.Use, "gsbt")
	}

	if rootCmd.Short != "Gameserver Backup Tool" {
		t.Errorf("rootCmd.Short = %q, want %q", rootCmd.Short, "Gameserver Backup Tool")
	}

	if !strings.Contains(rootCmd.Long, "modular backup tool") {
		t.Errorf("rootCmd.Long doesn't contain expected text, got %q", rootCmd.Long)
	}
}

func TestPersistentFlags(t *testing.T) {
	resetRootCmd()

	// Verify all persistent flags are registered
	flags := []string{"config", "output", "verbose", "quiet"}
	for _, flagName := range flags {
		flag := rootCmd.PersistentFlags().Lookup(flagName)
		if flag == nil {
			t.Errorf("persistent flag %q not found", flagName)
		}
	}

	// Verify default values
	configFlag := rootCmd.PersistentFlags().Lookup("config")
	if configFlag.DefValue != "" {
		t.Errorf("config default = %q, want empty string", configFlag.DefValue)
	}

	outputFlag := rootCmd.PersistentFlags().Lookup("output")
	if outputFlag.DefValue != "text" {
		t.Errorf("output default = %q, want %q", outputFlag.DefValue, "text")
	}

	verboseFlag := rootCmd.PersistentFlags().Lookup("verbose")
	if verboseFlag.DefValue != "false" {
		t.Errorf("verbose default = %q, want %q", verboseFlag.DefValue, "false")
	}

	quietFlag := rootCmd.PersistentFlags().Lookup("quiet")
	if quietFlag.DefValue != "false" {
		t.Errorf("quiet default = %q, want %q", quietFlag.DefValue, "false")
	}
}

func TestMutuallyExclusiveFlags(t *testing.T) {
	resetRootCmd()
	resetFlags()

	rootCmd.SetArgs([]string{"-v", "-q"})
	err := rootCmd.Execute()

	if err == nil {
		t.Error("Expected error when using --verbose and --quiet together, got nil")
	}

	expectedErr := "--verbose and --quiet flags cannot be used together"
	if err.Error() != expectedErr {
		t.Errorf("Expected error message %q, got %q", expectedErr, err.Error())
	}
}
