// internal/cli/version.go
package cli

import (
	"encoding/json"
	"fmt"

	"github.com/devtheops/gsbt/internal/version"
	"github.com/spf13/cobra"
)

var versionCmd = &cobra.Command{
	Use:   "version",
	Short: "Print version information",
	Run: func(cmd *cobra.Command, args []string) {
		if GetOutputFormat() == "json" {
			data := map[string]string{
				"version":    version.Version,
				"commit":     version.Commit,
				"build_date": version.BuildDate,
			}
			out, _ := json.MarshalIndent(data, "", "  ")
			fmt.Fprintln(cmd.OutOrStdout(), string(out))
		} else {
			fmt.Fprintf(cmd.OutOrStdout(), "gsbt %s\n", version.Version)
			fmt.Fprintf(cmd.OutOrStdout(), "  commit: %s\n", version.Commit)
			fmt.Fprintf(cmd.OutOrStdout(), "  built:  %s\n", version.BuildDate)
		}
	},
}

func init() {
	rootCmd.AddCommand(versionCmd)
}
