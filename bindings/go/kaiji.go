// Package kaiji provides Go bindings for the kaiji CJK normalization engine.
//
// Before using this package, build the Rust kaiji-c library:
//
//	cargo build --release --manifest-path path/to/crates/kaiji-c/Cargo.toml
//
// Then set CGO_LDFLAGS to point to the build output:
//
//	export CGO_LDFLAGS="-L/path/to/crates/kaiji-c/target/release"
package kaiji

/*
#cgo LDFLAGS: -lkaiji_c
#include "kaiji.h"
#include <stdlib.h>
*/
import "C"
import (
	"errors"
	"unsafe"
)

// Normalize normalizes a CJK string using the default configuration.
// Variant characters are folded to canonical forms (e.g. 齋→斉) and IVS selectors are stripped.
func Normalize(input string) (string, error) {
	cInput := C.CString(input)
	defer C.free(unsafe.Pointer(cInput))

	cResult := C.kaiji_normalize(cInput)
	if cResult == nil {
		return "", errors.New("kaiji: normalization failed")
	}
	defer C.kaiji_free_string(cResult)
	return C.GoString(cResult), nil
}

// Matches returns true if a and b are equivalent after CJK normalization.
func Matches(a, b string) (bool, error) {
	cA := C.CString(a)
	defer C.free(unsafe.Pointer(cA))
	cB := C.CString(b)
	defer C.free(unsafe.Pointer(cB))

	result := C.kaiji_matches(cA, cB)
	switch result {
	case 1:
		return true, nil
	case 0:
		return false, nil
	default:
		return false, errors.New("kaiji: match check failed")
	}
}

// Similarity returns the Jaro-Winkler similarity score between a and b
// after CJK normalization, in the range [0.0, 1.0].
func Similarity(a, b string) (float32, error) {
	cA := C.CString(a)
	defer C.free(unsafe.Pointer(cA))
	cB := C.CString(b)
	defer C.free(unsafe.Pointer(cB))

	score := float32(C.kaiji_similarity(cA, cB))
	if score < 0 {
		return 0, errors.New("kaiji: similarity computation failed")
	}
	return score, nil
}
