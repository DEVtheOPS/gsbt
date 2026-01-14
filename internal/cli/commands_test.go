// internal/cli/commands_test.go
package cli

import (
	"bytes"
	"strings"
	"testing"
)

// TestBackupCommandMetadata tests backup command structure
func TestBackupCommandMetadata(t *testing.T) {
	if backupCmd.Use != "backup" {
		t.Errorf("backupCmd.Use = %q, want %q", backupCmd.Use, "backup")
	}

	if backupCmd.Short != "Backup gameserver files" {
		t.Errorf("backupCmd.Short = %q, want %q", backupCmd.Short, "Backup gameserver files")
	}

	expectedLong := "Download and archive files from configured gameservers."
	if backupCmd.Long != expectedLong {
		t.Errorf("backupCmd.Long = %q, want %q", backupCmd.Long, expectedLong)
	}
}

// TestBackupCommandFlags tests backup command flags
func TestBackupCommandFlags(t *testing.T) {
	// Check --server flag
	serverFlag := backupCmd.Flags().Lookup("server")
	if serverFlag == nil {
		t.Fatal("backup command missing --server flag")
	}
	if serverFlag.DefValue != "" {
		t.Errorf("--server default = %q, want empty string", serverFlag.DefValue)
	}

	// Check --sequential flag
	sequentialFlag := backupCmd.Flags().Lookup("sequential")
	if sequentialFlag == nil {
		t.Fatal("backup command missing --sequential flag")
	}
	if sequentialFlag.DefValue != "false" {
		t.Errorf("--sequential default = %q, want %q", sequentialFlag.DefValue, "false")
	}
}

// TestBackupCommandStub tests backup command stub output
func TestBackupCommandStub(t *testing.T) {
	resetRootCmd()
	resetFlags()
	rootCmd.AddCommand(backupCmd)

	buf := new(bytes.Buffer)
	rootCmd.SetOut(buf)
	rootCmd.SetErr(buf)
	rootCmd.SetArgs([]string{"backup"})

	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("backup command failed: %v", err)
	}

	output := buf.String()
	expected := "backup command - not yet implemented"
	if !strings.Contains(output, expected) {
		t.Errorf("backup output missing %q\nGot: %s", expected, output)
	}
}

// TestBackupCommandHelp tests backup command help
func TestBackupCommandHelp(t *testing.T) {
	resetRootCmd()
	resetFlags()
	rootCmd.AddCommand(backupCmd)

	buf := new(bytes.Buffer)
	rootCmd.SetOut(buf)
	rootCmd.SetErr(buf)
	rootCmd.SetArgs([]string{"backup", "--help"})

	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("backup --help failed: %v", err)
	}

	output := buf.String()

	expectedStrings := []string{
		"backup",
		"Download and archive files from configured gameservers",
		"--server",
		"--sequential",
		"backup specific server only",
		"run backups sequentially",
	}

	for _, expected := range expectedStrings {
		if !strings.Contains(output, expected) {
			t.Errorf("backup help missing %q\nFull output: %s", expected, output)
		}
	}
}

// TestPruneCommandMetadata tests prune command structure
func TestPruneCommandMetadata(t *testing.T) {
	if pruneCmd.Use != "prune" {
		t.Errorf("pruneCmd.Use = %q, want %q", pruneCmd.Use, "prune")
	}

	if pruneCmd.Short != "Remove old backups" {
		t.Errorf("pruneCmd.Short = %q, want %q", pruneCmd.Short, "Remove old backups")
	}

	expectedLong := "Delete backups older than the configured prune_age."
	if pruneCmd.Long != expectedLong {
		t.Errorf("pruneCmd.Long = %q, want %q", pruneCmd.Long, expectedLong)
	}
}

// TestPruneCommandFlags tests prune command flags
func TestPruneCommandFlags(t *testing.T) {
	// Check --server flag
	serverFlag := pruneCmd.Flags().Lookup("server")
	if serverFlag == nil {
		t.Fatal("prune command missing --server flag")
	}
	if serverFlag.DefValue != "" {
		t.Errorf("--server default = %q, want empty string", serverFlag.DefValue)
	}

	// Check --dry-run flag
	dryRunFlag := pruneCmd.Flags().Lookup("dry-run")
	if dryRunFlag == nil {
		t.Fatal("prune command missing --dry-run flag")
	}
	if dryRunFlag.DefValue != "false" {
		t.Errorf("--dry-run default = %q, want %q", dryRunFlag.DefValue, "false")
	}
}

// TestPruneCommandStub tests prune command stub output
func TestPruneCommandStub(t *testing.T) {
	resetRootCmd()
	resetFlags()
	rootCmd.AddCommand(pruneCmd)

	buf := new(bytes.Buffer)
	rootCmd.SetOut(buf)
	rootCmd.SetErr(buf)
	rootCmd.SetArgs([]string{"prune"})

	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("prune command failed: %v", err)
	}

	output := buf.String()
	expected := "prune command - not yet implemented"
	if !strings.Contains(output, expected) {
		t.Errorf("prune output missing %q\nGot: %s", expected, output)
	}
}

