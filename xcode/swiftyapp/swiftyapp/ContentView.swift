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
    case activity

    var id: String { rawValue }

    var title: String {
        switch self {
        case .overview: return "Overview"
        case .repository: return "Repository"
        case .network: return "Network"
        case .activity: return "Activity"
        }
    }

    var subtitle: String {
        switch self {
        case .overview: return "Status at a glance"
        case .repository: return "Local repo snapshot"
        case .network: return "Peer and CID details"
        case .activity: return "Recent actions"
        }
    }

    var icon: String {
        switch self {
        case .overview: return "square.grid.2x2"
        case .repository: return "externaldrive.connected.to.line.below"
        case .network: return "point.3.connected.trianglepath.dotted"
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
    var rawSummary: String = "Tap Refresh Snapshot to build a temporary repo and capture a live Kubo result."
    var updatedAt: Date?

    static let placeholder = KuboSnapshot()
}

@MainActor
final class DashboardStore: ObservableObject {
    @Published var selection: DashboardSection = .overview
    @Published var snapshot: KuboSnapshot = .placeholder
    @Published var activity: [String] = []
    @Published var isRefreshing = false

    func refresh() {
        guard !isRefreshing else { return }
        isRefreshing = true
        appendActivity("Refreshing live Kubo snapshot…")

        Task {
            let result = Self.captureSnapshot()
            await MainActor.run {
                self.snapshot = result.snapshot
                self.isRefreshing = false
                self.appendActivity(result.logMessage)
            }
        }
    }

    private static func captureSnapshot() -> (snapshot: KuboSnapshot, logMessage: String) {
        let summary = rustHello()
        let sum = rustAdd(a: 10, b: 32)
        let parsed = parseSummary(summary)

        let snapshot = KuboSnapshot(
            version: parsed.version,
            peerID: parsed.peerID,
            cid: parsed.cid,
            roundTrip: parsed.roundTrip,
            mathResult: "10 + 32 = \(sum)",
            status: parsed.isHealthy ? "Ready" : "Needs attention",
            rawSummary: summary,
            updatedAt: Date()
        )

        let message = parsed.isHealthy
            ? "Snapshot refreshed with CID \(parsed.cid)"
            : "Snapshot returned a fallback message"
        return (snapshot, message)
    }

    private static func parseSummary(_ summary: String) -> (version: String, peerID: String, cid: String, roundTrip: String, isHealthy: Bool) {
        let parts = summary
            .split(separator: "|")
            .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }

        var version = "—"
        var peerID = "—"
        var cid = "—"
        var roundTrip = "—"

        for part in parts {
            if part.hasPrefix("kubo-rs ") {
                version = String(part.dropFirst("kubo-rs ".count))
            } else if part.hasPrefix("peer ") {
                peerID = String(part.dropFirst("peer ".count))
            } else if part.hasPrefix("cid ") {
                cid = String(part.dropFirst("cid ".count))
            } else if part.hasPrefix("round-trip ") {
                roundTrip = String(part.dropFirst("round-trip ".count))
            }
        }

        let isHealthy = summary.contains("kubo-rs") && summary.contains("round-trip")
        return (version, peerID, cid, roundTrip, isHealthy)
    }

    private func appendActivity(_ message: String) {
        let timestamp = Self.timestampFormatter.string(from: Date())
        activity.insert("[\(timestamp)] \(message)", at: 0)
        if activity.count > 8 {
            activity.removeLast(activity.count - 8)
        }
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateStyle = .none
        formatter.timeStyle = .medium
        return formatter
    }()
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
    }

    deinit {
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

        if peerID.isEmpty {
            libp2pPeerID = "Unavailable"
            libp2pAddrs = []
            libp2pProtocols = []
            addMessage("libp2p host failed to start")
        } else {
            libp2pPeerID = peerID
            libp2pAddrs = addrs
            libp2pProtocols = protocols
            addMessage("Started libp2p host \(libp2pPeerID)")
        }
    }

    private func libp2pHandshakeMessage() -> String {
        let addresses = libp2pAddrs.joined(separator: "\n")
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
            if p2pConnect(addr: addr) {
                addMessage("Dialed \(remoteName) at \(addr)")
            } else {
                addMessage("Dial failed for \(remoteName) at \(addr)")
            }
        }
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
            self.processLibp2pHandshake(text, from: peerID)
            self.upsertPeer(peerID, state: .connected, nearby: true)
        }
    }

    func session(_ session: MCSession, didReceive stream: InputStream, withName streamName: String, fromPeer peerID: MCPeerID) {}

    func session(_ session: MCSession, didStartReceivingResourceWithName resourceName: String, fromPeer peerID: MCPeerID, with progress: Progress) {}

    func session(_ session: MCSession, didFinishReceivingResourceWithName resourceName: String, fromPeer peerID: MCPeerID, at localURL: URL?, withError error: Error?) {}
}

