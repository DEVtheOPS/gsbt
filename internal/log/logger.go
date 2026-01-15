// internal/log/logger.go
package log

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"regexp"
	"strings"
	"sync"
	"time"
)

// Level represents the log level
type Level int

const (
	DebugLevel Level = iota
	InfoLevel
	WarnLevel
	ErrorLevel
)

func (l Level) String() string {
	switch l {
	case DebugLevel:
		return "debug"
	case InfoLevel:
		return "info"
	case WarnLevel:
		return "warn"
	case ErrorLevel:
		return "error"
	default:
		return "unknown"
	}
}

// Meta represents structured metadata for log entries
type Meta map[string]interface{}

// Logger provides Rich-like structured logging with markup support
type Logger struct {
	out          io.Writer
	err          io.Writer
	level        Level
	prefix       string
	outputFormat string // "text", "json", "rich"
	quiet        bool
	verbose      bool
	mu           sync.Mutex
}

// New creates a new Logger with default stdout/stderr
func New() *Logger {
	return &Logger{
		out:          os.Stdout,
		err:          os.Stderr,
		level:        InfoLevel,
		outputFormat: "text",
	}
}

// NewWithWriters creates a Logger with custom writers (useful for testing)
func NewWithWriters(out, err io.Writer) *Logger {
	return &Logger{
		out:          out,
		err:          err,
		level:        InfoLevel,
		outputFormat: "text",
	}
}

// SetOutput sets the output writer for non-error logs
func (l *Logger) SetOutput(w io.Writer) {
	l.mu.Lock()
	defer l.mu.Unlock()
	l.out = w
}

// SetErrorOutput sets the output writer for error logs
func (l *Logger) SetErrorOutput(w io.Writer) {
	l.mu.Lock()
	defer l.mu.Unlock()
	l.err = w
}

// SetOutputFormat sets the output format (text, json, rich)
func (l *Logger) SetOutputFormat(format string) {
	l.mu.Lock()
	defer l.mu.Unlock()
	l.outputFormat = format
}

// SetQuiet enables quiet mode (only errors)
func (l *Logger) SetQuiet(quiet bool) {
	l.mu.Lock()
	defer l.mu.Unlock()
	l.quiet = quiet
	if quiet {
		l.level = ErrorLevel
	}
}

// SetVerbose enables verbose mode (includes debug + metadata)
func (l *Logger) SetVerbose(verbose bool) {
	l.mu.Lock()
	defer l.mu.Unlock()
	l.verbose = verbose
	if verbose && l.level > DebugLevel {
		l.level = DebugLevel
	}
}

// SetLevel sets the minimum log level
func (l *Logger) SetLevel(level Level) {
	l.mu.Lock()
	defer l.mu.Unlock()
	l.level = level
}

// WithPrefix returns a new logger with the given prefix (e.g., "[server-name]")
func (l *Logger) WithPrefix(prefix string) *Logger {
	return &Logger{
		out:          l.out,
		err:          l.err,
		level:        l.level,
		prefix:       prefix,
		outputFormat: l.outputFormat,
		quiet:        l.quiet,
		verbose:      l.verbose,
	}
}

// Debug logs debug-level messages (only shown in verbose mode)
func (l *Logger) Debug(msg string, meta ...Meta) {
	l.log(DebugLevel, msg, meta...)
}

// Info logs informational messages
func (l *Logger) Info(msg string, meta ...Meta) {
	l.log(InfoLevel, msg, meta...)
}

// Warn logs warning messages
func (l *Logger) Warn(msg string, meta ...Meta) {
	l.log(WarnLevel, msg, meta...)
}

// Error logs error messages (always shown unless completely silenced)
func (l *Logger) Error(msg string, meta ...Meta) {
	l.log(ErrorLevel, msg, meta...)
}

