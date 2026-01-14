// internal/cli/restore.go
package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

var (
	restoreServer string
	restoreLocal  string
	restoreDryRun bool
	restoreForce  bool
)

var restoreCmd = &cobra.Command{
	Use:   "restore <backup-file>",
	Short: "Restore a backup",
	Long:  `Restore a backup to a server or extract locally.`,
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Fprintf(cmd.OutOrStdout(), "restore command - not yet implemented (file: %s)\n", args[0])
	},
}

func init() {
	restoreCmd.Flags().StringVar(&restoreServer, "server", "", "restore to server")
	restoreCmd.Flags().StringVar(&restoreLocal, "local", "", "extract to local path")
	restoreCmd.Flags().BoolVar(&restoreDryRun, "dry-run", false, "show what would be restored")
	restoreCmd.Flags().BoolVar(&restoreForce, "force", false, "skip confirmation prompt")
	rootCmd.AddCommand(restoreCmd)
}
