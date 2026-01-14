// internal/cli/backup.go
package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

var (
	backupServer     string
	backupSequential bool
)

var backupCmd = &cobra.Command{
	Use:   "backup",
	Short: "Backup gameserver files",
	Long:  `Download and archive files from configured gameservers.`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Fprintln(cmd.OutOrStdout(), "backup command - not yet implemented")
	},
}

func init() {
	backupCmd.Flags().StringVar(&backupServer, "server", "", "backup specific server only")
	backupCmd.Flags().BoolVar(&backupSequential, "sequential", false, "run backups sequentially")
	rootCmd.AddCommand(backupCmd)
}
