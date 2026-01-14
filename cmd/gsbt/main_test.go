// cmd/gsbt/main_test.go
package main

import "testing"

func TestGetMessage(t *testing.T) {
	expected := "gsbt - gameserver backup tool"
	actual := getMessage()

	if actual != expected {
		t.Errorf("getMessage() = %q, want %q", actual, expected)
	}
}
