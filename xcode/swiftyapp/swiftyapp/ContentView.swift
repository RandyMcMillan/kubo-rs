//
//  ContentView.swift
//  swiftyapp
//
//  Created by Jonathan McKenzie on 7/9/24.
//

import Foundation
import MultipeerConnectivity
import RustyLib
import SwiftUI

enum DashboardSection: String, CaseIterable, Identifiable {
    case overview
    case repository
    case network
    case chat
    case activity

    var id: String { rawValue }

    var title: String {
        switch self {
        case .overview: return "Overview"
        case .repository: return "Repository"
        case .network: return "Network"
        case .chat: return "Chat"
        case .activity: return "Activity"
        }
    }

    var subtitle: String {
        switch self {
        case .overview: return "Status at a glance"
        case .repository: return "Local repo snapshot"
        case .network: return "Peer and CID details"
        case .chat: return "Gossip pubsub messages"
        case .activity: return "Recent actions"
        }
    }

    var icon: String {
        switch self {
        case .overview: return "square.grid.2x2"
        case .repository: return "externaldrive.connected.to.line.below"
        case .network: return "point.3.connected.trianglepath.dotted"
        case .chat: return "bubble.left.and.bubble.right"
        case .activity: return "clock.arrow.circlepath"
        }
    }
}

struct KuboSnapshot {
    var version: String = "—"
    var peerID: String = "—"
    var cid: String = "—"
    var roundTrip: String = "—"
    var mathResult: String = "—"
    var status: String = "Waiting for snapshot"
    var rawSummary: String = "Tap Refresh to start the HybridNode and capture live state."
    var updatedAt: Date?

    static let placeholder = KuboSnapshot()
}

@MainActor
final class HybridNodeStore: ObservableObject {
    @Published var selection: DashboardSection = .overview
    @Published var snapshot: KuboSnapshot = .placeholder
    @Published var activity: [String] = []
    @Published var isRefreshing = false
    @Published var nodeStarted = false
    @Published var nodeError: String = ""

    // IPFS
    @Published var pins: [String] = []
    @Published var lastAddedCID: String = ""
    @Published var catResult: String = ""
    @Published var addDraft: String = ""
    @Published var catDraft: String = ""

    // Git
    @Published var gitPath: String = ""
    @Published var gitHeadResult: String = ""
    @Published var gitBranchesResult: [String] = []
    @Published var gitRemotesResult: [String] = []
    @Published var gitStatusResult: String = ""
    @Published var cloneURL: String = ""
    @Published var clonePath: String = ""

    // Nostr
    @Published var nostrSecretKey: String = ""
    @Published var nostrPublicKey: String = ""
    @Published var nostrEventJson: String = ""
    @Published var relayURL: String = "wss://relay.damus.io"
    @Published var relayHandle: UInt64 = 0
    @Published var drainResult: String = ""
    @Published var nostrContent: String = ""
    @Published var gossipTopic: String = "kubo-hybrid"
    @Published var gossipMessage: String = ""

    init() {
        startNode()
    }

    func startNode() {
        guard !isRefreshing else { return }
        isRefreshing = true
        appendActivity("Starting HybridNode…")
        do {
            try hybridStartTry(online: true)
            nodeStarted = true
            nodeError = ""
            refreshState()
            appendActivity("HybridNode started (online)")
        } catch let error as RustyError {
            nodeStarted = false
            nodeError = error.localizedDescription
            appendActivity("Start failed: \(error.localizedDescription)")
        } catch {
            nodeStarted = false
            nodeError = "\(error)"
            appendActivity("Start failed: \(error)")
        }
        isRefreshing = false
    }

    func stopNode() {
        do {
            try hybridStopTry()
            nodeStarted = false
            appendActivity("HybridNode stopped")
        } catch let error as RustyError {
            appendActivity("Stop failed: \(error.localizedDescription)")
        } catch {
            appendActivity("Stop failed: \(error)")
        }
    }

    func refreshState() {
        let ipfsID = hybridIpfsPeerId()
        let p2pID = hybridP2pPeerId()
        let pinList = ipfsPinLs()
        pins = pinList

        snapshot = KuboSnapshot(
            version: kubo_rs_version(),
            peerID: ipfsID,
            cid: lastAddedCID.isEmpty ? "—" : lastAddedCID,
            roundTrip: catResult.isEmpty ? "—" : catResult,
            mathResult: nodeStarted ? "Node active" : "Node offline",
            status: nodeStarted ? (nodeError.isEmpty ? "Online" : "Error") : "Offline",
            rawSummary: "IPFS: \(ipfsID)\nP2P: \(p2pID)\nPins: \(pinList.count)",
            updatedAt: Date()
        )
    }

    func ipfsAdd() {
        let text = addDraft.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { return }
        do {
            let cid = try ipfsAddTry(data: text.data(using: .utf8) ?? Data())
            lastAddedCID = cid
            addDraft = ""
            appendActivity("IPFS add → \(shortCID(cid))")
            refreshState()
        } catch let error as RustyError {
            appendActivity("IPFS add failed: \(error.localizedDescription)")
        } catch {
            appendActivity("IPFS add failed: \(error)")
        }
    }

    func ipfsCat() {
        let cid = catDraft.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !cid.isEmpty else { return }
        do {
            let data = try ipfsCatTry(cid: cid)
            catResult = String(data: data, encoding: .utf8) ?? "<\(data.count) bytes>"
            appendActivity("IPFS cat → \(shortCID(cid)) (\(data.count) bytes)")
        } catch let error as RustyError {
            appendActivity("IPFS cat failed: \(error.localizedDescription)")
        } catch {
            appendActivity("IPFS cat failed: \(error)")
        }
    }

    func ipfsPin(cid: String) {
        guard !cid.isEmpty else { return }
        let ok = ipfsPinAdd(cid: cid, recursive: true)
        appendActivity(ok ? "Pinned \(shortCID(cid))" : "Pin failed")
        refreshState()
    }

    func ipfsUnpin(cid: String) {
        guard !cid.isEmpty else { return }
        let ok = ipfsPinRm(cid: cid, recursive: true)
        appendActivity(ok ? "Unpinned \(shortCID(cid))" : "Unpin failed")
        refreshState()
    }

