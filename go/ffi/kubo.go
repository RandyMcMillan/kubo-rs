package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"bytes"
	"context"
	"encoding/base64"
	"fmt"
	"io"
	"net"
	"net/http"
	gopath "path"
	"strings"
	"sync"
	"time"
	"unsafe"

	"github.com/ipfs/boxo/files"
	"github.com/ipfs/boxo/mfs"
	"github.com/ipfs/boxo/path"
	blocks "github.com/ipfs/go-block-format"
	"github.com/ipfs/go-cid"
	ipldlegacy "github.com/ipfs/go-ipld-legacy"
	ipfs "github.com/ipfs/kubo"
	"github.com/ipfs/kubo/commands"
	"github.com/ipfs/kubo/config"
	"github.com/ipfs/kubo/core"
	"github.com/ipfs/kubo/core/coreapi"
	"github.com/ipfs/kubo/core/corehttp"
	coreiface "github.com/ipfs/kubo/core/coreiface"
	"github.com/ipfs/kubo/core/coreiface/options"
	"github.com/ipfs/kubo/core/node/libp2p"
	"github.com/ipfs/kubo/plugin/loader"
	"github.com/ipfs/kubo/repo/fsrepo"
	"github.com/ipld/go-ipld-prime/multicodec"
	basicnode "github.com/ipld/go-ipld-prime/node/basic"
	"github.com/libp2p/go-libp2p/core/peer"
	mc "github.com/multiformats/go-multicodec"
	ma "github.com/multiformats/go-multiaddr"
	manet "github.com/multiformats/go-multiaddr/net"

	// DAG codecs
	_ "github.com/ipld/go-codec-dagpb"
	_ "github.com/ipld/go-ipld-prime/codec/cbor"
	_ "github.com/ipld/go-ipld-prime/codec/dagcbor"
	_ "github.com/ipld/go-ipld-prime/codec/dagjson"
	_ "github.com/ipld/go-ipld-prime/codec/json"
	_ "github.com/ipld/go-ipld-prime/codec/raw"
)

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

//export kubo_version
func kubo_version() *C.char {
	return C.CString(ipfs.CurrentVersionNumber)
}

// ---------------------------------------------------------------------------
// Plugin loading (once)
// ---------------------------------------------------------------------------

var (
	pluginsOnce sync.Once
	pluginsErr  error
)

func ensurePlugins() error {
	pluginsOnce.Do(func() {
		plugins, err := loader.NewPluginLoader("")
		if err != nil {
			pluginsErr = fmt.Errorf("error loading plugins: %w", err)
			return
		}
		if err := plugins.Initialize(); err != nil {
			pluginsErr = fmt.Errorf("error initializing plugins: %w", err)
			return
		}
		if err := plugins.Inject(); err != nil {
			pluginsErr = fmt.Errorf("error injecting plugins: %w", err)
			return
		}
	})
	return pluginsErr
}

// ---------------------------------------------------------------------------
// Repo initialization
// ---------------------------------------------------------------------------

