// internal/version/version_test.go
package version

import "testing"

func TestVersionVariables(t *testing.T) {
	tests := []struct {
		name     string
		variable *string
		want     string
	}{
		{
			name:     "Version default",
			variable: &Version,
			want:     "dev",
		},
		{
			name:     "Commit default",
			variable: &Commit,
			want:     "none",
		},
		{
			name:     "BuildDate default",
			variable: &BuildDate,
			want:     "unknown",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if *tt.variable != tt.want {
				t.Errorf("variable = %q, want %q", *tt.variable, tt.want)
			}
		})
	}
}