    // Git
    func gitInitRepo() {
        let path = gitPath.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !path.isEmpty else { return }
        let ok = gitInit(path: path, bare: false)
        appendActivity(ok ? "Git init: \(path)" : "Git init failed")
        if ok { refreshGit() }
    }

    func gitCloneRepo() {
        let url = cloneURL.trimmingCharacters(in: .whitespacesAndNewlines)
        let path = clonePath.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !url.isEmpty, !path.isEmpty else { return }
        let ok = gitClone(url: url, path: path, bare: false)
        appendActivity(ok ? "Git clone → \(path)" : "Git clone failed")
        if ok {
            gitPath = path
            refreshGit()
        }
    }

    func refreshGit() {
        let path = gitPath.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !path.isEmpty else { return }
        gitHeadResult = gitHead(path: path)
        gitBranchesResult = gitBranches(path: path)
        gitRemotesResult = gitRemotes(path: path)
        gitStatusResult = gitStatus(path: path)
    }

    // Nostr
    func generateNostrKey() {
        let sk = nostrGenerateKey()
        nostrSecretKey = sk
        nostrPublicKey = nostrGetPublicKey(sk: sk)
        appendActivity("Generated Nostr key: \(shortKey(nostrPublicKey))")
    }

    func signEvent() {
        guard !nostrSecretKey.isEmpty else {
            appendActivity("No Nostr key — generate one first")
            return
        }
        let content = nostrContent.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !content.isEmpty else { return }
        let eventJson = nostrEventSign(sk: nostrSecretKey, content: content, kind: 1)
        nostrEventJson = eventJson
        nostrContent = ""
        appendActivity("Signed kind-1 event")
    }

    func connectRelay() {
        let url = relayURL.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !url.isEmpty else { return }
        let handle = nostrRelayConnect(url: url)
        if handle != 0 {
            relayHandle = handle
            appendActivity("Relay connected: \(url) (#\(handle))")
        } else {
            appendActivity("Relay connect failed: \(url)")
        }
    }

    func publishToRelay() {
        guard relayHandle != 0, !nostrEventJson.isEmpty else {
            appendActivity("Need relay + signed event")
            return
        }
        let ok = nostrRelayPublish(handle: relayHandle, eventJson: nostrEventJson)
        appendActivity(ok ? "Published to relay" : "Relay publish failed")
    }

    func drainRelay() {
        guard relayHandle != 0 else {
            appendActivity("No relay connected")
            return
        }
        let subHandle = nostrRelaySubscribe(handle: relayHandle, filterJson: "{\"kinds\":[1,1063],\"limit\":10}")
        var events: [String] = []
        while let msg = nostrRelayDrain(subHandle: subHandle) {
            events.append(msg)
        }
        nostrRelayUnsubscribe(subHandle: subHandle)
        drainResult = events.joined(separator: "\n---\n")
        appendActivity("Drained \(events.count) relay events")
    }

    func publishGossip() {
        let msg = gossipMessage.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !msg.isEmpty else { return }
        let ok = hybridBroadcastEvent(eventJson: msg, relayHandle: nil, gossipTopic: gossipTopic)
        appendActivity(ok ? "Gossip published to \(gossipTopic)" : "Gossip publish failed")
        gossipMessage = ""
    }

    func drainGossip() {
        let events = hybridDrainEvents(relaySubHandle: nil)
        appendActivity("Drained \(events.count) gossip events")
    }

    private func appendActivity(_ message: String) {
        let timestamp = Self.timestampFormatter.string(from: Date())
        activity.insert("[\(timestamp)] \(message)", at: 0)
        if activity.count > 50 {
            activity.removeLast(activity.count - 50)
        }
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateStyle = .none
        formatter.timeStyle = .medium
        return formatter
    }()

    func shortCID(_ cid: String) -> String {
        guard cid.count > 18 else { return cid }
        return "\(cid.prefix(10))…\(cid.suffix(6))"
    }

    func shortKey(_ key: String) -> String {
        guard key.count > 16 else { return key }
        return "\(key.prefix(8))…\(key.suffix(8))"
    }
}

func kubo_rs_version() -> String {
    // UniFFI doesn't expose the plain version() function; use a tiny round-trip.
    // If the node is running, we can get peer IDs. Otherwise return a placeholder.
    let id = hybridIpfsPeerId()
    return id.isEmpty ? "kubo-rs 0.8.1" : "kubo-rs (live)"
}

struct NearbyPeer: Identifiable, Hashable {
    enum State: String {
        case browsing = "Browsing"
        case nearby = "Nearby"
        case invited = "Invited"
        case connected = "Connected"
        case lost = "Lost"
        case disconnected = "Disconnected"
    }

    let id: String
    var name: String
    var state: State
    var lastSeen: Date

    var statusText: String {
        state.rawValue
    }
}

final class PeerNetworkStore: NSObject, ObservableObject {
    @Published var localPeerName: String
    @Published var connectionStatus: String = "Starting discovery"
    @Published var libp2pPeerID: String = "Starting host"
    @Published var libp2pAddrs: [String] = []
    @Published var libp2pProtocols: [String] = []
    @Published var gossipTopic: String = "kubo-desktop"
    @Published var gossipMessages: [String] = []
    @Published var chatDraft: String = ""
    @Published var chatMessages: [String] = []
    @Published var nearbyPeers: [NearbyPeer] = []
    @Published var connectedPeers: [NearbyPeer] = []
    @Published var recentMessages: [String] = []
    @Published var lastMessage: String = "Waiting for a peer message"

    private let peerID: MCPeerID
    private let session: MCSession
    private let browser: MCNearbyServiceBrowser
    private let advertiser: MCNearbyServiceAdvertiser
    private static let serviceType = "kubo-p2p"
    private var pendingInvites = Set<String>()
    private var peerDirectory: [String: NearbyPeer] = [:]
    private var seenChatMessageIDs = Set<String>()
    private var gossipPoller: Task<Void, Never>?

