// internal/cli/progress.go
package cli

import (
	"fmt"
	"io"
	"os"
	"strings"
	"time"

	"github.com/spf13/cobra"
)

// simpleProgress writes plain log-style output; richProgress draws a live line.
type progressReporter interface {
	Message(msg string)
	Start(totalBytes int64, fileCount int)
	FileStart(name string, size int64)
	FileProgress(name string, written int64, size int64)
	FileDone(name string)
}

func newProgressReporter(cmd *cobra.Command) progressReporter {
	if backupRich && !IsQuiet() && isTTY(cmd.OutOrStdout()) {
		return &richProgress{w: cmd.OutOrStdout()}
	}
	return &simpleProgress{out: cmd.OutOrStdout(), err: cmd.ErrOrStderr()}
}

type simpleProgress struct {
	out io.Writer
	err io.Writer
}

func (p *simpleProgress) Message(msg string) { fmt.Fprintln(p.out, msg) }
func (p *simpleProgress) Start(totalBytes int64, fileCount int) {
	fmt.Fprintf(p.out, "Files: %d, Total: %.1f MB\n", fileCount, float64(totalBytes)/1e6)
}
func (p *simpleProgress) FileStart(name string, size int64) {
	fmt.Fprintf(p.out, "- %s (%.1f MB)\n", name, float64(size)/1e6)
}
func (p *simpleProgress) FileProgress(name string, written int64, size int64) {
	// keep quiet to avoid spam; only rich mode streams progress
}
func (p *simpleProgress) FileDone(name string) { fmt.Fprintf(p.out, "  done %s\n", name) }

type richProgress struct {
	w       io.Writer
	total   int64
	current int64
	last    time.Time
}

func (r *richProgress) Message(msg string) { fmt.Fprintf(r.w, "\r%s\n", msg) }

func (r *richProgress) Start(totalBytes int64, fileCount int) {
	r.total = totalBytes
	fmt.Fprintf(r.w, "\rStarting backup: %d files, %.1f MB\n", fileCount, float64(totalBytes)/1e6)
}

func (r *richProgress) FileStart(name string, size int64) {
	r.render(name, 0, size)
}

func (r *richProgress) FileProgress(name string, written int64, size int64) {
	// throttle rendering to ~20fps
	if time.Since(r.last) < 50*time.Millisecond {
		return
	}
	r.last = time.Now()
	r.render(name, written, size)
}

func (r *richProgress) FileDone(name string) {
	r.render(name, 1, 1)
	fmt.Fprintf(r.w, "\n")
}

func (r *richProgress) render(name string, written int64, size int64) {
	percent := 0.0
	if size > 0 {
		percent = float64(written) / float64(size) * 100
	}
	bar := progressBar(percent, 30)
	fmt.Fprintf(r.w, "\r%-40s %s %5.1f%%", truncate(name, 36), bar, percent)
}

func progressBar(percent float64, width int) string {
	filled := int(percent / 100 * float64(width))
	if filled > width {
		filled = width
	}
	return fmt.Sprintf("[%s%s]", strings.Repeat("=", filled)+strings.Repeat(">", boolToInt(filled < width)), strings.Repeat(" ", width-filled))
}

func boolToInt(b bool) int {
	if b {
		return 1
	}
	return 0
}

// best-effort TTY check
func isTTY(w io.Writer) bool {
	if f, ok := w.(*os.File); ok {
		fi, _ := f.Stat()
		return (fi.Mode() & os.ModeCharDevice) != 0
	}
	return false
}

func truncate(s string, max int) string {
	if len(s) <= max {
		return s
	}
	if max <= 3 {
		return s[:max]
	}
	return s[:max-3] + "..."
}
