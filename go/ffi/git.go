package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"fmt"
	"sync"

	"github.com/go-git/go-git/v5"
)

var (
	gitReposMu    sync.RWMutex
	gitRepos      = make(map[uint64]*git.Repository)
	gitNextHandle uint64 = 1
)

//export git_clone
func git_clone(url *C.char, path *C.char, bare C.uint8_t) int64 {
	_, err := git.PlainClone(C.GoString(path), bare != 0, &git.CloneOptions{
		URL:      C.GoString(url),
		Progress: nil,
	})
	if err != nil {
		setError(fmt.Errorf("git clone: %w", err))
		return -1
	}
	setError(nil)
	return 0
}

//export git_open
func git_open(path *C.char) uint64 {
	repo, err := git.PlainOpen(C.GoString(path))
	if err != nil {
		setError(fmt.Errorf("git open: %w", err))
		return 0
	}

	gitReposMu.Lock()
	handle := gitNextHandle
	gitNextHandle++
	gitRepos[handle] = repo
	gitReposMu.Unlock()

	setError(nil)
	return handle
}

//export git_repo_head
func git_repo_head(handle uint64) *C.char {
	gitReposMu.RLock()
	repo, ok := gitRepos[handle]
	gitReposMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return nil
	}

	ref, err := repo.Head()
	if err != nil {
		setError(fmt.Errorf("git head: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(ref.Hash().String())
}

//export git_repo_free
func git_repo_free(handle uint64) int64 {
	gitReposMu.Lock()
	_, ok := gitRepos[handle]
	if ok {
		delete(gitRepos, handle)
	}
	gitReposMu.Unlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return -1
	}

	setError(nil)
	return 0
}

//export git_init
func git_init(path *C.char, bare C.uint8_t) int64 {
	_, err := git.PlainInit(C.GoString(path), bare != 0)
	if err != nil {
		setError(fmt.Errorf("git init: %w", err))
		return -1
	}
	setError(nil)
	return 0
}
