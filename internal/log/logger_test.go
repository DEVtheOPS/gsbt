// internal/log/logger_test.go
package log

import (
	"bytes"
	"encoding/json"
	"strings"
	"testing"
)

func TestLoggerTextMode(t *testing.T) {
	var out, errOut bytes.Buffer
	logger := NewWithWriters(&out, &errOut)
	logger.SetOutputFormat("text")

	logger.Info("plain message")
	if !strings.Contains(out.String(), "plain message") {
		t.Errorf("expected plain message in output, got: %s", out.String())
	}

	out.Reset()
	logger.Info("[bold]formatted[/bold] message")
	if strings.Contains(out.String(), "[bold]") {
		t.Errorf("expected markup stripped in text mode, got: %s", out.String())
	}
	if !strings.Contains(out.String(), "formatted message") {
		t.Errorf("expected stripped message, got: %s", out.String())
	}
}

func TestLoggerRichMode(t *testing.T) {
	var out bytes.Buffer
	logger := NewWithWriters(&out, &out)
	logger.SetOutputFormat("rich")

	logger.Info("[green]success[/green]")
	output := out.String()

	// Should contain ANSI codes
	if !strings.Contains(output, "\033[") {
		t.Errorf("expected ANSI codes in rich mode, got: %s", output)
	}

	// Should contain reset code
	if !strings.Contains(output, "\033[0m") {
		t.Errorf("expected reset code in rich mode, got: %s", output)
	}
}

func TestLoggerJSONMode(t *testing.T) {
	var out bytes.Buffer
	logger := NewWithWriters(&out, &out)
	logger.SetOutputFormat("json")

	logger.Info("[bold]test message[/bold]", Meta{"key": "value"})

	var entry map[string]interface{}
	if err := json.Unmarshal(out.Bytes(), &entry); err != nil {
		t.Fatalf("expected valid JSON, got error: %v, output: %s", err, out.String())
	}

	if entry["message"] != "test message" {
		t.Errorf("expected stripped message in JSON, got: %v", entry["message"])
	}

	if entry["level"] != "info" {
		t.Errorf("expected level=info, got: %v", entry["level"])
	}

	if meta, ok := entry["metadata"].(map[string]interface{}); !ok || meta["key"] != "value" {
		t.Errorf("expected metadata in JSON, got: %v", entry)
	}
}

func TestLoggerWithPrefix(t *testing.T) {
	var out bytes.Buffer
	logger := NewWithWriters(&out, &out)
	logger.SetOutputFormat("text")

	prefixed := logger.WithPrefix("[test-server]")
	prefixed.Info("message")

	if !strings.Contains(out.String(), "[test-server]") {
		t.Errorf("expected prefix in output, got: %s", out.String())
	}
	if !strings.Contains(out.String(), "message") {
		t.Errorf("expected message in output, got: %s", out.String())
	}
}

func TestLoggerQuietMode(t *testing.T) {
	var out, errOut bytes.Buffer
	logger := NewWithWriters(&out, &errOut)
	logger.SetQuiet(true)

	logger.Info("info message")
	if out.String() != "" {
		t.Errorf("expected no output in quiet mode for Info, got: %s", out.String())
	}

	logger.Error("error message")
	if !strings.Contains(errOut.String(), "error message") {
		t.Errorf("expected error message in quiet mode, got: %s", errOut.String())
	}
}

func TestLoggerVerboseMode(t *testing.T) {
	var out bytes.Buffer
	logger := NewWithWriters(&out, &out)
	logger.SetOutputFormat("text")
	logger.SetVerbose(true)

	logger.Debug("debug message")
	if !strings.Contains(out.String(), "debug message") {
		t.Errorf("expected debug message in verbose mode, got: %s", out.String())
	}
}

func TestLoggerMetadataVerbose(t *testing.T) {
	var out bytes.Buffer
	logger := NewWithWriters(&out, &out)
	logger.SetOutputFormat("text")
	logger.SetVerbose(true)

	logger.Info("test", Meta{"foo": "bar", "count": 42})

	output := out.String()
	if !strings.Contains(output, "foo=bar") {
		t.Errorf("expected metadata in verbose text mode, got: %s", output)
	}
	if !strings.Contains(output, "count=42") {
		t.Errorf("expected metadata in verbose text mode, got: %s", output)
	}
}

func TestLoggerMetadataNotVerbose(t *testing.T) {
	var out bytes.Buffer
	logger := NewWithWriters(&out, &out)
	logger.SetOutputFormat("text")
	logger.SetVerbose(false)

	logger.Info("test", Meta{"foo": "bar"})

	output := out.String()
	if strings.Contains(output, "foo=bar") {
		t.Errorf("expected no metadata in non-verbose text mode, got: %s", output)
	}
}

func TestLoggerLevels(t *testing.T) {
	var out, errOut bytes.Buffer
	logger := NewWithWriters(&out, &errOut)
	logger.SetOutputFormat("text")
	logger.SetLevel(WarnLevel)

	logger.Debug("debug")
	logger.Info("info")
	logger.Warn("warn")
	logger.Error("error")

	output := out.String()
	if strings.Contains(output, "debug") {
		t.Errorf("expected debug filtered out, got: %s", output)
	}
	if strings.Contains(output, "info") {
		t.Errorf("expected info filtered out, got: %s", output)
	}
	if !strings.Contains(output, "warn") {
		t.Errorf("expected warn in output, got: %s", output)
	}

	if !strings.Contains(errOut.String(), "error") {
		t.Errorf("expected error in error output, got: %s", errOut.String())
	}
}

func TestStripMarkup(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"plain text", "plain text"},
		{"[bold]bold text[/bold]", "bold text"},
		{"[green]colored[/green] text", "colored text"},
		{"[bold][red]nested[/red][/bold]", "nested"},
	}

	for _, tt := range tests {
		result := stripMarkup(tt.input)
		if result != tt.expected {
			t.Errorf("stripMarkup(%q) = %q, want %q", tt.input, result, tt.expected)
		}
	}
}

func TestRenderMarkup(t *testing.T) {
	tests := []struct {
		input    string
		contains []string
	}{
		{"[bold]text[/bold]", []string{"\033[1m", "text", "\033[0m"}},
		{"[green]success[/green]", []string{"\033[32m", "success", "\033[0m"}},
		{"[red]error[/red]", []string{"\033[31m", "error", "\033[0m"}},
	}

	for _, tt := range tests {
		result := renderMarkup(tt.input)
		for _, substr := range tt.contains {
			if !strings.Contains(result, substr) {
				t.Errorf("renderMarkup(%q) = %q, expected to contain %q", tt.input, result, substr)
			}
		}
	}
}
