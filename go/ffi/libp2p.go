package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"context"
	"fmt"
	"os"
	"strings"
	"sync"
	"time"

	dht "github.com/libp2p/go-libp2p-kad-dht"
	"github.com/libp2p/go-libp2p"
	pubsub "github.com/libp2p/go-libp2p-pubsub"
	"github.com/libp2p/go-libp2p/core/host"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/libp2p/go-libp2p/core/routing"
	"github.com/libp2p/go-libp2p/p2p/host/autorelay"
	"github.com/libp2p/go-libp2p/p2p/protocol/ping"
	ma "github.com/multiformats/go-multiaddr"
)

var (
	libp2pHostsMu    sync.RWMutex
	libp2pHosts      = make(map[uint64]*libp2pHostHandle)
	libp2pNextHandle uint64 = 1
)

type libp2pHostHandle struct {
	ctx    context.Context
	cancel context.CancelFunc
	host   host.Host

	pubsubMu       sync.Mutex
	pubsub         *pubsub.PubSub
	pubsubTopic    *pubsub.Topic        // default topic (backward compat)
	pubsubSub      *pubsub.Subscription // default sub (backward compat)
	pubsubTopicID  string               // default topic name
	pubsubTopics   map[string]*pubsub.Topic
	pubsubSubs     map[string]*pubsub.Subscription
	pubsubMsgs     []string
	gossipInitErr  string
}

func bootstrapEnabled() bool {
	value := strings.TrimSpace(strings.ToLower(os.Getenv("KUBO_LIBP2P_BOOTSTRAP")))
	if value == "" {
		return true
	}

	switch value {
	case "1", "true", "yes", "on":
		return true
	default:
		return false
	}
}

func gossipEnabled() bool {
	value := strings.TrimSpace(strings.ToLower(os.Getenv("KUBO_LIBP2P_GOSSIP")))
	if value == "" {
		return true
	}

	switch value {
	case "1", "true", "yes", "on":
		return true
	default:
		return false
	}
}

func gossipTopicName() string {
	topic := strings.TrimSpace(os.Getenv("KUBO_LIBP2P_GOSSIP_TOPIC"))
	if topic == "" {
		return "kubo-desktop"
	}
	return topic
}

func configuredBootstrapPeers() []peer.AddrInfo {
	if !bootstrapEnabled() {
		return nil
	}

	peers := make([]peer.AddrInfo, 0, len(dht.DefaultBootstrapPeers))
	for _, addr := range dht.DefaultBootstrapPeers {
		info, err := peer.AddrInfoFromP2pAddr(addr)
		if err != nil {
			continue
		}
		peers = append(peers, *info)
	}
	return peers
}

func relayPeerSource(static []peer.AddrInfo) autorelay.PeerSource {
	return func(ctx context.Context, num int) <-chan peer.AddrInfo {
		out := make(chan peer.AddrInfo, num)
		go func() {
			defer close(out)
			if len(static) == 0 {
				return
			}
			if num > len(static) {
				num = len(static)
			}
			for i := 0; i < num; i++ {
				select {
				case <-ctx.Done():
					return
				case out <- static[i]:
				}
			}
		}()
		return out
	}
}

func connectBootstrapPeers(ctx context.Context, h host.Host, peers []peer.AddrInfo) {
	for _, info := range peers {
		info := info
		go func() {
			connectCtx, cancel := context.WithTimeout(ctx, 10*time.Second)
			defer cancel()
			_ = h.Connect(connectCtx, info)
		}()
	}
}

func startGossip(handle *libp2pHostHandle) error {
	if !gossipEnabled() {
		return nil
	}

	ps, err := pubsub.NewGossipSub(handle.ctx, handle.host, pubsub.WithFloodPublish(true))
	if err != nil {
		return fmt.Errorf("pubsub new: %w", err)
	}

	topicName := gossipTopicName()
	topic, err := ps.Join(topicName)
	if err != nil {
		return fmt.Errorf("pubsub join %s: %w", topicName, err)
	}

	sub, err := topic.Subscribe()
	if err != nil {
		return fmt.Errorf("pubsub subscribe %s: %w", topicName, err)
	}

	handle.pubsubMu.Lock()
	handle.pubsub = ps
	handle.pubsubTopic = topic
	handle.pubsubSub = sub
	handle.pubsubTopicID = topicName
	handle.pubsubTopics = make(map[string]*pubsub.Topic)
	handle.pubsubSubs = make(map[string]*pubsub.Subscription)
	handle.pubsubMu.Unlock()

	go func() {
		for {
			msg, err := sub.Next(handle.ctx)
			if err != nil {
				return
			}

			payload := strings.TrimSpace(string(msg.Data))
			if payload == "" || msg.ReceivedFrom == handle.host.ID() {
				continue
			}

			entry := fmt.Sprintf("%s|%s|%s", topicName, msg.ReceivedFrom.String(), payload)
			handle.pubsubMu.Lock()
			handle.pubsubMsgs = append(handle.pubsubMsgs, entry)
			handle.pubsubMu.Unlock()
		}
	}()

	return nil
}