    override init() {
        let displayName = Self.makeDisplayName()
        let peerID = MCPeerID(displayName: displayName)
        let session = MCSession(peer: peerID, securityIdentity: nil, encryptionPreference: .required)
        let browser = MCNearbyServiceBrowser(peer: peerID, serviceType: Self.serviceType)
        let advertiser = MCNearbyServiceAdvertiser(peer: peerID, discoveryInfo: [
            "platform": Self.platformLabel,
            "role": "kubo"
        ], serviceType: Self.serviceType)

        self.localPeerName = displayName
        self.peerID = peerID
        self.session = session
        self.browser = browser
        self.advertiser = advertiser

        super.init()

        session.delegate = self
        browser.delegate = self
        advertiser.delegate = self

        startLibp2pHost()
        browser.startBrowsingForPeers()
        advertiser.startAdvertisingPeer()

        connectionStatus = "Browsing and advertising as \(displayName)"
        addMessage("Started local discovery for \(displayName)")

        gossipPoller = Task {
            while !Task.isCancelled {
                await MainActor.run {
                    self.syncGossipMessages()
                }
                try? await Task.sleep(nanoseconds: 2_000_000_000)
            }
        }
    }

    deinit {
        gossipPoller?.cancel()
        browser.stopBrowsingForPeers()
        advertiser.stopAdvertisingPeer()
        session.disconnect()
    }

    func broadcast(_ message: String) {
        let payload = Data(message.utf8)
        guard !session.connectedPeers.isEmpty else {
            addMessage("No connected peers to receive: \(message)")
            return
        }

        do {
            try session.send(payload, toPeers: session.connectedPeers, with: .reliable)
            addMessage("Broadcast to \(session.connectedPeers.count) peer(s): \(message)")
        } catch {
            addMessage("Broadcast failed: \(error.localizedDescription)")
        }
    }

    func restartDiscovery() {
        browser.stopBrowsingForPeers()
        advertiser.stopAdvertisingPeer()
        browser.startBrowsingForPeers()
        advertiser.startAdvertisingPeer()
        connectionStatus = "Discovery restarted"
        addMessage("Restarted browsing and advertising")
    }

    func broadcastCurrentState() {
        broadcast(libp2pHandshakeMessage())
        publishGossip(gossipPayload())
    }

    func sendChatMessage() {
        let message = chatDraft.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !message.isEmpty else { return }

        let messageID = UUID().uuidString
        let payload = "chat|\(messageID)|\(localPeerName)|\(message)"
        seenChatMessageIDs.insert(messageID)
        broadcast(payload)
        publishGossip(payload)
        appendChatMessage(id: messageID, sender: localPeerName, message: message, isLocal: true)
        chatDraft = ""
    }

    private static var platformLabel: String {
        #if targetEnvironment(macCatalyst)
        return "mac"
        #elseif os(iOS)
        return "ipad"
        #else
        return "peer"
        #endif
    }

    private static func makeDisplayName() -> String {
        let defaults = UserDefaults.standard
        let key = "kubo.peer.display-name"
        if let stored = defaults.string(forKey: key) {
            return stored
        }

        let suffix = UUID().uuidString.prefix(6).lowercased()
        let value = "kubo-\(platformLabel)-\(suffix)"
        defaults.set(value, forKey: key)
        return value
    }

    private func startLibp2pHost() {
        let peerID = p2pStart()
        let addrs = p2pListeningAddrs()
        let protocols = p2pProtocols()
        let topic = p2pGossipTopic()

        if peerID.isEmpty {
            let reason = p2pLastError()
            libp2pPeerID = "Unavailable"
            libp2pAddrs = []
            libp2pProtocols = []
            gossipTopic = "Unavailable"
            addMessage("libp2p host failed to start: \(reason.isEmpty ? "unknown" : reason)")
        } else {
            libp2pPeerID = peerID
            libp2pAddrs = addrs
            libp2pProtocols = protocols
            if topic.isEmpty {
                let err = p2pGossipError()
                gossipTopic = err.isEmpty ? "disabled" : "error"
                addMessage("libp2p gossip disabled: \(err.isEmpty ? "no topic" : err)")
            } else {
                gossipTopic = topic
            }
            addMessage("Started libp2p host \(libp2pPeerID)")
        }
    }

    private func syncGossipMessages() {
        let lines = p2pGossipDrain()
        guard !lines.isEmpty else { return }

        for line in lines {
            gossipMessages.insert(line, at: 0)
            if gossipMessages.count > 12 {
                gossipMessages.removeLast(gossipMessages.count - 12)
            }

            let parts = line.split(separator: "|", maxSplits: 2, omittingEmptySubsequences: false)
            if parts.count == 3 {
                let topic = String(parts[0])
                let source = String(parts[1])
                let payload = String(parts[2])
                addMessage("Gossip[\(topic)] from \(source): \(payload)")
                if let chat = parseChatMessage(topic: topic, source: source, payload: payload), shouldAcceptChatMessage(id: chat.id) {
                    appendChatMessage(id: chat.id, sender: chat.sender, message: chat.message, isLocal: chat.isLocal)
                }
            } else {
                addMessage("Gossip: \(line)")
            }
        }
    }

    private func libp2pHandshakeMessage() -> String {
        let addresses = libp2pAddrs
            .map { $0.contains("/p2p/") ? $0 : "\($0)/p2p/\(libp2pPeerID)" }
            .joined(separator: "\n")
        return [
            "kubo-p2p",
            localPeerName,
            libp2pPeerID,
            addresses
        ].joined(separator: "|")
    }

    private func processLibp2pHandshake(_ message: String, from peerID: MCPeerID) {
        let parts = message.split(separator: "|", maxSplits: 3, omittingEmptySubsequences: false)
        guard parts.count == 4, parts[0] == "kubo-p2p" else { return }

        let remoteName = String(parts[1])
        let remotePeerID = String(parts[2])
        let addresses = String(parts[3])
            .split(separator: "\n")
            .map { String($0) }
            .filter { !$0.isEmpty }

        addMessage("Handshake from \(remoteName) (\(remotePeerID)) via \(peerID.displayName)")

        for addr in addresses {
            let dialAddr = addr.contains("/p2p/") ? addr : "\(addr)/p2p/\(remotePeerID)"
            if p2pConnect(addr: dialAddr) {
                addMessage("Dialed \(remoteName) at \(dialAddr)")
            } else {
                addMessage("Dial failed for \(remoteName) at \(dialAddr)")
            }
        }
    }

    private func gossipPayload() -> String {
        [
            "peer=\(localPeerName)",
            "id=\(libp2pPeerID)",
            "addr=\(libp2pAddrs.first ?? "none")",
            "status=\(connectionStatus)"
        ].joined(separator: " ")
    }