// TestPruneCommandHelp tests prune command help
func TestPruneCommandHelp(t *testing.T) {
	resetRootCmd()
	resetFlags()
	rootCmd.AddCommand(pruneCmd)

	buf := new(bytes.Buffer)
	rootCmd.SetOut(buf)
	rootCmd.SetErr(buf)
	rootCmd.SetArgs([]string{"prune", "--help"})

	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("prune --help failed: %v", err)
	}

	output := buf.String()
	expectedStrings := []string{
		"prune",
		"Delete backups older than the configured prune_age",
		"--server",
		"--dry-run",
		"prune specific server only",
		"show what would be deleted",
	}

	for _, expected := range expectedStrings {
		if !strings.Contains(output, expected) {
			t.Errorf("prune help missing %q\nFull output: %s", expected, output)
		}
	}
}

// TestListCommandMetadata tests list command structure
func TestListCommandMetadata(t *testing.T) {
	if listCmd.Use != "list" {
		t.Errorf("listCmd.Use = %q, want %q", listCmd.Use, "list")
	}

	if listCmd.Short != "List configured servers" {
		t.Errorf("listCmd.Short = %q, want %q", listCmd.Short, "List configured servers")
	}

	expectedLong := "Show configured servers and their backup status."
	if listCmd.Long != expectedLong {
		t.Errorf("listCmd.Long = %q, want %q", listCmd.Long, expectedLong)
	}
}

// TestListCommandFlags tests list command flags
func TestListCommandFlags(t *testing.T) {
	// Check --server flag
	serverFlag := listCmd.Flags().Lookup("server")
	if serverFlag == nil {
		t.Fatal("list command missing --server flag")
	}
	if serverFlag.DefValue != "" {
		t.Errorf("--server default = %q, want empty string", serverFlag.DefValue)
	}
}

// TestListCommandStub tests list command stub output
func TestListCommandStub(t *testing.T) {
	resetRootCmd()
	resetFlags()
	rootCmd.AddCommand(listCmd)

	buf := new(bytes.Buffer)
	rootCmd.SetOut(buf)
	rootCmd.SetErr(buf)
	rootCmd.SetArgs([]string{"list"})

	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("list command failed: %v", err)
	}

	output := buf.String()
	expected := "list command - not yet implemented"
	if !strings.Contains(output, expected) {
		t.Errorf("list output missing %q\nGot: %s", expected, output)
	}
}

// TestListCommandHelp tests list command help
func TestListCommandHelp(t *testing.T) {
	resetRootCmd()
	resetFlags()
	rootCmd.AddCommand(listCmd)

	buf := new(bytes.Buffer)
	rootCmd.SetOut(buf)
	rootCmd.SetErr(buf)
	rootCmd.SetArgs([]string{"list", "--help"})

	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("list --help failed: %v", err)
	}

	output := buf.String()
	expectedStrings := []string{
		"list",
		"Show configured servers and their backup status",
		"--server",
		"show specific server details",
	}

	for _, expected := range expectedStrings {
		if !strings.Contains(output, expected) {
			t.Errorf("list help missing %q\nFull output: %s", expected, output)
		}
	}
}

// TestRestoreCommandMetadata tests restore command structure
func TestRestoreCommandMetadata(t *testing.T) {
	if restoreCmd.Use != "restore <backup-file>" {
		t.Errorf("restoreCmd.Use = %q, want %q", restoreCmd.Use, "restore <backup-file>")
	}

	if restoreCmd.Short != "Restore a backup" {
		t.Errorf("restoreCmd.Short = %q, want %q", restoreCmd.Short, "Restore a backup")
	}

	expectedLong := "Restore a backup to a server or extract locally."
	if restoreCmd.Long != expectedLong {
		t.Errorf("restoreCmd.Long = %q, want %q", restoreCmd.Long, expectedLong)
	}
}

// TestRestoreCommandFlags tests restore command flags
func TestRestoreCommandFlags(t *testing.T) {
	// Check --server flag
	serverFlag := restoreCmd.Flags().Lookup("server")
	if serverFlag == nil {
		t.Fatal("restore command missing --server flag")
	}
	if serverFlag.DefValue != "" {
		t.Errorf("--server default = %q, want empty string", serverFlag.DefValue)
	}

	// Check --local flag
	localFlag := restoreCmd.Flags().Lookup("local")
	if localFlag == nil {
		t.Fatal("restore command missing --local flag")
	}
	if localFlag.DefValue != "" {
		t.Errorf("--local default = %q, want empty string", localFlag.DefValue)
	}

	// Check --dry-run flag
	dryRunFlag := restoreCmd.Flags().Lookup("dry-run")
	if dryRunFlag == nil {
		t.Fatal("restore command missing --dry-run flag")
	}
	if dryRunFlag.DefValue != "false" {
		t.Errorf("--dry-run default = %q, want %q", dryRunFlag.DefValue, "false")
	}

	// Check --force flag
	forceFlag := restoreCmd.Flags().Lookup("force")
	if forceFlag == nil {
		t.Fatal("restore command missing --force flag")
	}
	if forceFlag.DefValue != "false" {
		t.Errorf("--force default = %q, want %q", forceFlag.DefValue, "false")
	}
}

