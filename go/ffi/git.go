package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"fmt"
	"strings"
	"sync"

	"github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing"
)

var (
	gitReposMu    sync.RWMutex
	gitRepos      = make(map[uint64]*git.Repository)
	gitNextHandle uint64 = 1
)

//export kubo_git_clone
func kubo_git_clone(url *C.char, path *C.char, bare C.uint8_t) int64 {
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

//export kubo_git_open
func kubo_git_open(path *C.char) uint64 {
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

//export kubo_git_repo_head
func kubo_git_repo_head(handle uint64) *C.char {
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

//export kubo_git_repo_free
func kubo_git_repo_free(handle uint64) int64 {
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

//export kubo_git_init
func kubo_git_init(path *C.char, bare C.uint8_t) int64 {
	_, err := git.PlainInit(C.GoString(path), bare != 0)
	if err != nil {
		setError(fmt.Errorf("git init: %w", err))
		return -1
	}
	setError(nil)
	return 0
}

//export kubo_git_repo_is_bare
func kubo_git_repo_is_bare(handle uint64) int64 {
	gitReposMu.RLock()
	repo, ok := gitRepos[handle]
	gitReposMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return -1
	}

	cfg, err := repo.Config()
	if err != nil {
		setError(fmt.Errorf("git config: %w", err))
		return -1
	}

	if cfg.Core.IsBare {
		return 1
	}
	return 0
}

//export kubo_git_repo_branches
func kubo_git_repo_branches(handle uint64) *C.char {
	gitReposMu.RLock()
	repo, ok := gitRepos[handle]
	gitReposMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return nil
	}

	iter, err := repo.Branches()
	if err != nil {
		setError(fmt.Errorf("git branches: %w", err))
		return nil
	}
	defer iter.Close()

	var names []string
	for {
		ref, err := iter.Next()
		if err != nil {
			break
		}
		names = append(names, ref.Name().Short())
	}

	setError(nil)
	return C.CString(strings.Join(names, "\n"))
}

//export kubo_git_repo_remotes
func kubo_git_repo_remotes(handle uint64) *C.char {
	gitReposMu.RLock()
	repo, ok := gitRepos[handle]
	gitReposMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return nil
	}

	remotes, err := repo.Remotes()
	if err != nil {
		setError(fmt.Errorf("git remotes: %w", err))
		return nil
	}

	var names []string
	for _, r := range remotes {
		names = append(names, r.Config().Name)
	}

	setError(nil)
	return C.CString(strings.Join(names, "\n"))
}

//export kubo_git_repo_create_branch
func kubo_git_repo_create_branch(handle uint64, name *C.char, commit_hash *C.char) int64 {
	gitReposMu.RLock()
	repo, ok := gitRepos[handle]
	gitReposMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return -1
	}

	hash := plumbing.NewHash(C.GoString(commit_hash))
	ref := plumbing.NewHashReference(plumbing.NewBranchReferenceName(C.GoString(name)), hash)
	if err := repo.Storer.SetReference(ref); err != nil {
		setError(fmt.Errorf("git create branch: %w", err))
		return -1
	}

	setError(nil)
	return 0
}
