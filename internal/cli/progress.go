// internal/cli/progress.go
package cli

import (
	"fmt"
	"io"
	"os"
	"strings"
	"time"

	"github.com/hedzr/progressbar"
	"github.com/spf13/cobra"
)

// simpleProgress writes plain log-style output; richProgress draws a live line.
type progressReporter interface {
	Message(msg string)
	Start(totalBytes int64, fileCount int)
	FileStart(name string, size int64)
	FileProgress(name string, written int64, size int64)
	FileDone(name string)
	Close()
}

func newProgressReporter(cmd *cobra.Command) progressReporter {
	format := GetOutputFormat()
	if format == "json" || IsQuiet() {
		return nil
	}
	if format == "rich" && isTTY(cmd.OutOrStdout()) {
		return newRichProgress(cmd.OutOrStdout())
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
func (p *simpleProgress) Close()               {}

type richProgress struct {
	w        io.Writer
	mpb      progressbar.MultiPB
	updates  chan int64
	last     int64
	barIdx   int
	lastTick time.Time
}

func newRichProgress(w io.Writer) *richProgress {
	mpb := progressbar.New(progressbar.WithOutputDevice(w))
	return &richProgress{w: w, mpb: mpb}
}

func (r *richProgress) Message(msg string) { fmt.Fprintf(r.w, "\r%s\n", msg) }

func (r *richProgress) Start(totalBytes int64, fileCount int) {
	fmt.Fprintf(r.w, "\rStarting backup: %d files, %.1f MB\n", fileCount, float64(totalBytes)/1e6)
}

func (r *richProgress) FileStart(name string, size int64) {
	r.last = 0
	r.lastTick = time.Time{}
	desc := truncate(name, 30)
	r.updates = make(chan int64, 32)
	r.barIdx = r.mpb.Add(size, desc,
		progressbar.WithBarWidth(30),
		progressbar.WithBarStepper(0),
		progressbar.WithBarOnCompleted(func(pb progressbar.PB) {
			fmt.Fprintln(r.w)
		}),
		progressbar.WithBarWorker(func(pb progressbar.PB, exit <-chan struct{}) (stop bool) {
			for {
				select {
				case d, ok := <-r.updates:
					if !ok {
						return true
					}
					pb.Step(d)
				case <-exit:
					return true
				}
			}
		}),
	)
}

func (r *richProgress) FileProgress(name string, written int64, size int64) {
	if r.updates == nil {
		return
	}
	// throttle rendering to ~20fps
	if time.Since(r.lastTick) < 50*time.Millisecond {
		return
	}
	r.lastTick = time.Now()
	delta := written - r.last
	if delta > 0 {
		r.updates <- delta
		r.last = written
	}
}

func (r *richProgress) FileDone(name string) {
	if r.updates != nil {
		close(r.updates)
		r.updates = nil
	}
}

func (r *richProgress) Close() {
	if r.mpb != nil {
		r.mpb.Close()
	}
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
