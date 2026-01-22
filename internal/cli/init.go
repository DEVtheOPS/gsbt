// internal/cli/init.go
package cli

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/spf13/cobra"
)

var (
	outFile string
	force   bool
)

var initCmd = &cobra.Command{
	Use:   "init",
	Short: "Initialize a new configuration file",
	Long:  `Creates a new example configuration file with defaults and comments.`,
	RunE: func(cmd *cobra.Command, args []string) error {
		// content is the sample configuration
		// We point to the latest release schema by default
		content := `# yaml-language-server: $schema=https://github.com/devtheops/gsbt/releases/latest/download/gsbt.schema.json

defaults:
  backup_location: ./backups
  temp_dir: ./.tmp
  prune_age: 30
  retry_attempts: 3
  retry_delay: 5
  retry_backoff: true
  # nitrado_api_key: ${NITRADO_API_KEY}

servers:
  - name: example-ftp-server
    description: "An example FTP server backup"
    connection:
      type: ftp
      host: ftp.example.com
      port: 21
      username: user
      password: ${FTP_PASSWORD}
      remote_path: /game/saves
      include: ["*"]
      exclude: ["*.log", "Logs/"]

  # - name: example-nitrado-server
  #   connection:
  #     type: nitrado
  #     service_id: "1234567"
  #     remote_path: /games/ark/saves
`

		// Check if file exists to avoid accidental overwrite
		if _, err := os.Stat(outFile); err == nil && !force {
			return fmt.Errorf("file %s already exists; use --force to overwrite", outFile)
		}

		// Ensure directory exists
		dir := filepath.Dir(outFile)
		if dir != "." {
			if err := os.MkdirAll(dir, 0755); err != nil {
				return fmt.Errorf("failed to create directory: %w", err)
			}
		}

		if err := os.WriteFile(outFile, []byte(content), 0644); err != nil {
			return fmt.Errorf("failed to write config file: %w", err)
		}

		fmt.Printf("Configuration initialized at %s\n", outFile)
		return nil
	},
}

func init() {
	rootCmd.AddCommand(initCmd)
	initCmd.Flags().StringVarP(&outFile, "outfile", "o", ".gsbt-config.yml", "output file path")
	initCmd.Flags().BoolVarP(&force, "force", "f", false, "overwrite existing config file")
}
