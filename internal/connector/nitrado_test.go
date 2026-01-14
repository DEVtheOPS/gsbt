// internal/connector/nitrado_test.go
package connector

import (
	"testing"
)

func TestNewNitradoConnector(t *testing.T) {
	cfg := Config{
		Type:       "nitrado",
		APIKey:     "test-api-key",
		ServiceID:  "12345",
		RemotePath: "/games/ark/",
		Include:    []string{"*"},
		Exclude:    []string{"*.log"},
	}

	conn := NewNitradoConnector(cfg)
	if conn == nil {
		t.Fatal("NewNitradoConnector returned nil")
	}

	if conn.Name() != "nitrado://12345" {
		t.Errorf("unexpected name: %s", conn.Name())
	}
}
