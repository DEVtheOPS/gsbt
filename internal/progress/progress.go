// internal/progress/progress.go
package progress

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"time"

	"github.com/devtheops/gsbt/internal/log"
	"github.com/hedzr/progressbar"
)

// Reporter tracks and reports progress for long-running operations
type Reporter interface {
	// Start initializes progress tracking with total bytes and file count
	Start(totalBytes int64, fileCount int)

	// FileStart marks the beginning of a file download/upload
	FileStart(name string, size int64)

	// FileProgress updates progress for the current file
	FileProgress(name string, written int64, size int64)

	// FileDone marks completion of the current file
	FileDone(name string)

	// Message logs an informational message
	Message(msg string)

	// Close cleans up resources
	Close()
}

// New creates a progress reporter based on output format
func New(logger *log.Logger, format string) Reporter {
	if logger.IsQuiet() || format == "json" {
		return &nullProgress{}
	}

	if format == "rich" && isTTY(os.Stdout) {
		return newRichProgress(logger)
	}

	return &simpleProgress{logger: logger}
}

// nullProgress is a no-op reporter for quiet/json modes
type nullProgress struct{}

func (n *nullProgress) Start(totalBytes int64, fileCount int)             {}
func (n *nullProgress) FileStart(name string, size int64)                 {}
func (n *nullProgress) FileProgress(name string, written int64, size int64) {}
func (n *nullProgress) FileDone(name string)                              {}
func (n *nullProgress) Message(msg string)                                {}
func (n *nullProgress) Close()                                            {}

// simpleProgress outputs text-based progress updates
type simpleProgress struct {
	logger *log.Logger
}

func (s *simpleProgress) Start(totalBytes int64, fileCount int) {
	s.logger.Info(fmt.Sprintf("Files: %d, Total: %.1f MB", fileCount, float64(totalBytes)/1e6))
}

func (s *simpleProgress) FileStart(name string, size int64) {
	s.logger.Info(fmt.Sprintf("- %s (%.1f MB)", name, float64(size)/1e6))
}

func (s *simpleProgress) FileProgress(name string, written int64, size int64) {
	// Don't spam output; only show start/done
}

func (s *simpleProgress) FileDone(name string) {
	s.logger.Debug(fmt.Sprintf("  done %s", name))
}

func (s *simpleProgress) Message(msg string) {
	s.logger.Info(msg)
}

func (s *simpleProgress) Close() {}

// richProgress renders an animated progress bar using hedzr/progressbar
type richProgress struct {
	logger      *log.Logger
	mpb         progressbar.MultiPB
	updates     chan int64
	total       int64
	current     int64
	fileWritten int64
	barIdx      int
	lastTick    time.Time
	startTime   time.Time
}

func newRichProgress(logger *log.Logger) *richProgress {
	mpb := progressbar.New(progressbar.WithOutputDevice(os.Stdout))
	return &richProgress{
		logger: logger,
		mpb:    mpb,
	}
}

func (r *richProgress) Start(totalBytes int64, fileCount int) {
	if totalBytes <= 0 {
		totalBytes = 1
	}
	r.total = totalBytes
	r.startTime = time.Now()
	r.updates = make(chan int64, 128)

	r.barIdx = r.mpb.Add(totalBytes, "backup",
		progressbar.WithBarWidth(30),
		progressbar.WithBarStepper(0),
		progressbar.WithBarOnCompleted(func(pb progressbar.PB) {
			fmt.Println() // newline after completion
		}),
		progressbar.WithBarWorker(func(pb progressbar.PB, exit <-chan struct{}) (stop bool) {
			for {
				select {
				case d, ok := <-r.updates:
					if !ok {
						return true
					}
					if d > 0 {
						pb.Step(d)
					}
				case <-exit:
					return true
				}
			}
		}),
	)

	r.logger.Info(fmt.Sprintf("Starting backup: %d files, %.1f MB", fileCount, float64(totalBytes)/1e6))
}

func (r *richProgress) FileStart(name string, size int64) {
	r.fileWritten = 0
	r.lastTick = time.Time{}

	if bar := r.mpb.Bar(r.barIdx); bar != nil {
		bar.SetAppendText(" " + truncate(name, 30))
	}
}

func (r *richProgress) FileProgress(name string, written int64, size int64) {
	if r.updates == nil {
		return
	}

	delta := written - r.fileWritten
	if delta <= 0 {
		return
	}

	// Throttle rendering to ~20fps to avoid spam
	if time.Since(r.lastTick) < 50*time.Millisecond {
		return
	}

	r.lastTick = time.Now()
	r.fileWritten += delta
	r.current += delta
	r.updates <- delta
}

func (r *richProgress) FileDone(name string) {
	// Progress bar shows overall completion
}

func (r *richProgress) Message(msg string) {
	fmt.Fprintf(os.Stdout, "\r%s\n", msg)
}

func (r *richProgress) Close() {
	if r.mpb != nil {
		if r.updates != nil {
			close(r.updates)
			r.updates = nil
		}
		r.mpb.Close()
	}
}

// jsonProgress outputs structured JSON progress events
type jsonProgress struct {
	out       io.Writer
	startTime time.Time
}

func (j *jsonProgress) Start(totalBytes int64, fileCount int) {
	j.startTime = time.Now()
	j.emit("start", map[string]interface{}{
		"total_bytes": totalBytes,
		"file_count":  fileCount,
	})
}

func (j *jsonProgress) FileStart(name string, size int64) {
	j.emit("file_start", map[string]interface{}{
		"file": name,
		"size": size,
	})
}

func (j *jsonProgress) FileProgress(name string, written int64, size int64) {
	j.emit("file_progress", map[string]interface{}{
		"file":    name,
		"written": written,
		"size":    size,
		"percent": float64(written) / float64(size) * 100,
	})
}

func (j *jsonProgress) FileDone(name string) {
	j.emit("file_done", map[string]interface{}{
		"file": name,
	})
}

func (j *jsonProgress) Message(msg string) {
	j.emit("message", map[string]interface{}{
		"text": msg,
	})
}

func (j *jsonProgress) Close() {
	j.emit("complete", map[string]interface{}{
		"duration_ms": time.Since(j.startTime).Milliseconds(),
	})
}

func (j *jsonProgress) emit(event string, data map[string]interface{}) {
	entry := map[string]interface{}{
		"event":     event,
		"timestamp": time.Now().Format(time.RFC3339),
		"data":      data,
	}
	raw, _ := json.Marshal(entry)
	fmt.Fprintln(j.out, string(raw))
}

// isTTY checks if stdout is a terminal (for rich mode detection)
func isTTY(f *os.File) bool {
	fi, err := f.Stat()
	if err != nil {
		return false
	}
	return (fi.Mode() & os.ModeCharDevice) != 0
}

// truncate shortens strings for display
func truncate(s string, max int) string {
	if len(s) <= max {
		return s
	}
	if max <= 3 {
		return s[:max]
	}
	return s[:max-3] + "..."
}
