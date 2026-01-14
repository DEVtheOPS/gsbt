// internal/connector/factory.go
package connector

import "fmt"

// NewConnector instantiates the correct connector implementation based on cfg.Type.
func NewConnector(cfg Config) (Connector, error) {
	switch cfg.Type {
	case "ftp":
		return NewFTPConnector(cfg), nil
	case "sftp":
		return NewSFTPConnector(cfg), nil
	case "nitrado":
		return NewNitradoConnector(cfg), nil
	default:
		return nil, fmt.Errorf("unsupported connector type: %s", cfg.Type)
	}
}