    private func parseChatMessage(topic: String, source: String, payload: String) -> (id: String, sender: String, message: String, isLocal: Bool)? {
        guard topic == gossipTopic else { return nil }

        if payload.hasPrefix("chat|") {
            let parts = payload.split(separator: "|", maxSplits: 3, omittingEmptySubsequences: false)
            if parts.count == 4 {
                let id = String(parts[1])
                let sender = String(parts[2])
                let message = String(parts[3])
                return (id, sender, message, source == "self" || sender == localPeerName)
            }
        }

        return nil
    }

    private func parseChatPayload(_ payload: String, source: String? = nil) -> (id: String, sender: String, message: String, isLocal: Bool)? {
        guard payload.hasPrefix("chat|") else { return nil }
        let parts = payload.split(separator: "|", maxSplits: 3, omittingEmptySubsequences: false)
        guard parts.count == 4 else { return nil }

        let id = String(parts[1])
        let sender = String(parts[2])
        let message = String(parts[3])
        let isLocal = source == nil ? sender == localPeerName : source == "self" || sender == localPeerName
        return (id, sender, message, isLocal)
    }

    private func upsertPeer(_ peerID: MCPeerID, state: NearbyPeer.State, nearby: Bool) {
        let id = peerID.displayName
        let current = peerDirectory[id]
        let peer = NearbyPeer(
            id: id,
            name: current?.name ?? id,
            state: state,
            lastSeen: Date()
        )
        peerDirectory[id] = peer
        rebuildPeerLists()

        if nearby {
            connectionStatus = "Found \(id)"
        }
    }

    private func markLost(_ peerID: MCPeerID) {
        let id = peerID.displayName
        let current = peerDirectory[id] ?? NearbyPeer(id: id, name: id, state: .lost, lastSeen: Date())
        peerDirectory[id] = NearbyPeer(id: current.id, name: current.name, state: .lost, lastSeen: Date())
        pendingInvites.remove(id)
        rebuildPeerLists()
    }

    private func markConnected(_ peerID: MCPeerID) {
        let id = peerID.displayName
        let current = peerDirectory[id] ?? NearbyPeer(id: id, name: id, state: .connected, lastSeen: Date())
        peerDirectory[id] = NearbyPeer(id: current.id, name: current.name, state: .connected, lastSeen: Date())
        pendingInvites.remove(id)
        rebuildPeerLists()
        connectionStatus = "Connected to \(connectedPeers.count) peer(s)"
    }

    private func rebuildPeerLists() {
        let peers = peerDirectory.values.sorted { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }
        nearbyPeers = peers.filter { $0.state != .lost && $0.state != .disconnected }
        connectedPeers = peers.filter { $0.state == .connected }
    }

    private func invite(_ peerID: MCPeerID) {
        let id = peerID.displayName
        guard !pendingInvites.contains(id) else { return }
        guard peerID != self.peerID else { return }
        pendingInvites.insert(id)
        upsertPeer(peerID, state: .invited, nearby: true)
        browser.invitePeer(peerID, to: session, withContext: nil, timeout: 10)
    }

    private func addMessage(_ message: String) {
        let timestamp = Self.timestampFormatter.string(from: Date())
        let entry = "[\(timestamp)] \(message)"
        recentMessages.insert(entry, at: 0)
        if recentMessages.count > 8 {
            recentMessages.removeLast(recentMessages.count - 8)
        }
        lastMessage = message
    }

    private func shouldAcceptChatMessage(id: String) -> Bool {
        if seenChatMessageIDs.contains(id) {
            return false
        }
        seenChatMessageIDs.insert(id)
        return true
    }

    private func appendChatMessage(id: String, sender: String, message: String, isLocal: Bool) {
        let prefix = isLocal ? "You" : sender
        let timestamp = Self.timestampFormatter.string(from: Date())
        let entry = "[\(timestamp)] \(prefix): \(message)"
        chatMessages.insert(entry, at: 0)
        if chatMessages.count > 24 {
            chatMessages.removeLast(chatMessages.count - 24)
        }
        lastMessage = message
    }

    private func publishGossip(_ message: String) {
        guard p2pGossipPublish(message: message) else {
            addMessage("Gossip publish failed")
            return
        }
        addMessage("Gossip published: \(message)")
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateStyle = .none
        formatter.timeStyle = .medium
        return formatter
    }()
}

extension PeerNetworkStore: MCNearbyServiceBrowserDelegate {
    func browser(_ browser: MCNearbyServiceBrowser, foundPeer peerID: MCPeerID, withDiscoveryInfo info: [String: String]?) {
        DispatchQueue.main.async {
            self.upsertPeer(peerID, state: .nearby, nearby: true)
            self.addMessage("Discovered \(peerID.displayName)")
            self.invite(peerID)
        }
    }

    func browser(_ browser: MCNearbyServiceBrowser, lostPeer peerID: MCPeerID) {
        DispatchQueue.main.async {
            self.markLost(peerID)
            self.addMessage("Lost \(peerID.displayName)")
        }
    }
}

extension PeerNetworkStore: MCNearbyServiceAdvertiserDelegate {
    func advertiser(_ advertiser: MCNearbyServiceAdvertiser, didReceiveInvitationFromPeer peerID: MCPeerID, withContext context: Data?, invitationHandler: @escaping (Bool, MCSession?) -> Void) {
        DispatchQueue.main.async {
            self.upsertPeer(peerID, state: .invited, nearby: true)
            invitationHandler(true, self.session)
            self.addMessage("Accepted invitation from \(peerID.displayName)")
        }
    }
}

extension PeerNetworkStore: MCSessionDelegate {
    func session(_ session: MCSession, peer peerID: MCPeerID, didChange state: MCSessionState) {
        DispatchQueue.main.async {
            switch state {
            case .connected:
                self.markConnected(peerID)
                self.addMessage("Connected with \(peerID.displayName)")
                self.broadcastCurrentState()
            case .connecting:
                self.upsertPeer(peerID, state: .invited, nearby: true)
                self.connectionStatus = "Connecting to \(peerID.displayName)"
            case .notConnected:
                self.peerDirectory[peerID.displayName] = NearbyPeer(
                    id: peerID.displayName,
                    name: peerID.displayName,
                    state: .disconnected,
                    lastSeen: Date()
                )
                self.pendingInvites.remove(peerID.displayName)
                self.rebuildPeerLists()
                self.addMessage("Disconnected from \(peerID.displayName)")
            @unknown default:
                self.addMessage("Peer state changed unexpectedly for \(peerID.displayName)")
            }
        }
    }

