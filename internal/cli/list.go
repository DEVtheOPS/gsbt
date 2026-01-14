// internal/cli/list.go
package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

var listServer string

var listCmd = &cobra.Command{
	Use:   "list",
	Short: "List configured servers",
	Long:  `Show configured servers and their backup status.`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Fprintln(cmd.OutOrStdout(), "list command - not yet implemented")
	},
}

func init() {
	listCmd.Flags().StringVar(&listServer, "server", "", "show specific server details")
	rootCmd.AddCommand(listCmd)
}