func drainPubsubMessages(handle *libp2pHostHandle) []string {
	handle.pubsubMu.Lock()
	defer handle.pubsubMu.Unlock()

	if len(handle.pubsubMsgs) == 0 {
		return nil
	}

	out := append([]string(nil), handle.pubsubMsgs...)
	handle.pubsubMsgs = nil
	return out
}

//export kubo_libp2p_host_new
func kubo_libp2p_host_new() uint64 {
	ctx, cancel := context.WithCancel(context.Background())

	bootstrapPeers := configuredBootstrapPeers()
	var idht *dht.IpfsDHT

	opts := []libp2p.Option{
		libp2p.ListenAddrStrings(
			"/ip4/0.0.0.0/tcp/0",
			"/ip6/::/tcp/0",
			"/ip4/0.0.0.0/udp/0/quic-v1",
			"/ip6/::/udp/0/quic-v1",
		),
		libp2p.NATPortMap(),
		libp2p.EnableNATService(),
		libp2p.EnableHolePunching(),
	}

	if len(bootstrapPeers) > 0 {
		opts = append(opts, libp2p.EnableAutoRelayWithPeerSource(relayPeerSource(bootstrapPeers)))
	}

	opts = append(opts, libp2p.Routing(func(h host.Host) (routing.PeerRouting, error) {
		kademliaDHT, err := dht.New(h)
		if err != nil {
			return nil, err
		}
		idht = kademliaDHT
		return kademliaDHT, nil
	}))

	h, err := libp2p.New(opts...)
	if err != nil {
		cancel()
		setError(fmt.Errorf("libp2p new: %w", err))
		return 0
	}

	if idht != nil {
		go func() {
			if err := idht.Bootstrap(ctx); err != nil {
				fmt.Fprintf(os.Stderr, "libp2p bootstrap: %v\n", err)
			}
		}()
	}

	if len(bootstrapPeers) > 0 {
		connectBootstrapPeers(ctx, h, bootstrapPeers)
	}

	handle := &libp2pHostHandle{
		ctx:    ctx,
		cancel: cancel,
		host:   h,
	}

	if err := startGossip(handle); err != nil {
		// Log the error but don't kill the host; other libp2p features still work.
		fmt.Fprintf(os.Stderr, "libp2p gossip init failed: %v\n", err)
		handle.gossipInitErr = err.Error()
		setError(err)
	}

	libp2pHostsMu.Lock()
	hid := libp2pNextHandle
	libp2pNextHandle++
	libp2pHosts[hid] = handle
	libp2pHostsMu.Unlock()

	setError(nil)
	return hid
}

//export kubo_libp2p_host_close
func kubo_libp2p_host_close(handle uint64) int64 {
	libp2pHostsMu.Lock()
	h, ok := libp2pHosts[handle]
	if ok {
		delete(libp2pHosts, handle)
	}
	libp2pHostsMu.Unlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return -1
	}

	h.cancel()
	if err := h.host.Close(); err != nil {
		setError(fmt.Errorf("close host: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_libp2p_host_peer_id
func kubo_libp2p_host_peer_id(handle uint64) *C.char {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return nil
	}

	setError(nil)
	return C.CString(h.host.ID().String())
}

//export kubo_libp2p_host_listening_addrs
func kubo_libp2p_host_listening_addrs(handle uint64) *C.char {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return nil
	}

	var parts []string
	for _, a := range h.host.Addrs() {
		parts = append(parts, a.String())
	}

	setError(nil)
	return C.CString(strings.Join(parts, "\n"))
}

//export kubo_libp2p_host_connect
func kubo_libp2p_host_connect(handle uint64, addr *C.char) int64 {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
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

	if err := h.host.Connect(h.ctx, *info); err != nil {
		setError(fmt.Errorf("connect: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_libp2p_host_ping
func kubo_libp2p_host_ping(handle uint64, peer_id *C.char) int64 {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return -1
	}

	pid, err := peer.Decode(C.GoString(peer_id))
	if err != nil {
		setError(fmt.Errorf("decode peer id: %w", err))
		return -1
	}

	ctx, cancel := context.WithTimeout(h.ctx, 10*time.Second)
	defer cancel()

	result := <-ping.Ping(ctx, h.host, pid)
	if result.Error != nil {
		setError(fmt.Errorf("ping: %w", result.Error))
		return -1
	}

	setError(nil)
	return int64(result.RTT.Milliseconds())
}

//export kubo_libp2p_host_protocols
func kubo_libp2p_host_protocols(handle uint64) *C.char {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return nil
	}

	protos := h.host.Mux().Protocols()
	var names []string
	for _, p := range protos {
		names = append(names, string(p))
	}

	setError(nil)
	return C.CString(strings.Join(names, "\n"))
}

//export kubo_libp2p_host_gossip_topic
func kubo_libp2p_host_gossip_topic(handle uint64) *C.char {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return nil
	}

	h.pubsubMu.Lock()
	topic := h.pubsubTopicID
	h.pubsubMu.Unlock()

	setError(nil)
	return C.CString(topic)
}

//export kubo_libp2p_host_gossip_error
func kubo_libp2p_host_gossip_error(handle uint64) *C.char {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return nil
	}

	setError(nil)
	if h.gossipInitErr == "" {
		return nil
	}
	return C.CString(h.gossipInitErr)
}

//export kubo_libp2p_host_gossip_publish
func kubo_libp2p_host_gossip_publish(handle uint64, message *C.char) int64 {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return -1
	}

	msg := C.GoString(message)
	h.pubsubMu.Lock()
	topic := h.pubsubTopic
	topicName := h.pubsubTopicID
	h.pubsubMu.Unlock()

	if topic == nil {
		setError(fmt.Errorf("gossip topic disabled"))
		return -1
	}

	if err := topic.Publish(h.ctx, []byte(msg)); err != nil {
		setError(fmt.Errorf("pubsub publish: %w", err))
		return -1
	}

	h.pubsubMu.Lock()
	h.pubsubMsgs = append(h.pubsubMsgs, fmt.Sprintf("%s|self|%s", topicName, msg))
	h.pubsubMu.Unlock()

	setError(nil)
	return 0
}

