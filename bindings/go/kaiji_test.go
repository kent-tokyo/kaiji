package kaiji_test

import (
	"testing"

	kaiji "github.com/kent-tokyo/kaiji"
)

func TestNormalize(t *testing.T) {
	result, err := kaiji.Normalize("齋藤")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result != "斉藤" {
		t.Errorf("expected 斉藤, got %s", result)
	}
}

func TestMatches(t *testing.T) {
	matched, err := kaiji.Matches("斎藤", "齋藤")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !matched {
		t.Error("expected true, got false")
	}
}

func TestMatchesFalse(t *testing.T) {
	matched, err := kaiji.Matches("斎藤", "佐藤")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if matched {
		t.Error("expected false, got true")
	}
}

func TestSimilarity(t *testing.T) {
	score, err := kaiji.Similarity("斎藤", "齋藤")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if score != 1.0 {
		t.Errorf("expected 1.0, got %f", score)
	}
}
