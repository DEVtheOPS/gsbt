// internal/connector/factory_test.go
package connector

import (
	"fmt"
	"testing"
)

func TestNewConnector(t *testing.T) {
	tests := []struct {
		name     string
		connType string
		wantType string
		wantErr  bool
	}{
		{"ftp", "ftp", "*connector.FTPConnector", false},
		{"sftp", "sftp", "*connector.SFTPConnector", false},
		{"nitrado", "nitrado", "*connector.NitradoConnector", false},
		{"unknown", "unknown", "", true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cfg := Config{Type: tt.connType}
			conn, err := NewConnector(cfg)

			if tt.wantErr {
				if err == nil {
					t.Fatalf("expected error for type %s, got nil", tt.connType)
				}
				return
			}

			if err != nil {
				t.Fatalf("unexpected error for type %s: %v", tt.connType, err)
			}

			typeName := fmt.Sprintf("%T", conn)
			if typeName != tt.wantType {
				t.Fatalf("connector type = %s, want %s", typeName, tt.wantType)
			}
		})
	}
}