func (l *Logger) log(level Level, msg string, meta ...Meta) {
	l.mu.Lock()
	defer l.mu.Unlock()

	if level < l.level {
		return
	}

	// Merge metadata if multiple provided
	var metadata Meta
	if len(meta) > 0 {
		metadata = make(Meta)
		for _, m := range meta {
			for k, v := range m {
				metadata[k] = v
			}
		}
	}

	var w io.Writer
	if level >= ErrorLevel {
		w = l.err
	} else {
		w = l.out
	}

	switch l.outputFormat {
	case "json":
		l.writeJSON(w, level, msg, metadata)
	case "rich":
		l.writeRich(w, level, msg, metadata)
	default: // "text"
		l.writeText(w, level, msg, metadata)
	}
}

func (l *Logger) writeJSON(w io.Writer, level Level, msg string, meta Meta) {
	entry := map[string]interface{}{
		"timestamp": time.Now().Format(time.RFC3339),
		"level":     level.String(),
		"message":   stripMarkup(msg),
	}

	if l.prefix != "" {
		entry["prefix"] = stripMarkup(l.prefix)
	}

	if meta != nil && len(meta) > 0 {
		entry["metadata"] = meta
	}

	data, _ := json.Marshal(entry)
	fmt.Fprintln(w, string(data))
}

func (l *Logger) writeRich(w io.Writer, level Level, msg string, meta Meta) {
	// Render ANSI color codes from markup
	rendered := renderMarkup(msg)

	if l.prefix != "" {
		fmt.Fprintf(w, "%s %s\n", renderMarkup(l.prefix), rendered)
	} else {
		fmt.Fprintln(w, rendered)
	}

	// Show metadata on second line if verbose
	if l.verbose && meta != nil && len(meta) > 0 {
		metaStr := formatMetadata(meta)
		// Dim gray for metadata
		fmt.Fprintf(w, "\033[2m%s\033[0m\n", metaStr)
	}
}

func (l *Logger) writeText(w io.Writer, level Level, msg string, meta Meta) {
	// Strip markup tags for plain text
	plain := stripMarkup(msg)

	if l.prefix != "" {
		fmt.Fprintf(w, "%s %s\n", stripMarkup(l.prefix), plain)
	} else {
		fmt.Fprintln(w, plain)
	}

	// Show metadata on second line if verbose
	if l.verbose && meta != nil && len(meta) > 0 {
		metaStr := formatMetadata(meta)
		fmt.Fprintf(w, "  %s\n", metaStr)
	}
}

// IsQuiet returns true if quiet mode is enabled
func (l *Logger) IsQuiet() bool {
	l.mu.Lock()
	defer l.mu.Unlock()
	return l.quiet
}

// IsVerbose returns true if verbose mode is enabled
func (l *Logger) IsVerbose() bool {
	l.mu.Lock()
	defer l.mu.Unlock()
	return l.verbose
}

// ANSI color codes for rich mode
var colorMap = map[string]string{
	"black":   "\033[30m",
	"red":     "\033[31m",
	"green":   "\033[32m",
	"yellow":  "\033[33m",
	"blue":    "\033[34m",
	"magenta": "\033[35m",
	"cyan":    "\033[36m",
	"white":   "\033[37m",
	"bold":    "\033[1m",
	"dim":     "\033[2m",
	"italic":  "\033[3m",
	"reset":   "\033[0m",
}

var markupRegex = regexp.MustCompile(`\[(/?)([a-z]+)\]`)

// renderMarkup converts markup tags to ANSI escape codes
// [bold]text[/bold] -> \033[1mtext\033[0m
func renderMarkup(s string) string {
	return markupRegex.ReplaceAllStringFunc(s, func(match string) string {
		parts := markupRegex.FindStringSubmatch(match)
		if len(parts) < 3 {
			return match
		}

		closing := parts[1] == "/"
		tag := parts[2]

		if closing {
			return colorMap["reset"]
		}

		if code, ok := colorMap[tag]; ok {
			return code
		}

		return match
	})
}

// stripMarkup removes all markup tags
// [bold]text[/bold] -> text
func stripMarkup(s string) string {
	return markupRegex.ReplaceAllString(s, "")
}

// formatMetadata formats metadata as key=value pairs
func formatMetadata(meta Meta) string {
	if len(meta) == 0 {
		return ""
	}

	var parts []string
	for k, v := range meta {
		parts = append(parts, fmt.Sprintf("%s=%v", k, v))
	}

	return strings.Join(parts, " ")
}
