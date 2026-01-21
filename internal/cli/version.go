// internal/cli/version.go
package cli

import (
	"encoding/json"
	"fmt"

	"github.com/spf13/cobra"
)

var (
	version = "1.2.3"
	date    = "01-01-1970"
	commit  = "abCDeFgh"
	branch  = "devbranch"
)

var versionCmd = &cobra.Command{
	Use:   "version",
	Short: "Print version information",
	Run: func(cmd *cobra.Command, args []string) {
		if GetOutputFormat() == "json" {
			data := map[string]string{
				"version":    version,
				"commit":     commit,
				"build_date": date,
				"branch":     branch,
			}
			out, _ := json.MarshalIndent(data, "", "  ")
			fmt.Fprintln(cmd.OutOrStdout(), string(out))
		} else {
			fmt.Fprintf(cmd.OutOrStdout(), "gsbt %s (%s: %s) %s\n", version, branch, commit, date)
		}
	},
}

func init() {
	rootCmd.AddCommand(versionCmd)
}
