// Package main exports C symbols for Rust via CGO.
package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"sync"
	"unsafe"
)

var (
	lastErrorMu sync.Mutex
	lastError   string
)

func setError(err error) {
	lastErrorMu.Lock()
	defer lastErrorMu.Unlock()
	if err != nil {
		lastError = err.Error()
	} else {
		lastError = ""
	}
}

//export ffi_last_error
func ffi_last_error() *C.char {
	lastErrorMu.Lock()
	defer lastErrorMu.Unlock()
	if lastError == "" {
		return nil
	}
	return C.CString(lastError)
}

//export ffi_free_string
func ffi_free_string(s *C.char) {
	C.free(unsafe.Pointer(s))
}

//export ffi_free_buffer
func ffi_free_buffer(buf *C.uint8_t) {
	C.free(unsafe.Pointer(buf))
}