//export kubo_libp2p_host_gossip_join
func kubo_libp2p_host_gossip_join(handle uint64, topicName *C.char) int64 {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return -1
	}

	name := C.GoString(topicName)
	h.pubsubMu.Lock()
	ps := h.pubsub
	if ps == nil {
		h.pubsubMu.Unlock()
		setError(fmt.Errorf("pubsub not initialized"))
		return -1
	}
	if _, exists := h.pubsubTopics[name]; exists {
		h.pubsubMu.Unlock()
		setError(nil)
		return 0
	}
	h.pubsubMu.Unlock()

	topic, err := ps.Join(name)
	if err != nil {
		setError(fmt.Errorf("pubsub join %s: %w", name, err))
		return -1
	}

	sub, err := topic.Subscribe()
	if err != nil {
		setError(fmt.Errorf("pubsub subscribe %s: %w", name, err))
		return -1
	}

	h.pubsubMu.Lock()
	h.pubsubTopics[name] = topic
	h.pubsubSubs[name] = sub
	h.pubsubMu.Unlock()

	go func() {
		for {
			msg, err := sub.Next(h.ctx)
			if err != nil {
				return
			}

			payload := strings.TrimSpace(string(msg.Data))
			if payload == "" || msg.ReceivedFrom == h.host.ID() {
				continue
			}

			entry := fmt.Sprintf("%s|%s|%s", name, msg.ReceivedFrom.String(), payload)
			h.pubsubMu.Lock()
			h.pubsubMsgs = append(h.pubsubMsgs, entry)
			h.pubsubMu.Unlock()
		}
	}()

	setError(nil)
	return 0
}

//export kubo_libp2p_host_gossip_leave
func kubo_libp2p_host_gossip_leave(handle uint64, topicName *C.char) int64 {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return -1
	}

	name := C.GoString(topicName)
	h.pubsubMu.Lock()
	if sub, exists := h.pubsubSubs[name]; exists {
		sub.Cancel()
		delete(h.pubsubSubs, name)
	}
	if topic, exists := h.pubsubTopics[name]; exists {
		_ = topic.Close()
		delete(h.pubsubTopics, name)
	}
	h.pubsubMu.Unlock()

	setError(nil)
	return 0
}

//export kubo_libp2p_host_gossip_publish_to
func kubo_libp2p_host_gossip_publish_to(handle uint64, topicName *C.char, message *C.char) int64 {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return -1
	}

	name := C.GoString(topicName)
	msg := C.GoString(message)

	h.pubsubMu.Lock()
	topic, exists := h.pubsubTopics[name]
	if !exists {
		// fallback to default topic if name matches
		if h.pubsubTopicID == name && h.pubsubTopic != nil {
			topic = h.pubsubTopic
			exists = true
		}
	}
	h.pubsubMu.Unlock()

	if !exists {
		setError(fmt.Errorf("gossip topic %s not joined", name))
		return -1
	}

	if err := topic.Publish(h.ctx, []byte(msg)); err != nil {
		setError(fmt.Errorf("pubsub publish: %w", err))
		return -1
	}

	h.pubsubMu.Lock()
	h.pubsubMsgs = append(h.pubsubMsgs, fmt.Sprintf("%s|self|%s", name, msg))
	h.pubsubMu.Unlock()

	setError(nil)
	return 0
}

//export kubo_libp2p_host_gossip_drain
func kubo_libp2p_host_gossip_drain(handle uint64) *C.char {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return nil
	}

	msgs := drainPubsubMessages(h)
	setError(nil)
	return C.CString(strings.Join(msgs, "\x1e"))
}