struct ContentView: View {
    @StateObject private var store = DashboardStore()
    @StateObject private var peers = PeerNetworkStore()

    var body: some View {
        NavigationSplitView {
            sidebar
        } detail: {
            detail
        }
        .navigationSplitViewStyle(.balanced)
        .frame(minWidth: 1120, minHeight: 760)
        .background(backgroundGradient.ignoresSafeArea())
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button {
                    store.refresh()
                } label: {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .disabled(store.isRefreshing)
            }
        }
        .task {
            store.refresh()
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
                case .activity:
                    activityContent
                }
            }
            .padding(24)
        }
    }

    private var heroCard: some View {
        DashboardCard {
            HStack(alignment: .top, spacing: 20) {
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

                    HStack(spacing: 12) {
                        MetricPill(title: "Version", value: store.snapshot.version, symbol: "tag")
                        MetricPill(title: "Peer ID", value: shortPeerID(store.snapshot.peerID), symbol: "person.crop.circle")
                        MetricPill(title: "CID", value: shortCID(store.snapshot.cid), symbol: "link")
                    }
                }

                Spacer(minLength: 0)
            }
        }
    }

    private var overviewContent: some View {
        VStack(alignment: .leading, spacing: 20) {
            LazyVGrid(columns: adaptiveColumns, spacing: 16) {
                MetricCard(title: "Peer ID", value: store.snapshot.peerID, symbol: "person.2.circle", subtitle: "Node identity from the live repo snapshot")
                MetricCard(title: "CID", value: store.snapshot.cid, symbol: "doc.richtext", subtitle: "UnixFS round-trip content address")
                MetricCard(title: "Round-trip", value: store.snapshot.roundTrip, symbol: "arrow.2.circlepath", subtitle: "Bytes written to and read back from Kubo")
                MetricCard(title: "Arithmetic", value: store.snapshot.mathResult, symbol: "plus.forwardslash.minus", subtitle: "A tiny sanity check that the UI is responsive")
            }

            DashboardCard(title: "Actions") {
                HStack(spacing: 12) {
                    Button {
                        store.refresh()
                    } label: {
                        Label("Refresh snapshot", systemImage: "arrow.clockwise")
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(store.isRefreshing)

                    Button {
                        store.refresh()
                    } label: {
                        Label("Re-run demo", systemImage: "play.circle")
                    }
                    .buttonStyle(.bordered)
                    .disabled(store.isRefreshing)

                    Spacer()

                    if let updatedAt = store.snapshot.updatedAt {
                        Text("Updated \(updatedAt.formatted(date: .omitted, time: .standard))")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }

            DashboardCard(title: "Raw snapshot") {
                Text(store.snapshot.rawSummary)
                    .font(.system(.body, design: .monospaced))
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private var repositoryContent: some View {
        VStack(alignment: .leading, spacing: 20) {
            DashboardCard(title: "Repository lifecycle") {
                VStack(alignment: .leading, spacing: 12) {
                    infoRow(number: "1", title: "Create a temporary repo", detail: "The Rust bridge initializes Kubo in a fresh directory before each demo.")
                    infoRow(number: "2", title: "Start an offline node", detail: "The sample keeps the node local so the GUI stays safe and deterministic.")
                    infoRow(number: "3", title: "Write and read bytes", detail: "A live UnixFS add/cat round-trip proves the integration path works.")
                }
            }

            DashboardCard(title: "Demo output") {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Version \(store.snapshot.version)")
                    Text("Peer ID \(store.snapshot.peerID)")
                    Text("CID \(store.snapshot.cid)")
                    Text("Round-trip payload \(store.snapshot.roundTrip)")
                }
                .font(.system(.body, design: .monospaced))
            }
        }
    }

    private var networkContent: some View {
        VStack(alignment: .leading, spacing: 20) {
            LazyVGrid(columns: adaptiveColumns, spacing: 16) {
                MetricCard(title: "Local peer", value: peers.localPeerName, symbol: "person.crop.circle", subtitle: "Unique instance name advertised on the LAN")
                MetricCard(title: "Discovery", value: peers.connectionStatus, symbol: "antenna.radiowaves.left.and.right", subtitle: "Browsing and advertising via MultipeerConnectivity")
                MetricCard(title: "libp2p peer", value: peers.libp2pPeerID, symbol: "network", subtitle: "Rust host with relay and hole-punch support enabled")
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
