// internal/connector/nitrado.go
package connector

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

const nitradoAPIBase = "https://api.nitrado.net"

// NitradoConnector implements Connector for Nitrado game servers
// It fetches FTP credentials from Nitrado API and delegates to FTPConnector
type NitradoConnector struct {
	config     Config
	ftp        *FTPConnector
	apiKey     string
	serviceID  string
	apiBase    string
	httpClient *http.Client
}

// nitradoFTPResponse represents the Nitrado API response for FTP credentials
type nitradoFTPResponse struct {
	Status string `json:"status"`
	Data   struct {
		FTP struct {
			Hostname string `json:"hostname"`
			Port     int    `json:"port"`
			Username string `json:"username"`
			Password string `json:"password"`
		} `json:"ftp"`
	} `json:"data"`
	Message string `json:"message"`
}

// NewNitradoConnector creates a new Nitrado connector
func NewNitradoConnector(cfg Config) *NitradoConnector {
	return &NitradoConnector{
		config:     cfg,
		apiKey:     cfg.APIKey,
		serviceID:  cfg.ServiceID,
		apiBase:    nitradoAPIBase,
		httpClient: &http.Client{Timeout: 30 * time.Second},
	}
}

// Name returns the connector name for logging
func (n *NitradoConnector) Name() string {
	return fmt.Sprintf("nitrado://%s", n.serviceID)
}

// Connect fetches FTP credentials from Nitrado API and establishes FTP connection
func (n *NitradoConnector) Connect(ctx context.Context) error {
	if n.apiKey == "" {
		return fmt.Errorf("api_key is required for nitrado connector")
	}

	if n.serviceID == "" {
		return fmt.Errorf("service_id is required for nitrado connector")
	}

	// Fetch FTP credentials from Nitrado API
	creds, err := n.fetchFTPCredentials(ctx)
	if err != nil {
		return fmt.Errorf("failed to get Nitrado FTP credentials: %w", err)
	}

	// Create FTP connector with retrieved credentials
	ftpConfig := Config{
		Type:       "ftp",
		Host:       creds.Hostname,
		Port:       creds.Port,
		Username:   creds.Username,
		Password:   creds.Password,
		RemotePath: n.config.RemotePath,
		Include:    n.config.Include,
		Exclude:    n.config.Exclude,
		Passive:    true,

		RetryAttempts: n.config.RetryAttempts,
		RetryDelay:    n.config.RetryDelay,
		RetryBackoff:  n.config.RetryBackoff,
	}

	n.ftp = NewFTPConnector(ftpConfig)
	return n.ftp.Connect(ctx)
}

type ftpCredentials struct {
	Hostname string
	Port     int
	Username string
	Password string
}

func (n *NitradoConnector) fetchFTPCredentials(ctx context.Context) (*ftpCredentials, error) {
	client := n.httpClient
	if client == nil {
		client = &http.Client{Timeout: 30 * time.Second}
		n.httpClient = client
	}

	url := fmt.Sprintf("%s/services/%s/gameservers", n.apiBase, n.serviceID)

	req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
	if err != nil {
		return nil, err
	}

	req.Header.Set("Authorization", "Bearer "+n.apiKey)
	req.Header.Set("Accept", "application/json")

	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	// Handle rate limiting
	if resp.StatusCode == 429 {
		retryAfter := resp.Header.Get("Retry-After")
		return nil, fmt.Errorf("rate limited by Nitrado API (retry after: %s)", retryAfter)
	}

	if resp.StatusCode != 200 {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("Nitrado API error (status %d): %s", resp.StatusCode, string(body))
	}

	var apiResp nitradoFTPResponse
	if err := json.NewDecoder(resp.Body).Decode(&apiResp); err != nil {
		return nil, fmt.Errorf("failed to parse Nitrado response: %w", err)
	}

	if apiResp.Status != "success" {
		return nil, fmt.Errorf("Nitrado API returned error: %s", apiResp.Message)
	}

	return &ftpCredentials{
		Hostname: apiResp.Data.FTP.Hostname,
		Port:     apiResp.Data.FTP.Port,
		Username: apiResp.Data.FTP.Username,
		Password: apiResp.Data.FTP.Password,
	}, nil
}

// List delegates to FTP connector
func (n *NitradoConnector) List(ctx context.Context) ([]FileInfo, error) {
	if n.ftp == nil {
		return nil, fmt.Errorf("not connected")
	}
	return n.ftp.List(ctx)
}

// Download delegates to FTP connector
func (n *NitradoConnector) Download(ctx context.Context, remotePath string, w io.Writer) error {
	if n.ftp == nil {
		return fmt.Errorf("not connected")
	}
	return n.ftp.Download(ctx, remotePath, w)
}

// Upload delegates to FTP connector
func (n *NitradoConnector) Upload(ctx context.Context, r io.Reader, remotePath string) error {
	if n.ftp == nil {
		return fmt.Errorf("not connected")
	}
	return n.ftp.Upload(ctx, r, remotePath)
}

// Close terminates the FTP connection
func (n *NitradoConnector) Close() error {
	if n.ftp != nil {
		return n.ftp.Close()
	}
	return nil
}
