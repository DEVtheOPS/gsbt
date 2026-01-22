package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"github.com/devtheops/gsbt/internal/config"
	"github.com/invopop/jsonschema"
)

func main() {
	// Generate schema from the Config struct
	r := new(jsonschema.Reflector)
	r.DoNotReference = true // Embed definitions for a self-contained schema
	r.FieldNameTag = "yaml" // Use yaml tags for property names
	
	// Enable comments for descriptions
	// We assume we are running from the package directory or project root
	if err := r.AddGoComments("github.com/devtheops/gsbt", "."); err != nil {
		// Fallback to project root if . doesn't work (useful for local runs)
		_ = r.AddGoComments("github.com/devtheops/gsbt", "./internal/config")
	}
	
	// Add custom descriptions/titles if needed via tags in the struct,
	// or programmatically here if strictly necessary.
	// For now, we rely on the struct tags and comments.

	schema := r.Reflect(&config.Config{})
	
	// Set schema metadata
	schema.ID = "https://github.com/devtheops/gsbt/gsbt.schema.json"
	schema.Title = "GSBT Configuration"
	schema.Description = "Configuration schema for the Gameserver Backup Tool (gsbt)"

	// Marshal to JSON with indentation
	data, err := json.MarshalIndent(schema, "", "  ")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error marshaling schema: %v\n", err)
		os.Exit(1)
	}

	// Write to file
	outputPath := "gsbt.schema.json"
	if len(os.Args) > 1 {
		outputPath = os.Args[1]
	}
	
	// Ensure we are writing to the project root if running from elsewhere, 
	// unless absolute path is given.
	absPath, err := filepath.Abs(outputPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error resolving path: %v\n", err)
		os.Exit(1)
	}

	if err := os.WriteFile(absPath, data, 0644); err != nil {
		fmt.Fprintf(os.Stderr, "Error writing schema file: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("Schema successfully generated at %s\n", absPath)
}
