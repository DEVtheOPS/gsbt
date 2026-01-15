// internal/cli/backup.go
package cli

import (
    "context"
    "fmt"
    "sync"
    "time"

    "github.com/devtheops/gsbt/internal/backup"
    "github.com/devtheops/gsbt/internal/config"
    "github.com/devtheops/gsbt/internal/connector"
    "github.com/devtheops/gsbt/internal/log"
    "github.com/devtheops/gsbt/internal/progress"
    "github.com/spf13/cobra"
)

var (
	backupServer     string
	backupSequential bool
)

// allow tests to inject mocks
var newConnector = connector.NewConnector

var backupCmd = &cobra.Command{
	Use:   "backup",
	Short: "Backup gameserver files",
	Long:  `Download and archive files from configured gameservers.`,
	RunE: func(cmd *cobra.Command, args []string) error {
		ctx := context.Background()
		return runBackup(ctx, cmd)
	},
}

func init() {
	backupCmd.Flags().StringVar(&backupServer, "server", "", "backup specific server only")
	backupCmd.Flags().BoolVar(&backupSequential, "sequential", false, "run backups sequentially")
	rootCmd.AddCommand(backupCmd)
}

func runBackup(ctx context.Context, cmd *cobra.Command) error {
	// Setup logger
	logger := log.NewWithWriters(cmd.OutOrStdout(), cmd.ErrOrStderr())
	logger.SetOutputFormat(GetOutputFormat())
	logger.SetQuiet(IsQuiet())
	logger.SetVerbose(IsVerbose())

	cfgPath, err := config.FindConfigFile(GetConfigFile())
	if err != nil {
		return err
	}

	cfg, err := config.LoadConfig(cfgPath)
	if err != nil {
		return err
	}

	servers := cfg.Servers
	if backupServer != "" {
		filtered := make([]config.Server, 0, 1)
		for _, s := range servers {
			if s.Name == backupServer {
				filtered = append(filtered, s)
			}
		}
		if len(filtered) == 0 {
			return fmt.Errorf("server %q not found in config", backupServer)
		}
		servers = filtered
	}

	if len(servers) == 0 {
		return fmt.Errorf("no servers configured")
	}

    successes := 0
    failures := 0

    type result struct {
        success bool
        err     error
    }

    runOne := func(srv config.Server) result {
        serverLogger := logger.WithPrefix(fmt.Sprintf("[bold][cyan]%s[/cyan][/bold]", srv.Name))
        serverLogger.Info("[yellow]starting backup[/yellow]")

        connCfg, err := toConnectorConfig(srv, cfg.Defaults)
        if err != nil {
            serverLogger.Error(fmt.Sprintf("[red]config error:[/red] %v", err))
            return result{err: err}
        }

        conn, err := newConnector(connCfg)
        if err != nil {
            serverLogger.Error(fmt.Sprintf("[red]init error:[/red] %v", err))
            return result{err: err}
        }

        mgr := backup.Manager{
            BackupLocation: srv.GetBackupLocation(cfg.Defaults),
            TempDir:        cfg.Defaults.TempDir,
            Progress:       progress.New(serverLogger, GetOutputFormat()),
        }

        start := time.Now()
        archivePath, stats, err := mgr.Backup(ctx, conn)
        if err != nil {
            serverLogger.Error(fmt.Sprintf("[red]backup failed:[/red] %v", err))
            return result{err: err}
        }

        serverLogger.Info(fmt.Sprintf("[green]saved[/green] %s (%d files, %.1f MB, %.1fs)",
            archivePath, stats.Files, float64(stats.Bytes)/1e6, time.Since(start).Seconds()),
            log.Meta{
                "archive_path": archivePath,
                "files":        stats.Files,
                "bytes":        stats.Bytes,
                "duration_sec": time.Since(start).Seconds(),
            })

        return result{success: true}
    }

    if backupSequential || len(servers) == 1 {
        for _, srv := range servers {
            res := runOne(srv)
            if res.success {
                successes++
            } else {
                failures++
            }
        }
    } else {
        var wg sync.WaitGroup
        results := make(chan result, len(servers))

        for _, srv := range servers {
            srv := srv
            wg.Add(1)
            go func() {
                defer wg.Done()
                results <- runOne(srv)
            }()
        }

        wg.Wait()
        close(results)
        for res := range results {
            if res.success {
                successes++
            } else {
                failures++
            }
        }
    }

    if failures > 0 {
        return fmt.Errorf("backup complete with failures: %d success, %d failed", successes, failures)
    }

    logger.Info(fmt.Sprintf("[bold][green]backup complete[/green][/bold] (%d success)", successes))

    return nil
}

func toConnectorConfig(s config.Server, defaults config.Defaults) (connector.Config, error) {
	conn := s.Connection

	// Populate default include/exclude
	include := conn.GetInclude()
	exclude := conn.Exclude

	// Default API key for nitrado
	apiKey := conn.APIKey
	if apiKey == "" {
		apiKey = defaults.NitradoAPIKey
	}

	cfg := connector.Config{
		Type:          conn.Type,
		Host:          conn.Host,
		Port:          conn.Port,
		Username:      conn.Username,
		Password:      conn.Password,
		KeyFile:       conn.KeyFile,
		Passive:       conn.IsPassive(),
		TLS:           conn.TLS,
		APIKey:        apiKey,
		ServiceID:     conn.ServiceID,
		RemotePath:    conn.RemotePath,
		Include:       include,
		Exclude:       exclude,
		RetryAttempts: defaults.RetryAttempts,
		RetryDelay:    defaults.RetryDelay,
		RetryBackoff:  defaults.RetryBackoff,
	}

	if cfg.RemotePath == "" {
		return cfg, fmt.Errorf("connection.remote_path is required")
	}

	return cfg, nil
}
