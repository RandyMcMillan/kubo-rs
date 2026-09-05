package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"encoding/json"
	"fmt"

	"github.com/nbd-wtf/go-nostr"
)

//export kubo_nostr_generate_key
func kubo_nostr_generate_key() *C.char {
	key := nostr.GeneratePrivateKey()
	if key == "" {
		setError(fmt.Errorf("failed to generate key"))
		return nil
	}
	setError(nil)
	return C.CString(key)
}

//export kubo_nostr_get_public_key
func kubo_nostr_get_public_key(sk *C.char) *C.char {
	pk, err := nostr.GetPublicKey(C.GoString(sk))
	if err != nil {
		setError(fmt.Errorf("get public key: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(pk)
}

//export kubo_nostr_event_sign
func kubo_nostr_event_sign(sk *C.char, content *C.char, kind C.int) *C.char {
	evt := nostr.Event{
		CreatedAt: nostr.Now(),
		Kind:      int(kind),
		Content:   C.GoString(content),
		Tags:      make(nostr.Tags, 0),
	}

	if err := evt.Sign(C.GoString(sk)); err != nil {
		setError(fmt.Errorf("sign event: %w", err))
		return nil
	}

	jsonBytes, err := json.Marshal(evt)
	if err != nil {
		setError(fmt.Errorf("marshal event: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(string(jsonBytes))
}

//export kubo_nostr_event_verify
func kubo_nostr_event_verify(jsonStr *C.char) int64 {
	var evt nostr.Event
	if err := json.Unmarshal([]byte(C.GoString(jsonStr)), &evt); err != nil {
		setError(fmt.Errorf("unmarshal event: %w", err))
		return -1
	}

	if ok, err := evt.CheckSignature(); err != nil || !ok {
		setError(fmt.Errorf("invalid signature"))
		return 0
	}

	setError(nil)
	return 1
}