// TestRestoreCommandStub tests restore command stub output with argument
func TestRestoreCommandStub(t *testing.T) {
	resetRootCmd()
	resetFlags()
	rootCmd.AddCommand(restoreCmd)

	buf := new(bytes.Buffer)
	rootCmd.SetOut(buf)
	rootCmd.SetErr(buf)
	rootCmd.SetArgs([]string{"restore", "backup.tar.gz"})

	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("restore command failed: %v", err)
	}

	output := buf.String()
	expected := "restore command - not yet implemented (file: backup.tar.gz)"
	if !strings.Contains(output, expected) {
		t.Errorf("restore output missing %q\nGot: %s", expected, output)
	}
}

// TestRestoreCommandRequiresArg tests restore command requires exactly one argument
func TestRestoreCommandRequiresArg(t *testing.T) {
	tests := []struct {
		name      string
		args      []string
		wantError bool
	}{
		{
			name:      "no args",
			args:      []string{"restore"},
			wantError: true,
		},
		{
			name:      "one arg",
			args:      []string{"restore", "backup.tar.gz"},
			wantError: false,
		},
		{
			name:      "too many args",
			args:      []string{"restore", "backup1.tar.gz", "backup2.tar.gz"},
			wantError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			resetRootCmd()
			resetFlags()
			rootCmd.AddCommand(restoreCmd)

			buf := new(bytes.Buffer)
			rootCmd.SetOut(buf)
			rootCmd.SetErr(buf)
			rootCmd.SetArgs(tt.args)

			err := rootCmd.Execute()

			if tt.wantError && err == nil {
				t.Error("expected error but got none")
			}
			if !tt.wantError && err != nil {
				t.Errorf("unexpected error: %v", err)
			}
		})
	}
}

// TestRestoreCommandHelp tests restore command help
func TestRestoreCommandHelp(t *testing.T) {
	resetRootCmd()
	resetFlags()
	rootCmd.AddCommand(restoreCmd)

	buf := new(bytes.Buffer)
	rootCmd.SetOut(buf)
	rootCmd.SetErr(buf)
	rootCmd.SetArgs([]string{"restore", "--help"})

	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("restore --help failed: %v", err)
	}

	output := buf.String()
	expectedStrings := []string{
		"restore",
		"Restore a backup",
		"--server",
		"--local",
		"--dry-run",
		"--force",
		"restore to server",
		"extract to local path",
		"show what would be restored",
		"skip confirmation prompt",
	}

	for _, expected := range expectedStrings {
		if !strings.Contains(output, expected) {
			t.Errorf("restore help missing %q", expected)
		}
	}
}

// TestAllCommandsRegistered tests that all commands are registered with root
func TestAllCommandsRegistered(t *testing.T) {
	resetRootCmd()
	resetFlags()

	// Add all commands
	rootCmd.AddCommand(versionCmd)
	rootCmd.AddCommand(backupCmd)
	rootCmd.AddCommand(pruneCmd)
	rootCmd.AddCommand(listCmd)
	rootCmd.AddCommand(restoreCmd)

	expectedCommands := []string{"version", "backup", "prune", "list", "restore"}

	for _, cmdName := range expectedCommands {
		found := false
		for _, cmd := range rootCmd.Commands() {
			if cmd.Name() == cmdName {
				found = true
				break
			}
		}
		if !found {
			t.Errorf("command %q not registered with root command", cmdName)
		}
	}
}

// TestRootHelpShowsAllCommands tests that root help shows all commands
func TestRootHelpShowsAllCommands(t *testing.T) {
	resetRootCmd()
	resetFlags()

	// Add all commands
	rootCmd.AddCommand(versionCmd)
	rootCmd.AddCommand(backupCmd)
	rootCmd.AddCommand(pruneCmd)
	rootCmd.AddCommand(listCmd)
	rootCmd.AddCommand(restoreCmd)

	buf := new(bytes.Buffer)
	rootCmd.SetOut(buf)
	rootCmd.SetErr(buf)
	rootCmd.SetArgs([]string{"--help"})

	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("root --help failed: %v", err)
	}

	output := buf.String()
	expectedCommands := []string{"version", "backup", "prune", "list", "restore"}

	for _, cmdName := range expectedCommands {
		if !strings.Contains(output, cmdName) {
			t.Errorf("root help missing command %q", cmdName)
		}
	}
}