    func session(_ session: MCSession, didReceive data: Data, fromPeer peerID: MCPeerID) {
        let text = String(data: data, encoding: .utf8) ?? "\(data.count) bytes"
        DispatchQueue.main.async {
            self.addMessage("Received from \(peerID.displayName): \(text)")
            if let chat = self.parseChatPayload(text, source: peerID.displayName), self.shouldAcceptChatMessage(id: chat.id) {
                self.appendChatMessage(id: chat.id, sender: chat.sender, message: chat.message, isLocal: false)
            }
            self.processLibp2pHandshake(text, from: peerID)
            self.upsertPeer(peerID, state: .connected, nearby: true)
        }
    }

    func session(_ session: MCSession, didReceive stream: InputStream, withName streamName: String, fromPeer peerID: MCPeerID) {}

    func session(_ session: MCSession, didStartReceivingResourceWithName resourceName: String, fromPeer peerID: MCPeerID, with progress: Progress) {}

    func session(_ session: MCSession, didFinishReceivingResourceWithName resourceName: String, fromPeer peerID: MCPeerID, at localURL: URL?, withError error: Error?) {}
}

struct ContentView: View {
    @StateObject private var store = HybridNodeStore()
    @StateObject private var peers = PeerNetworkStore()
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass

    private var isCompact: Bool { horizontalSizeClass == .compact }

