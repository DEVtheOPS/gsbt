// internal/connector/ftp_test.go
package connector

import (
	"testing"
)

func TestNewFTPConnector(t *testing.T) {
	cfg := Config{
		Type:       "ftp",
		Host:       "localhost",
		Port:       21,
		Username:   "user",
		Password:   "pass",
		RemotePath: "/saves/",
		Include:    []string{"*"},
		Exclude:    []string{"*.log"},
	}

	conn := NewFTPConnector(cfg)
	if conn == nil {
		t.Fatal("NewFTPConnector returned nil")
	}

	if conn.Name() != "ftp://localhost:21" {
		t.Errorf("unexpected name: %s", conn.Name())
	}
}
