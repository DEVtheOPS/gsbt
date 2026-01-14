// internal/connector/sftp_test.go
package connector

import (
	"testing"
)

func TestNewSFTPConnector(t *testing.T) {
	cfg := Config{
		Type:       "sftp",
		Host:       "localhost",
		Port:       22,
		Username:   "user",
		Password:   "pass",
		RemotePath: "/home/user/saves/",
		Include:    []string{"*"},
		Exclude:    []string{"*.log"},
	}

	conn := NewSFTPConnector(cfg)
	if conn == nil {
		t.Fatal("NewSFTPConnector returned nil")
	}

	if conn.Name() != "sftp://localhost:22" {
		t.Errorf("unexpected name: %s", conn.Name())
	}
}

func TestNewSFTPConnectorWithKeyFile(t *testing.T) {
	cfg := Config{
		Type:       "sftp",
		Host:       "localhost",
		Port:       22,
		Username:   "user",
		KeyFile:    "/home/user/.ssh/id_rsa",
		RemotePath: "/saves/",
	}

	conn := NewSFTPConnector(cfg)
	if conn == nil {
		t.Fatal("NewSFTPConnector returned nil")
	}
}
