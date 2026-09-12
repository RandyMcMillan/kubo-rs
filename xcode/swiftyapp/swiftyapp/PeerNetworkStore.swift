//
//  PeerNetworkStore.swift
//  swiftyapp
//

import Foundation
import MultipeerConnectivity
import SwiftUI
import RustyLib

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
