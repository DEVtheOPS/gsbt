// internal/progress/progress_test.go
package progress

import (
	"bytes"
	"strings"
	"testing"

	"github.com/devtheops/gsbt/internal/log"
)

func TestNullProgress(t *testing.T) {
	var out bytes.Buffer
	logger := log.NewWithWriters(&out, &out)
	logger.SetQuiet(true)

	p := New(logger, "text")

	// All operations should be no-ops
	p.Start(1000, 5)
	p.FileStart("test.txt", 100)
	p.FileProgress("test.txt", 50, 100)
	p.FileDone("test.txt")
	p.Message("test")
	p.Close()

	if out.String() != "" {
		t.Errorf("expected no output in quiet mode, got: %s", out.String())
	}
}

func TestSimpleProgress(t *testing.T) {
	var out bytes.Buffer
	logger := log.NewWithWriters(&out, &out)
	logger.SetOutputFormat("text")

	p := New(logger, "text")

	p.Start(1000, 5)
	if !strings.Contains(out.String(), "Files: 5") {
		t.Errorf("expected file count in output, got: %s", out.String())
	}

	out.Reset()
	p.FileStart("test.txt", 100)
	if !strings.Contains(out.String(), "test.txt") {
		t.Errorf("expected filename in output, got: %s", out.String())
	}

	p.FileDone("test.txt")
	p.Close()
}

func TestSimpleProgressMessage(t *testing.T) {
	var out bytes.Buffer
	logger := log.NewWithWriters(&out, &out)
	logger.SetOutputFormat("text")

	p := New(logger, "text")
	p.Message("custom message")

	if !strings.Contains(out.String(), "custom message") {
		t.Errorf("expected custom message in output, got: %s", out.String())
	}
}

func TestNewReturnsCorrectType(t *testing.T) {
	var out bytes.Buffer
	logger := log.NewWithWriters(&out, &out)

	// Quiet mode should return nullProgress
	logger.SetQuiet(true)
	p := New(logger, "text")
	if _, ok := p.(*nullProgress); !ok {
		t.Errorf("expected nullProgress in quiet mode, got %T", p)
	}

	// JSON mode should return nullProgress
	logger.SetQuiet(false)
	p = New(logger, "json")
	if _, ok := p.(*nullProgress); !ok {
		t.Errorf("expected nullProgress in json mode, got %T", p)
	}

	// Text mode should return simpleProgress
	p = New(logger, "text")
	if _, ok := p.(*simpleProgress); !ok {
		t.Errorf("expected simpleProgress in text mode, got %T", p)
	}
}