    var body: some View {
        NavigationSplitView {
            sidebar
        } detail: {
            detail
        }
        .navigationSplitViewStyle(.balanced)
        .background(backgroundGradient.ignoresSafeArea())
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button {
                    store.refreshState()
                } label: {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .disabled(store.isRefreshing)
            }
        }
    }

    private var sidebar: some View {
        List {
            Section {
                ForEach(DashboardSection.allCases) { section in
                    Button {
                        store.selection = section
                    } label: {
                        Label {
                            VStack(alignment: .leading, spacing: 2) {
                                Text(section.title)
                                Text(section.subtitle)
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                        } icon: {
                            Image(systemName: section.icon)
                                .symbolRenderingMode(.hierarchical)
                        }
                    }
                    .buttonStyle(.plain)
                    .background(
                        store.selection == section
                            ? Color.accentColor.opacity(0.12)
                            : Color.clear
                    )
                    .clipShape(RoundedRectangle(cornerRadius: 8))
                }
            } header: {
                VStack(alignment: .leading, spacing: 10) {
                    Label("kubo-rs", systemImage: "network")
                        .font(.headline)
                    Text("Desktop-style IPFS control surface")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .padding(.vertical, 8)
            }

            Section {
                SidebarStatusCard(snapshot: store.snapshot, isRefreshing: store.isRefreshing)
                    .listRowInsets(EdgeInsets())
                    .listRowBackground(Color.clear)
            } header: {
                Text("Live status")
            }
        }
        .listStyle(.sidebar)
        .navigationTitle("kubo-rs")
    }

    @ViewBuilder
    private var detail: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                heroCard
                switch store.selection {
                case .overview:
                    overviewContent
                case .repository:
                    repositoryContent
                case .network:
                    networkContent
                case .chat:
                    chatContent
                case .activity:
                    activityContent
                }
            }
            .padding(24)
        }
    }

    private var heroCard: some View {
        DashboardCard {
            let layout = isCompact
                ? AnyLayout(VStackLayout(alignment: .leading, spacing: 16))
                : AnyLayout(HStackLayout(alignment: .top, spacing: 20))

            layout {
                ZStack {
                    RoundedRectangle(cornerRadius: 22, style: .continuous)
                        .fill(
                            LinearGradient(
                                colors: [Color.accentColor.opacity(0.95), Color.cyan.opacity(0.85)],
                                startPoint: .topLeading,
                                endPoint: .bottomTrailing
                            )
                        )
                    Image(systemName: "cube.transparent")
                        .font(.system(size: 34, weight: .semibold))
                        .foregroundStyle(.white)
                }
                .frame(width: 92, height: 92)

                VStack(alignment: .leading, spacing: 10) {
                    HStack(spacing: 10) {
                        Text("Kubo Desktop")
                            .font(.largeTitle.weight(.semibold))
                        StatusBadge(text: store.snapshot.status, isRefreshing: store.isRefreshing)
                    }

                    Text("A desktop-style control surface for kubo-rs, inspired by the IPFS Desktop layout.")
                        .foregroundStyle(.secondary)

                    FlowLayout(spacing: 12) {
                        MetricPill(title: "Version", value: store.snapshot.version, symbol: "tag")
                        MetricPill(title: "Peer ID", value: shortPeerID(store.snapshot.peerID), symbol: "person.crop.circle")
                        MetricPill(title: "CID", value: shortCID(store.snapshot.cid), symbol: "link")
                    }
                }

                if !isCompact {
                    Spacer(minLength: 0)
                }
            }
        }
    }

    private var overviewContent: some View {
        VStack(alignment: .leading, spacing: 20) {
            LazyVGrid(columns: adaptiveColumns, spacing: 16) {
                MetricCard(title: "IPFS Peer", value: shortPeerID(store.snapshot.peerID), symbol: "person.2.circle", subtitle: "HybridNode IPFS identity")
                MetricCard(title: "P2P Peer", value: shortPeerID(hybridP2pPeerId()), symbol: "network", subtitle: "HybridNode libp2p identity")
                MetricCard(title: "Pins", value: "\(store.pins.count)", symbol: "pin", subtitle: "Locally pinned CIDs")
                MetricCard(title: "Status", value: store.snapshot.status, symbol: "power.circle", subtitle: "HybridNode online state")
            }

            DashboardCard(title: "IPFS Add") {
                VStack(alignment: .leading, spacing: 12) {
                    TextField("Type text to add to IPFS…", text: $store.addDraft, axis: .vertical)
                        .textFieldStyle(.roundedBorder)
                        .lineLimit(2...4)
                    HStack(spacing: 12) {
                        Button {
                            store.ipfsAdd()
                        } label: {
                            Label("Add to IPFS", systemImage: "plus.circle")
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(store.addDraft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || store.isRefreshing)

                        if !store.lastAddedCID.isEmpty {
                            Text("CID: \(shortCID(store.lastAddedCID))")
                                .font(.system(.body, design: .monospaced))
                                .textSelection(.enabled)
                        }

                        Spacer()
                    }
                }
            }

            DashboardCard(title: "IPFS Cat") {
                VStack(alignment: .leading, spacing: 12) {
                    TextField("Enter CID to fetch…", text: $store.catDraft)
                        .textFieldStyle(.roundedBorder)
                    HStack(spacing: 12) {
                        Button {
                            store.ipfsCat()
                        } label: {
                            Label("Fetch", systemImage: "arrow.down.circle")
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(store.catDraft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                        if !store.lastAddedCID.isEmpty {
                            Button {
                                store.catDraft = store.lastAddedCID
                                store.ipfsCat()
                            } label: {
                                Label("Cat last CID", systemImage: "doc.text")
                            }
                            .buttonStyle(.bordered)
                        }

                        Spacer()
                    }
                    if !store.catResult.isEmpty {
                        Text(store.catResult)
                            .font(.system(.body, design: .monospaced))
                            .textSelection(.enabled)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(.top, 4)
                    }
                }
            }

            DashboardCard(title: "Pin Management") {
                VStack(alignment: .leading, spacing: 12) {
                    HStack(spacing: 12) {
                        Button {
                            store.ipfsPin(cid: store.lastAddedCID)
                        } label: {
                            Label("Pin last CID", systemImage: "pin")
                        }
                        .buttonStyle(.bordered)
                        .disabled(store.lastAddedCID.isEmpty)

                        Button {
                            store.ipfsUnpin(cid: store.lastAddedCID)
                        } label: {
                            Label("Unpin last CID", systemImage: "pin.slash")
                        }
                        .buttonStyle(.bordered)
                        .disabled(store.lastAddedCID.isEmpty)

                        Spacer()
                    }

                    if store.pins.isEmpty {
                        Text("No pinned CIDs yet.")
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(store.pins, id: \.self) { pin in
                            Text(pin)
                                .font(.system(.body, design: .monospaced))
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }
            }
        }
    }

    private var repositoryContent: some View {
        VStack(alignment: .leading, spacing: 20) {
            DashboardCard(title: "Git Init") {
                VStack(alignment: .leading, spacing: 12) {
                    TextField("Local path for new repo…", text: $store.gitPath)
                        .textFieldStyle(.roundedBorder)
                    HStack(spacing: 12) {
                        Button {
                            store.gitInitRepo()
                        } label: {
                            Label("Init repo", systemImage: "folder.badge.plus")
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(store.gitPath.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                        Button {
                            store.refreshGit()
                        } label: {
                            Label("Refresh", systemImage: "arrow.clockwise")
                        }
                        .buttonStyle(.bordered)
                        .disabled(store.gitPath.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                        Spacer()
                    }
                }
            }

            DashboardCard(title: "Git Clone") {
                VStack(alignment: .leading, spacing: 12) {
                    TextField("Remote URL…", text: $store.cloneURL)
                        .textFieldStyle(.roundedBorder)
                    TextField("Local path…", text: $store.clonePath)
                        .textFieldStyle(.roundedBorder)
                    Button {
                        store.gitCloneRepo()
                    } label: {
                        Label("Clone", systemImage: "arrow.down.doc")
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(store.cloneURL.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || store.clonePath.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                }
            }

            DashboardCard(title: "Repo state") {
                VStack(alignment: .leading, spacing: 12) {
                    if store.gitHeadResult.isEmpty {
                        Text("No repo loaded. Init or clone a repository above.")
                            .foregroundStyle(.secondary)
                    } else {
                        Text("HEAD: \(store.gitHeadResult)")
                        if !store.gitBranchesResult.isEmpty {
                            Text("Branches: \(store.gitBranchesResult.joined(separator: ", "))")
                        }
                        if !store.gitRemotesResult.isEmpty {
                            Text("Remotes: \(store.gitRemotesResult.joined(separator: ", "))")
                        }
                        if !store.gitStatusResult.isEmpty {
                            Text("Status:\n\(store.gitStatusResult)")
                        }
                    }
                }
                .font(.system(.body, design: .monospaced))
                .textSelection(.enabled)
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private var networkContent: some View {
        VStack(alignment: .leading, spacing: 20) {
            LazyVGrid(columns: adaptiveColumns, spacing: 16) {
                MetricCard(title: "Local peer", value: peers.localPeerName, symbol: "person.crop.circle", subtitle: "Unique instance name advertised on the LAN")
                MetricCard(title: "Discovery", value: peers.connectionStatus, symbol: "antenna.radiowaves.left.and.right", subtitle: "Browsing and advertising via MultipeerConnectivity")
                MetricCard(title: "libp2p peer", value: peers.libp2pPeerID, symbol: "network", subtitle: "Rust host with relay and hole-punch support enabled")
                MetricCard(title: "Gossip topic", value: peers.gossipTopic, symbol: "bubble.left.and.bubble.right", subtitle: "Shared pubsub topic joined by the libp2p host")
                MetricCard(title: "Connected peers", value: "\(peers.connectedPeers.count)", symbol: "person.2.circle", subtitle: "Peers with an active session")
                MetricCard(title: "Last message", value: peers.lastMessage, symbol: "message", subtitle: "Latest p2p status or broadcast")
            }

            DashboardCard(title: "libp2p transport") {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Listening addresses")
                        .font(.headline)
                    if peers.libp2pAddrs.isEmpty {
                        Text("No libp2p addresses yet.")
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(peers.libp2pAddrs, id: \.self) { addr in
                            Text(addr)
                                .font(.system(.body, design: .monospaced))
                                .textSelection(.enabled)
                        }
                    }

                    Text("Protocols")
                        .font(.headline)
                        .padding(.top, 4)
                    if peers.libp2pProtocols.isEmpty {
                        Text("No protocols reported yet.")
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(peers.libp2pProtocols, id: \.self) { proto in
                            Text(proto)
                                .font(.system(.body, design: .monospaced))
                        }
                    }
                }
            }

            DashboardCard(title: "Nearby peers") {
                VStack(alignment: .leading, spacing: 10) {
                    if peers.nearbyPeers.isEmpty {
                        Text("No peers discovered yet. Open the app on a Mac and an iPad on the same network.")
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(peers.nearbyPeers) { peer in
                            HStack(spacing: 12) {
                                VStack(alignment: .leading, spacing: 4) {
                                    Text(peer.name)
                                        .font(.headline)
                                    Text(peer.lastSeen.formatted(date: .omitted, time: .standard))
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                }

                                Spacer()

                                Text(peer.statusText)
                                    .font(.caption.weight(.semibold))
                                    .padding(.horizontal, 10)
                                    .padding(.vertical, 6)
                                    .background(Capsule(style: .continuous).fill(Color.primary.opacity(0.06)))
                            }
                            if peer.id != peers.nearbyPeers.last?.id {
                                Divider()
                            }
                        }
                    }
                }
            }

            DashboardCard(title: "Peer exchange") {
                VStack(alignment: .leading, spacing: 12) {
                    HStack(spacing: 12) {
                        Button {
                            peers.broadcastCurrentState()
                        } label: {
                            Label("Broadcast identity", systemImage: "dot.radiowaves.left.and.right")
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(peers.connectedPeers.isEmpty)

                        Button {
                            peers.restartDiscovery()
                        } label: {
                            Label("Restart discovery", systemImage: "arrow.clockwise")
                        }
                        .buttonStyle(.bordered)

                        Spacer()
                    }

                    if peers.gossipMessages.isEmpty {
                        Text("No gossip messages yet.")
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(Array(peers.gossipMessages.enumerated()), id: \.offset) { _, entry in
                            Text(entry)
                                .font(.system(.body, design: .monospaced))
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }
            }

            DashboardCard(title: "Recent activity") {
                VStack(alignment: .leading, spacing: 12) {
                    if peers.recentMessages.isEmpty {
                        Text("No peer messages yet.")
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(Array(peers.recentMessages.enumerated()), id: \.offset) { _, entry in
                            Text(entry)
                                .font(.system(.body, design: .monospaced))
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }
            }
        }
    }

    private var chatContent: some View {
        VStack(alignment: .leading, spacing: 20) {
            LazyVGrid(columns: adaptiveColumns, spacing: 16) {
                MetricCard(title: "Chat topic", value: peers.gossipTopic, symbol: "bubble.left.and.bubble.right", subtitle: "Messages published into the shared pubsub room")
                MetricCard(title: "Participants", value: "\(peers.connectedPeers.count)", symbol: "person.2.circle", subtitle: "Connected peers that can receive chat broadcasts")
                MetricCard(title: "Last message", value: peers.lastMessage, symbol: "message", subtitle: "Most recent chat or network event")
            }

            DashboardCard(title: "Nostr Keys") {
                VStack(alignment: .leading, spacing: 12) {
                    HStack(spacing: 12) {
                        Button {
                            store.generateNostrKey()
                        } label: {
                            Label("Generate key", systemImage: "key")
                        }
                        .buttonStyle(.borderedProminent)

                        Spacer()
                    }

                    if !store.nostrPublicKey.isEmpty {
                        Text("Public: \(store.nostrPublicKey)")
                            .font(.system(.body, design: .monospaced))
                            .textSelection(.enabled)
                        Text("Secret: \(store.shortKey(store.nostrSecretKey))")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }

            DashboardCard(title: "Nostr Event") {
                VStack(alignment: .leading, spacing: 12) {
                    TextField("Event content…", text: $store.nostrContent, axis: .vertical)
                        .textFieldStyle(.roundedBorder)
                        .lineLimit(2...4)
                    HStack(spacing: 12) {
                        Button {
                            store.signEvent()
                        } label: {
                            Label("Sign kind-1", systemImage: "signature")
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(store.nostrContent.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || store.nostrSecretKey.isEmpty)

                        Spacer()
                    }
                    if !store.nostrEventJson.isEmpty {
                        Text(store.nostrEventJson)
                            .font(.system(.caption, design: .monospaced))
                            .textSelection(.enabled)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
            }

            DashboardCard(title: "Relay") {
                VStack(alignment: .leading, spacing: 12) {
                    TextField("Relay URL", text: $store.relayURL)
                        .textFieldStyle(.roundedBorder)
                    HStack(spacing: 12) {
                        Button {
                            store.connectRelay()
                        } label: {
                            Label("Connect", systemImage: "network")
                        }
                        .buttonStyle(.borderedProminent)

                        Button {
                            store.publishToRelay()
                        } label: {
                            Label("Publish", systemImage: "paperplane")
                        }
                        .buttonStyle(.bordered)
                        .disabled(store.relayHandle == 0 || store.nostrEventJson.isEmpty)

                        Button {
                            store.drainRelay()
                        } label: {
                            Label("Drain", systemImage: "arrow.down")
                        }
                        .buttonStyle(.bordered)
                        .disabled(store.relayHandle == 0)

                        Spacer()
                    }
                    if !store.drainResult.isEmpty {
                        Text(store.drainResult)
                            .font(.system(.caption, design: .monospaced))
                            .textSelection(.enabled)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
            }

            DashboardCard(title: "Hybrid Gossip") {
                VStack(alignment: .leading, spacing: 12) {
                    TextField("Topic", text: $store.gossipTopic)
                        .textFieldStyle(.roundedBorder)
                    TextField("Message JSON…", text: $store.gossipMessage, axis: .vertical)
                        .textFieldStyle(.roundedBorder)
                        .lineLimit(2...4)
                    HStack(spacing: 12) {
                        Button {
                            store.publishGossip()
                        } label: {
                            Label("Publish", systemImage: "dot.radiowaves.left.and.right")
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(store.gossipMessage.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                        Button {
                            store.drainGossip()
                        } label: {
                            Label("Drain", systemImage: "arrow.down")
                        }
                        .buttonStyle(.bordered)

                        Spacer()
                    }
                }
            }

            DashboardCard(title: "Chat transcript") {
                VStack(alignment: .leading, spacing: 10) {
                    if peers.chatMessages.isEmpty {
                        Text("No chat messages yet.")
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(Array(peers.chatMessages.enumerated()), id: \.offset) { _, entry in
                            Text(entry)
                                .font(.system(.body, design: .monospaced))
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }
            }

            DashboardCard(title: "Gossip feed") {
                VStack(alignment: .leading, spacing: 10) {
                    if peers.gossipMessages.isEmpty {
                        Text("No gossip messages yet.")
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(Array(peers.gossipMessages.enumerated()), id: \.offset) { _, entry in
                            Text(entry)
                                .font(.system(.body, design: .monospaced))
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }
            }
        }
    }

    private var activityContent: some View {
        DashboardCard(title: "Recent activity") {
            VStack(alignment: .leading, spacing: 10) {
                if store.activity.isEmpty {
                    Text("No activity yet.")
                        .foregroundStyle(.secondary)
                } else {
                    ForEach(Array(store.activity.enumerated()), id: \.offset) { _, entry in
                        Text(entry)
                            .font(.system(.body, design: .monospaced))
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(.vertical, 2)
                    }
                }
            }
        }
    }

    private var backgroundGradient: LinearGradient {
        LinearGradient(
            colors: [
                Color(.systemBackground),
                Color.accentColor.opacity(0.07),
                Color.cyan.opacity(0.05)
            ],
            startPoint: .topLeading,
            endPoint: .bottomTrailing
        )
    }

    private var adaptiveColumns: [GridItem] {
        [GridItem(.adaptive(minimum: 220, maximum: 360), spacing: 16, alignment: .top)]
    }

    private func infoRow(number: String, title: String, detail: String) -> some View {
        HStack(alignment: .top, spacing: 12) {
            Circle()
                .fill(Color.accentColor.opacity(0.18))
                .frame(width: 30, height: 30)
                .overlay(
                    Text(number)
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(Color.accentColor)
                )

            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(.headline)
                Text(detail)
                    .foregroundStyle(.secondary)
            }

            Spacer(minLength: 0)
        }
    }

    private func shortPeerID(_ peerID: String) -> String {
        guard peerID.count > 18 else { return peerID }
        return "\(peerID.prefix(8))…\(peerID.suffix(6))"
    }

    private func shortCID(_ cid: String) -> String {
        guard cid.count > 18 else { return cid }
        return "\(cid.prefix(10))…\(cid.suffix(6))"
    }
}

private struct FlowLayout: Layout {
    var spacing: CGFloat = 8

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let result = FlowResult(in: proposal.width ?? 0, subviews: subviews, spacing: spacing)
        return result.size
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let result = FlowResult(in: bounds.width, subviews: subviews, spacing: spacing)
        for (index, subview) in subviews.enumerated() {
            subview.place(at: CGPoint(x: bounds.minX + result.positions[index].x,
                                      y: bounds.minY + result.positions[index].y),
                         proposal: .unspecified)
        }
    }

    private struct FlowResult {
        var size: CGSize = .zero
        var positions: [CGPoint] = []

        init(in maxWidth: CGFloat, subviews: Subviews, spacing: CGFloat) {
            var x: CGFloat = 0
            var y: CGFloat = 0
            var lineHeight: CGFloat = 0

            for subview in subviews {
                let size = subview.sizeThatFits(.unspecified)
                if x + size.width > maxWidth && x > 0 {
                    x = 0
                    y += lineHeight + spacing
                    lineHeight = 0
                }
                positions.append(CGPoint(x: x, y: y))
                lineHeight = max(lineHeight, size.height)
                x += size.width + spacing
            }

            self.size = CGSize(width: maxWidth, height: y + lineHeight)
        }
    }
}

private struct DashboardCard<Content: View>: View {
    var title: String?
    @ViewBuilder var content: Content

    init(title: String? = nil, @ViewBuilder content: () -> Content) {
        self.title = title
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            if let title {
                Text(title)
                    .font(.title3.weight(.semibold))
            }

            content
        }
        .padding(20)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 22, style: .continuous)
                .fill(.thinMaterial)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 22, style: .continuous)
                .strokeBorder(Color.primary.opacity(0.08))
        )
    }
}

private struct MetricCard: View {
    let title: String
    let value: String
    let symbol: String
    let subtitle: String

    var body: some View {
        DashboardCard {
            VStack(alignment: .leading, spacing: 14) {
                HStack(alignment: .top) {
                    Label(title, systemImage: symbol)
                        .font(.headline)
                        .labelStyle(.titleAndIcon)

                    Spacer(minLength: 0)
                }

                Text(value)
                    .font(.system(.body, design: .monospaced))
                    .lineLimit(4)
                    .textSelection(.enabled)

                Text(subtitle)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
    }
}

private struct MetricPill: View {
    let title: String
    let value: String
    let symbol: String

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: symbol)
                .foregroundStyle(.secondary)
            VStack(alignment: .leading, spacing: 1) {
                Text(title)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text(value)
                    .font(.subheadline.weight(.medium))
                    .lineLimit(1)
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .background(
            Capsule(style: .continuous)
                .fill(Color.primary.opacity(0.05))
        )
    }
}

private struct StatusBadge: View {
    let text: String
    let isRefreshing: Bool

    var body: some View {
        HStack(spacing: 6) {
            Circle()
                .fill(isRefreshing ? Color.orange : Color.green)
                .frame(width: 8, height: 8)

            Text(isRefreshing ? "Refreshing" : text)
                .font(.caption.weight(.semibold))
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(
            Capsule(style: .continuous)
                .fill(Color.primary.opacity(0.06))
        )
    }
}

private struct SidebarStatusCard: View {
    let snapshot: KuboSnapshot
    let isRefreshing: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Live status")
                    .font(.headline)
                Spacer()
                StatusBadge(text: snapshot.status, isRefreshing: isRefreshing)
            }

            VStack(alignment: .leading, spacing: 8) {
                statusLine(label: "Version", value: snapshot.version)
                statusLine(label: "Peer", value: snapshot.peerID)
                statusLine(label: "CID", value: snapshot.cid)
            }
            .font(.caption)
        }
        .padding(16)
        .background(
            RoundedRectangle(cornerRadius: 18, style: .continuous)
                .fill(Color.primary.opacity(0.04))
        )
    }

    private func statusLine(label: String, value: String) -> some View {
        HStack(alignment: .top, spacing: 8) {
            Text(label)
                .foregroundStyle(.secondary)
                .frame(width: 52, alignment: .leading)
            Text(value)
                .lineLimit(2)
                .truncationMode(.middle)
            Spacer(minLength: 0)
        }
    }
}

#Preview {
    ContentView()
}
