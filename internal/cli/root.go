// internal/cli/root.go
package cli

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var (
	cfgFile   string
	outputFmt string
	verbose   bool
	quiet     bool
)

var rootCmd = &cobra.Command{
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

func Execute() {
	err := rootCmd.Execute()
	if err != nil {
		os.Exit(1)
	}
}

func init() {
	rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "", "config file path")
	rootCmd.PersistentFlags().StringVar(&outputFmt, "output", "text", "output format (text, json, rich)")
	rootCmd.PersistentFlags().BoolVarP(&verbose, "verbose", "v", false, "verbose output")
	rootCmd.PersistentFlags().BoolVarP(&quiet, "quiet", "q", false, "suppress non-error output")
}

// GetConfigFile returns the config file path from flags
func GetConfigFile() string {
	return cfgFile
}

// GetOutputFormat returns the output format
func GetOutputFormat() string {
	return outputFmt
}

// IsVerbose returns verbose flag state
func IsVerbose() bool {
	return verbose
}

// IsQuiet returns quiet flag state
func IsQuiet() bool {
	return quiet
}
