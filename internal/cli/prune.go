// internal/cli/prune.go
package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

var (
	pruneServer string
	pruneDryRun bool
)

var pruneCmd = &cobra.Command{
	Use:   "prune",
	Short: "Remove old backups",
	Long:  `Delete backups older than the configured prune_age.`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Fprintln(cmd.OutOrStdout(), "prune command - not yet implemented")
	},
}

func init() {
	pruneCmd.Flags().StringVar(&pruneServer, "server", "", "prune specific server only")
	pruneCmd.Flags().BoolVar(&pruneDryRun, "dry-run", false, "show what would be deleted")
	rootCmd.AddCommand(pruneCmd)
}