//export kubo_init_repo
func kubo_init_repo(repoPath *C.char) int64 {
	if err := ensurePlugins(); err != nil {
		setError(err)
		return -1
	}

	path := C.GoString(repoPath)

	identity, err := config.CreateIdentity(io.Discard, []options.KeyGenerateOption{
		options.Key.Type(options.Ed25519Key),
	})
	if err != nil {
		setError(fmt.Errorf("create identity: %w", err))
		return -1
	}

	cfg, err := config.InitWithIdentity(identity)
	if err != nil {
		setError(fmt.Errorf("init config: %w", err))
		return -1
	}

	cfg.Addresses.Swarm = []string{
		"/ip4/127.0.0.1/tcp/0",
	}
	cfg.Swarm.Transports.Network.QUIC = config.False
	cfg.Swarm.Transports.Network.Relay = config.False
	cfg.Swarm.Transports.Network.WebTransport = config.False
	cfg.Swarm.Transports.Network.WebRTCDirect = config.False
	cfg.Swarm.Transports.Network.Websocket = config.False
	cfg.AutoTLS.Enabled = config.False
	cfg.Routing.Type = config.NewOptionalString("none")
	cfg.Bootstrap = []string{}
	cfg.Discovery.MDNS.Enabled = false

	if err := fsrepo.Init(path, cfg); err != nil {
		setError(fmt.Errorf("init repo: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

// ---------------------------------------------------------------------------
// Node registry
// ---------------------------------------------------------------------------

type nodeHandle struct {
	ctx            context.Context
	cancel         context.CancelFunc
	node           *core.IpfsNode
	api            coreiface.CoreAPI
	apiListener    manet.Listener
	apiNetListener net.Listener
	apiServer      *http.Server
}

var (
	nodesMu    sync.RWMutex
	nodes      = make(map[uint64]*nodeHandle)
	nextHandle uint64 = 1
)

//export kubo_node_start
func kubo_node_start(repoPath *C.char, online C.uint8_t) uint64 {
	if err := ensurePlugins(); err != nil {
		setError(err)
		return 0
	}

	path := C.GoString(repoPath)

	repo, err := fsrepo.Open(path)
	if err != nil {
		setError(fmt.Errorf("open repo: %w", err))
		return 0
	}

	ctx, cancel := context.WithCancel(context.Background())

	cfg := &core.BuildCfg{
		Online: online != 0,
		Repo:   repo,
	}
	if !cfg.Online {
		cfg.Routing = libp2p.NilRouterOption
	}

	n, err := core.NewNode(ctx, cfg)
	if err != nil {
		cancel()
		setError(fmt.Errorf("new node: %w", err))
		return 0
	}

	api, err := coreapi.NewCoreAPI(n)
	if err != nil {
		n.Close()
		cancel()
		setError(fmt.Errorf("core api: %w", err))
		return 0
	}

	h := &nodeHandle{
		ctx:    ctx,
		cancel: cancel,
		node:   n,
		api:    api,
	}

	nodesMu.Lock()
	handle := nextHandle
	nextHandle++
	nodes[handle] = h
	nodesMu.Unlock()

	setError(nil)
	return handle
}

//export kubo_node_stop
func kubo_node_stop(handle uint64) int64 {
	nodesMu.Lock()
	h, ok := nodes[handle]
	if ok {
		delete(nodes, handle)
	}
	nodesMu.Unlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	if h.apiListener != nil {
		h.apiListener.Close()
	}
	h.cancel()
	if err := h.node.Close(); err != nil {
		setError(fmt.Errorf("close node: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_node_start_api
func kubo_node_start_api(handle uint64, multiaddr *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	addrStr := C.GoString(multiaddr)
	addr, err := ma.NewMultiaddr(addrStr)
	if err != nil {
		setError(fmt.Errorf("parse multiaddr: %w", err))
		return nil
	}

	listener, err := manet.Listen(addr)
	if err != nil {
		setError(fmt.Errorf("listen: %w", err))
		return nil
	}

	cctx := commands.Context{
		ConstructNode: func() (*core.IpfsNode, error) {
			return h.node, nil
		},
		ReqLog: &commands.ReqLog{},
	}

	opts := []corehttp.ServeOption{
		corehttp.CheckVersionOption(),
		corehttp.CommandsOption(cctx),
		corehttp.WebUIOption,
		corehttp.VersionOption(),
	}

	h.apiListener = listener
	h.apiNetListener = manet.NetListener(listener)
	h.apiServer = &http.Server{}

	go func() {
		handler, err := corehttp.MakeHandler(h.node, h.apiNetListener, opts...)
		if err != nil {
			setError(fmt.Errorf("make handler: %w", err))
			return
		}
		h.apiServer.Handler = handler
		h.apiServer.Serve(h.apiNetListener)
	}()

	setError(nil)
	return C.CString(listener.Multiaddr().String())
}

//export kubo_node_api_addrs
func kubo_node_api_addrs(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	if h.apiListener == nil {
		setError(fmt.Errorf("api not started"))
		return nil
	}

	setError(nil)
	return C.CString(h.apiListener.Multiaddr().String())
}

//export kubo_node_peer_id
func kubo_node_peer_id(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	return C.CString(h.node.Identity.String())
}

//export kubo_node_listening_addrs
func kubo_node_listening_addrs(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	addrs, err := h.api.Swarm().LocalAddrs(h.ctx)
	if err != nil {
		setError(fmt.Errorf("local addrs: %w", err))
		return nil
	}

	var parts []string
	for _, a := range addrs {
		parts = append(parts, a.String())
	}

	setError(nil)
	return C.CString(strings.Join(parts, "\n"))
}

//export kubo_node_connect
func kubo_node_connect(handle uint64, addr *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	addrStr := C.GoString(addr)
	maddr, err := ma.NewMultiaddr(addrStr)
	if err != nil {
		setError(fmt.Errorf("parse multiaddr: %w", err))
		return -1
	}

	info, err := peer.AddrInfoFromP2pAddr(maddr)
	if err != nil {
		setError(fmt.Errorf("extract peer info: %w", err))
		return -1
	}

	if err := h.api.Swarm().Connect(h.ctx, *info); err != nil {
		setError(fmt.Errorf("connect: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_swarm_peers
func kubo_swarm_peers(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	peers, err := h.api.Swarm().Peers(h.ctx)
	if err != nil {
		setError(fmt.Errorf("swarm peers: %w", err))
		return nil
	}

	var parts []string
	for _, p := range peers {
		addr := ""
		if p.Address() != nil {
			addr = p.Address().String()
		}
		parts = append(parts, p.ID().String()+"\t"+addr)
	}

	setError(nil)
	return C.CString(strings.Join(parts, "\n"))
}

//export kubo_node_id
func kubo_node_id(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	id := h.node.Identity.String()
	pk, err := h.node.Identity.ExtractPublicKey()
	if err != nil {
		setError(fmt.Errorf("extract public key: %w", err))
		return nil
	}
	pkBytes, err := pk.Raw()
	if err != nil {
		setError(fmt.Errorf("raw public key: %w", err))
		return nil
	}

	info := fmt.Sprintf(
		`{"id":"%s","public_key":"%s"}`,
		id,
		base64.StdEncoding.EncodeToString(pkBytes),
	)
	setError(nil)
	return C.CString(info)
}

// ---------------------------------------------------------------------------
// UnixFS helpers
// ---------------------------------------------------------------------------

//export kubo_unixfs_add_bytes
func kubo_unixfs_add_bytes(handle uint64, data *C.uint8_t, length C.size_t) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	goData := C.GoBytes(unsafe.Pointer(data), C.int(length))
	file := files.NewBytesFile(goData)

	p, err := h.api.Unixfs().Add(h.ctx, file)
	if err != nil {
		setError(fmt.Errorf("unixfs add: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(p.RootCid().String())
}

//export kubo_unixfs_cat
func kubo_unixfs_cat(handle uint64, cidStr *C.char, out **C.uint8_t, outLen *C.size_t) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	cidStrGo := C.GoString(cidStr)
	var p path.Path
	var err error

	if strings.HasPrefix(cidStrGo, "/ipfs/") || strings.HasPrefix(cidStrGo, "/ipns/") {
		p, err = path.NewPath(cidStrGo)
	} else {
		var c cid.Cid
		c, err = cid.Decode(cidStrGo)
		if err == nil {
			p = path.FromCid(c)
		}
	}
	if err != nil {
		setError(fmt.Errorf("parse path: %w", err))
		return -1
	}

	node, err := h.api.Unixfs().Get(h.ctx, p)
	if err != nil {
		setError(fmt.Errorf("unixfs get: %w", err))
		return -1
	}
	defer node.Close()

	file, ok := node.(files.File)
	if !ok {
		setError(fmt.Errorf("node is not a file"))
		return -1
	}

	buf, err := io.ReadAll(file)
	if err != nil {
		setError(fmt.Errorf("read file: %w", err))
		return -1
	}

	if len(buf) == 0 {
		*out = nil
		*outLen = 0
		return 0
	}

	cBuf := C.malloc(C.size_t(len(buf)))
	copy((*[1 << 30]byte)(cBuf)[:len(buf):len(buf)], buf)
	*out = (*C.uint8_t)(cBuf)
	*outLen = C.size_t(len(buf))

	setError(nil)
	return 0
}

// ---------------------------------------------------------------------------
// Block API
// ---------------------------------------------------------------------------

//export kubo_block_put
func kubo_block_put(handle uint64, data *C.uint8_t, length C.size_t) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	goData := C.GoBytes(unsafe.Pointer(data), C.int(length))
	stat, err := h.api.Block().Put(h.ctx, bytes.NewReader(goData))
	if err != nil {
		setError(fmt.Errorf("block put: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(stat.Path().RootCid().String())
}

//export kubo_block_get
func kubo_block_get(handle uint64, cidStr *C.char, out **C.uint8_t, outLen *C.size_t) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	cidStrGo := C.GoString(cidStr)
	var p path.Path
	var err error

	if strings.HasPrefix(cidStrGo, "/ipfs/") || strings.HasPrefix(cidStrGo, "/ipns/") {
		p, err = path.NewPath(cidStrGo)
	} else {
		var c cid.Cid
		c, err = cid.Decode(cidStrGo)
		if err == nil {
			p = path.FromCid(c)
		}
	}
	if err != nil {
		setError(fmt.Errorf("parse path: %w", err))
		return -1
	}

	reader, err := h.api.Block().Get(h.ctx, p)
	if err != nil {
		setError(fmt.Errorf("block get: %w", err))
		return -1
	}

	buf, err := io.ReadAll(reader)
	if err != nil {
		setError(fmt.Errorf("read block: %w", err))
		return -1
	}

	if len(buf) == 0 {
		*out = nil
		*outLen = 0
		return 0
	}

	cBuf := C.malloc(C.size_t(len(buf)))
	copy((*[1 << 30]byte)(cBuf)[:len(buf):len(buf)], buf)
	*out = (*C.uint8_t)(cBuf)
	*outLen = C.size_t(len(buf))

	setError(nil)
	return 0
}

//export kubo_block_stat
func kubo_block_stat(handle uint64, cidStr *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	cidStrGo := C.GoString(cidStr)
	var p path.Path
	var err error

	if strings.HasPrefix(cidStrGo, "/ipfs/") || strings.HasPrefix(cidStrGo, "/ipns/") {
		p, err = path.NewPath(cidStrGo)
	} else {
		var c cid.Cid
		c, err = cid.Decode(cidStrGo)
		if err == nil {
			p = path.FromCid(c)
		}
	}
	if err != nil {
		setError(fmt.Errorf("parse path: %w", err))
		return -1
	}

	stat, err := h.api.Block().Stat(h.ctx, p)
	if err != nil {
		setError(fmt.Errorf("block stat: %w", err))
		return -1
	}

	setError(nil)
	return int64(stat.Size())
}

// ---------------------------------------------------------------------------
// Swarm
// ---------------------------------------------------------------------------

//export kubo_swarm_disconnect
func kubo_swarm_disconnect(handle uint64, addr *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	addrStr := C.GoString(addr)
	maddr, err := ma.NewMultiaddr(addrStr)
	if err != nil {
		setError(fmt.Errorf("parse multiaddr: %w", err))
		return -1
	}

	if err := h.api.Swarm().Disconnect(h.ctx, maddr); err != nil {
		setError(fmt.Errorf("disconnect: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

// ---------------------------------------------------------------------------
// Pin
// ---------------------------------------------------------------------------

//export kubo_pin_add
func kubo_pin_add(handle uint64, cidStr *C.char, recursive C.uint8_t) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	cidStrGo := C.GoString(cidStr)
	var p path.Path
	var err error

	if strings.HasPrefix(cidStrGo, "/ipfs/") || strings.HasPrefix(cidStrGo, "/ipns/") {
		p, err = path.NewPath(cidStrGo)
	} else {
		var c cid.Cid
		c, err = cid.Decode(cidStrGo)
		if err == nil {
			p = path.FromCid(c)
		}
	}
	if err != nil {
		setError(fmt.Errorf("parse path: %w", err))
		return -1
	}

	opts := []options.PinAddOption{options.Pin.Recursive(recursive != 0)}
	if err := h.api.Pin().Add(h.ctx, p, opts...); err != nil {
		setError(fmt.Errorf("pin add: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_pin_rm
func kubo_pin_rm(handle uint64, cidStr *C.char, recursive C.uint8_t) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	cidStrGo := C.GoString(cidStr)
	var p path.Path
	var err error

	if strings.HasPrefix(cidStrGo, "/ipfs/") || strings.HasPrefix(cidStrGo, "/ipns/") {
		p, err = path.NewPath(cidStrGo)
	} else {
		var c cid.Cid
		c, err = cid.Decode(cidStrGo)
		if err == nil {
			p = path.FromCid(c)
		}
	}
	if err != nil {
		setError(fmt.Errorf("parse path: %w", err))
		return -1
	}

	opts := []options.PinRmOption{options.Pin.RmRecursive(recursive != 0)}
	if err := h.api.Pin().Rm(h.ctx, p, opts...); err != nil {
		setError(fmt.Errorf("pin rm: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_pin_ls
func kubo_pin_ls(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	ch := make(chan coreiface.Pin)
	var pins []string
	var lsErr error

	go func() {
		lsErr = h.api.Pin().Ls(h.ctx, ch)
	}()

	for pin := range ch {
		pins = append(pins, pin.Path().String()+"\t"+pin.Type())
	}

	if lsErr != nil {
		setError(fmt.Errorf("pin ls: %w", lsErr))
		return nil
	}

	setError(nil)
	return C.CString(strings.Join(pins, "\n"))
}

// ---------------------------------------------------------------------------
// DHT / Routing
// ---------------------------------------------------------------------------

//export kubo_dht_findpeer
func kubo_dht_findpeer(handle uint64, peerIDStr *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	pid, err := peer.Decode(C.GoString(peerIDStr))
	if err != nil {
		setError(fmt.Errorf("decode peer id: %w", err))
		return nil
	}

	info, err := h.api.Routing().FindPeer(h.ctx, pid)
	if err != nil {
		setError(fmt.Errorf("find peer: %w", err))
		return nil
	}

	var addrs []string
	for _, a := range info.Addrs {
		addrs = append(addrs, a.String())
	}

	result := fmt.Sprintf("%s\t%s", info.ID.String(), strings.Join(addrs, ","))
	setError(nil)
	return C.CString(result)
}

//export kubo_dht_findprovs
func kubo_dht_findprovs(handle uint64, cidStr *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	c, err := cid.Decode(C.GoString(cidStr))
	if err != nil {
		setError(fmt.Errorf("decode cid: %w", err))
		return nil
	}

	p := path.FromCid(c)
	ctx, cancel := context.WithTimeout(h.ctx, 30*time.Second)
	defer cancel()

	provCh, err := h.api.Routing().FindProviders(ctx, p)
	if err != nil {
		setError(fmt.Errorf("find providers: %w", err))
		return nil
	}

	var providers []string
	for info := range provCh {
		var addrs []string
		for _, a := range info.Addrs {
			addrs = append(addrs, a.String())
		}
		providers = append(providers, info.ID.String()+"\t"+strings.Join(addrs, ","))
	}

	setError(nil)
	return C.CString(strings.Join(providers, "\n"))
}

// ---------------------------------------------------------------------------
// Name / IPNS
// ---------------------------------------------------------------------------

//export kubo_name_publish
func kubo_name_publish(handle uint64, cidStr *C.char, lifetimeSec C.int64_t) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	cidStrGo := C.GoString(cidStr)
	var p path.Path
	var err error

	if strings.HasPrefix(cidStrGo, "/ipfs/") || strings.HasPrefix(cidStrGo, "/ipns/") {
		p, err = path.NewPath(cidStrGo)
	} else {
		var c cid.Cid
		c, err = cid.Decode(cidStrGo)
		if err == nil {
			p = path.FromCid(c)
		}
	}
	if err != nil {
		setError(fmt.Errorf("parse path: %w", err))
		return nil
	}

	lifetime := time.Duration(lifetimeSec) * time.Second
	opts := []options.NamePublishOption{options.Name.ValidTime(lifetime)}
	name, err := h.api.Name().Publish(h.ctx, p, opts...)
	if err != nil {
		setError(fmt.Errorf("name publish: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(name.String())
}

//export kubo_name_resolve
func kubo_name_resolve(handle uint64, nameStr *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	name := C.GoString(nameStr)
	ctx, cancel := context.WithTimeout(h.ctx, 30*time.Second)
	defer cancel()

	resolved, err := h.api.Name().Resolve(ctx, name)
	if err != nil {
		setError(fmt.Errorf("name resolve: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(resolved.String())
}

//export kubo_key_gen
func kubo_key_gen(handle uint64, nameStr *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	name := C.GoString(nameStr)
	key, err := h.api.Key().Generate(h.ctx, name)
	if err != nil {
		setError(fmt.Errorf("key gen: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(key.ID().String())
}

//export kubo_key_list
func kubo_key_list(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	keys, err := h.api.Key().List(h.ctx)
	if err != nil {
		setError(fmt.Errorf("key list: %w", err))
		return nil
	}

	var parts []string
	for _, k := range keys {
		parts = append(parts, fmt.Sprintf("%s:%s", k.Name(), k.ID().String()))
	}

	setError(nil)
	return C.CString(strings.Join(parts, "\n"))
}

//export kubo_key_rm
func kubo_key_rm(handle uint64, nameStr *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	name := C.GoString(nameStr)
	key, err := h.api.Key().Remove(h.ctx, name)
	if err != nil {
		setError(fmt.Errorf("key rm: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(key.ID().String())
}

//export kubo_dag_put
func kubo_dag_put(handle uint64, data *C.uint8_t, length C.size_t,
	inputCodecStr *C.char, storeCodecStr *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	inputCodecName := C.GoString(inputCodecStr)
	storeCodecName := C.GoString(storeCodecStr)

	var icodec mc.Code
	if err := icodec.Set(inputCodecName); err != nil {
		setError(fmt.Errorf("invalid input codec: %w", err))
		return nil
	}
	var scodec mc.Code
	if err := scodec.Set(storeCodecName); err != nil {
		setError(fmt.Errorf("invalid store codec: %w", err))
		return nil
	}

	decoder, err := multicodec.LookupDecoder(uint64(icodec))
	if err != nil {
		setError(fmt.Errorf("lookup decoder: %w", err))
		return nil
	}
	encoder, err := multicodec.LookupEncoder(uint64(scodec))
	if err != nil {
		setError(fmt.Errorf("lookup encoder: %w", err))
		return nil
	}

	buf := C.GoBytes(unsafe.Pointer(data), C.int(length))
	node := basicnode.Prototype.Any.NewBuilder()
	if err := decoder(node, bytes.NewReader(buf)); err != nil {
		setError(fmt.Errorf("decode: %w", err))
		return nil
	}
	n := node.Build()

	bd := bytes.NewBuffer([]byte{})
	if err := encoder(n, bd); err != nil {
		setError(fmt.Errorf("encode: %w", err))
		return nil
	}

	cfg, err := h.node.Repo.Config()
	if err != nil {
		setError(fmt.Errorf("repo config: %w", err))
		return nil
	}
	hash := cfg.Import.HashFunction.WithDefault(config.DefaultHashFunction)
	var mhType mc.Code
	if err := mhType.Set(hash); err != nil {
		setError(fmt.Errorf("hash function: %w", err))
		return nil
	}

	prefix := cid.Prefix{
		Version:  1,
		Codec:    uint64(scodec),
		MhType:   uint64(mhType),
		MhLength: -1,
	}
	blockCid, err := prefix.Sum(bd.Bytes())
	if err != nil {
		setError(fmt.Errorf("cid sum: %w", err))
		return nil
	}
	blk, err := blocks.NewBlockWithCid(bd.Bytes(), blockCid)
	if err != nil {
		setError(fmt.Errorf("block: %w", err))
		return nil
	}
	ln := ipldlegacy.LegacyNode{
		Block: blk,
		Node:  n,
	}

	if err := h.api.Dag().Add(h.ctx, &ln); err != nil {
		setError(fmt.Errorf("dag add: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(ln.Cid().String())
}

//export kubo_dag_get
func kubo_dag_get(handle uint64, cidStr *C.char, outputCodecStr *C.char,
	out **C.uint8_t, outLen *C.size_t) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	c, err := cid.Decode(C.GoString(cidStr))
	if err != nil {
		setError(fmt.Errorf("decode cid: %w", err))
		return -1
	}

	codecName := C.GoString(outputCodecStr)
	var codec mc.Code
	if err := codec.Set(codecName); err != nil {
		setError(fmt.Errorf("invalid output codec: %w", err))
		return -1
	}

	obj, err := h.api.Dag().Get(h.ctx, c)
	if err != nil {
		setError(fmt.Errorf("dag get: %w", err))
		return -1
	}

	universal, ok := obj.(ipldlegacy.UniversalNode)
	if !ok {
		setError(fmt.Errorf("%T is not a valid IPLD node", obj))
		return -1
	}
	finalNode := universal

	encoder, err := multicodec.LookupEncoder(uint64(codec))
	if err != nil {
		setError(fmt.Errorf("lookup encoder: %w", err))
		return -1
	}

	bd := bytes.NewBuffer([]byte{})
	if err := encoder(finalNode, bd); err != nil {
		setError(fmt.Errorf("encode: %w", err))
		return -1
	}

	b := bd.Bytes()
	if len(b) == 0 {
		setError(nil)
		*out = nil
		*outLen = 0
		return 0
	}
	cb := C.CBytes(b)
	*out = (*C.uint8_t)(cb)
	*outLen = C.size_t(len(b))
	setError(nil)
	return 0
}

//export kubo_mfs_ls
func kubo_mfs_ls(handle uint64, pathStr *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	p := C.GoString(pathStr)
	node, err := mfs.Lookup(h.node.FilesRoot, p)
	if err != nil {
		setError(fmt.Errorf("mfs lookup: %w", err))
		return nil
	}

	dir, ok := node.(*mfs.Directory)
	if !ok {
		setError(fmt.Errorf("%s is not a directory", p))
		return nil
	}

	names, err := dir.ListNames(h.ctx)
	if err != nil {
		setError(fmt.Errorf("mfs ls: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(strings.Join(names, "\n"))
}

//export kubo_mfs_read
func kubo_mfs_read(handle uint64, pathStr *C.char, out **C.uint8_t, outLen *C.size_t) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	p := C.GoString(pathStr)
	node, err := mfs.Lookup(h.node.FilesRoot, p)
	if err != nil {
		setError(fmt.Errorf("mfs lookup: %w", err))
		return -1
	}

	fi, ok := node.(*mfs.File)
	if !ok {
		setError(fmt.Errorf("%s is not a file", p))
		return -1
	}

	fd, err := fi.Open(h.ctx, mfs.Flags{Read: true})
	if err != nil {
		setError(fmt.Errorf("mfs open: %w", err))
		return -1
	}
	defer fd.Close()

	size, _ := fd.Size()
	buf := make([]byte, size)
	_, err = fd.Read(buf)
	if err != nil && err.Error() != "EOF" {
		setError(fmt.Errorf("mfs read: %w", err))
		return -1
	}

	if len(buf) == 0 {
		setError(nil)
		*out = nil
		*outLen = 0
		return 0
	}
	cb := C.CBytes(buf)
	*out = (*C.uint8_t)(cb)
	*outLen = C.size_t(len(buf))
	setError(nil)
	return 0
}

//export kubo_mfs_write
func kubo_mfs_write(handle uint64, pathStr *C.char, data *C.uint8_t, length C.size_t) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	p := C.GoString(pathStr)
	buf := C.GoBytes(unsafe.Pointer(data), C.int(length))

	node, err := mfs.Lookup(h.node.FilesRoot, p)
	if err != nil {
		setError(fmt.Errorf("mfs lookup: %w", err))
		return -1
	}

	fi, ok := node.(*mfs.File)
	if !ok {
		setError(fmt.Errorf("%s is not a file", p))
		return -1
	}

	fd, err := fi.Open(h.ctx, mfs.Flags{Write: true, Sync: true})
	if err != nil {
		setError(fmt.Errorf("mfs open: %w", err))
		return -1
	}
	defer fd.Close()

	if err := fd.Truncate(0); err != nil {
		setError(fmt.Errorf("mfs truncate: %w", err))
		return -1
	}

	if _, err := fd.Write(buf); err != nil {
		setError(fmt.Errorf("mfs write: %w", err))
		return -1
	}

	if err := fd.Flush(); err != nil {
		setError(fmt.Errorf("mfs flush: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_mfs_mkdir
func kubo_mfs_mkdir(handle uint64, pathStr *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	p := C.GoString(pathStr)
	if err := mfs.Mkdir(h.node.FilesRoot, p, mfs.MkdirOpts{Mkparents: true}); err != nil {
		setError(fmt.Errorf("mfs mkdir: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_mfs_rm
func kubo_mfs_rm(handle uint64, pathStr *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	p := C.GoString(pathStr)
	if p == "/" {
		setError(fmt.Errorf("cannot delete root"))
		return -1
	}
	if len(p) > 0 && p[len(p)-1] == '/' {
		p = p[:len(p)-1]
	}

	dir, name := gopath.Split(p)
	parent, err := mfs.Lookup(h.node.FilesRoot, dir)
	if err != nil {
		setError(fmt.Errorf("mfs lookup parent: %w", err))
		return -1
	}
	pdir, ok := parent.(*mfs.Directory)
	if !ok {
		setError(fmt.Errorf("mfs rm: parent is not a directory"))
		return -1
	}

	if err := pdir.Unlink(name); err != nil {
		setError(fmt.Errorf("mfs rm: %w", err))
		return -1
	}
	if err := pdir.Flush(); err != nil {
		setError(fmt.Errorf("mfs rm flush: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_mfs_flush
func kubo_mfs_flush(handle uint64, pathStr *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	p := C.GoString(pathStr)
	nd, err := mfs.FlushPath(h.ctx, h.node.FilesRoot, p)
	if err != nil {
		setError(fmt.Errorf("mfs flush: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(nd.Cid().String())
}

//export kubo_mfs_stat
func kubo_mfs_stat(handle uint64, pathStr *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	p := C.GoString(pathStr)
	node, err := mfs.Lookup(h.node.FilesRoot, p)
	if err != nil {
		setError(fmt.Errorf("mfs lookup: %w", err))
		return nil
	}

	var result string
	switch n := node.(type) {
	case *mfs.File:
		size, _ := n.Size()
		result = fmt.Sprintf("file\t%d", size)
	case *mfs.Directory:
		result = "directory\t0"
	default:
		result = "unknown\t0"
	}

	setError(nil)
	return C.CString(result)
}
