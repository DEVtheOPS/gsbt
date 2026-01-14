// internal/connector/matcher_test.go
package connector

import "testing"

func TestMatchesPatterns(t *testing.T) {
	tests := []struct {
		path     string
		include  []string
		exclude  []string
		expected bool
	}{
		// Include all, no exclude
		{"file.txt", []string{"*"}, nil, true},

		// Include all, exclude logs
		{"file.txt", []string{"*"}, []string{"*.log"}, true},
		{"debug.log", []string{"*"}, []string{"*.log"}, false},

		// Exclude directories
		{"Logs/debug.log", []string{"*"}, []string{"Logs/"}, false},
		{"saves/game.sav", []string{"*"}, []string{"Logs/"}, true},

		// Multiple excludes
		{"temp.tmp", []string{"*"}, []string{"*.log", "*.tmp"}, false},

		// Specific includes
		{"game.sav", []string{"*.sav"}, nil, true},
		{"config.ini", []string{"*.sav"}, nil, false},
	}

	for _, tt := range tests {
		result := MatchesPatterns(tt.path, tt.include, tt.exclude)
		if result != tt.expected {
			t.Errorf("MatchesPatterns(%q, %v, %v) = %v, want %v",
				tt.path, tt.include, tt.exclude, result, tt.expected)
		}
	}
}
