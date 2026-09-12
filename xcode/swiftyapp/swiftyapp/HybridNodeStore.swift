//
//  HybridNodeStore.swift
//  swiftyapp
//

import Foundation
import SwiftUI
import RustyLib

enum NetworkTab: String, CaseIterable {
    case ipfs = "IPFS"
    case nostr = "NOSTR"
    case p2p = "P2P"
}

@MainActor
final class HybridNodeStore: ObservableObject {
    @Published var selection: DashboardSection = .repository
    @Published var networkTab: NetworkTab = .ipfs
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
    @Published var cloneURL: String = "https://github.com/RandyMcMillan/kubo-rs.git"
    @Published var clonePath: String = ""
    @Published var cloneResult: String = ""
    @Published var forceClone: Bool = false
    @Published var fetchResult: String = ""
    @Published var repoTree: [FileNode] = []
    @Published var selectedFilePath: String = ""
    @Published var selectedFileContent: String = ""
    @Published var blameLines: [BlameLine] = []
    @Published var blameError: String = ""
    @Published var commitHistory: [CommitInfo] = []
    @Published var tagList: [String] = []
    @Published var readmeContent: String = ""
    @Published var repos: [RepoEntry] = []

    // Nostr
    @Published var nostrSecretKey: String = ""
    @Published var nostrPublicKey: String = ""
    @Published var nostrEventJson: String = ""
    @Published var relayURL: String = "wss://relay.damus.io"
    @Published var relayHandle: UInt64 = 0
    @Published var relays: [RelayEntry] = []
    @Published var drainResult: String = ""
    @Published var nostrContent: String = ""
    @Published var gossipTopic: String = "kubo-hybrid"
    @Published var gossipMessage: String = ""

    // Typed Messages (Phase 15)
    @Published var typedMessages: [HybridMessage] = []
    @Published var typedCategoryFilter: MessageCategory? = nil
    @Published var typedBroadcastCategory: MessageCategory = .file
    @Published var typedContent: String = ""
    @Published var typedCID: String = ""
    @Published var typedFilename: String = ""
    @Published var typedRepoRef: String = ""
    @Published var typedTitle: String = ""
    @Published var typedBody: String = ""
    @Published var typedDiff: String = ""

    // NIP-94 (Phase 16)
    @Published var showFileImporter: Bool = false
    @Published var publishFileResult: String = ""
    @Published var nip94ResolveInput: String = ""
    @Published var nip94ResolveResult: String = ""

    // NIP-34 (Phase 17)
    @Published var nip34RepoDescription: String = ""
    @Published var nip34RepoCloneURLs: String = ""
    @Published var nip34PatchOldHash: String = ""
    @Published var nip34PatchNewHash: String = ""
    @Published var nip34IssueTitle: String = ""
    @Published var nip34IssueBody: String = ""
    @Published var nip34PublishResult: String = ""

    // Inbox (Phase 18)
    @Published var inboxMessages: [HybridMessage] = []
    @Published var inboxFilter: MessageCategory? = nil
    @Published var unreadCount: Int = 0

    // Background Polling (Phase 20)
    @Published var isPolling: Bool = false
    private var pollingTask: Task<Void, Never>?

    // Network / DHT
    @Published var dhtPeerID: String = ""
    @Published var dhtPeerAddrs: [String] = []
    @Published var dhtCID: String = ""
    @Published var dhtProviders: [String] = []
    @Published var p2pConnectAddr: String = ""

    // Name / Block
    @Published var namePublishCID: String = ""
    @Published var namePublishLifetime: String = "86400"
    @Published var namePublishResult: String = ""
    @Published var nameResolveName: String = ""
    @Published var nameResolveResult: String = ""
    @Published var blockData: String = ""
    @Published var blockCID: String = ""
    @Published var blockResult: String = ""
    @Published var blockStatSize: UInt64 = 0

    // Git extras
    @Published var commitHash: String = ""
    @Published var commitMessageResult: String = ""
    @Published var diffOldHash: String = ""
    @Published var diffNewHash: String = ""
    @Published var diffResult: String = ""

    init() {
        let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first?
            .appendingPathComponent("kubo-rs-clone").path ?? "/tmp/kubo-rs-clone"
        clonePath = docs
        scanForRepos()
        if FileManager.default.fileExists(atPath: (docs as NSString).appendingPathComponent(".git")) {
            gitPath = docs
            refreshGit()
        } else if let first = repos.first {
            gitPath = first.path
            refreshGit()
        }
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
        if ok {
            appendActivity("Git init: \(path)")
            refreshGit()
        } else {
            let err = goLastError()
            appendActivity("Git init failed: \(err)")
        }
    }

    func viewFile(path: String) {
        selectedFilePath = path
        blameLines = []
        blameError = ""
        if let data = FileManager.default.contents(atPath: path),
           let text = String(data: data, encoding: .utf8) {
            selectedFileContent = text
        } else {
            selectedFileContent = "<binary or unreadable file>"
        }
    }

    func viewBlame() {
        let repo = gitPath.trimmingCharacters(in: .whitespacesAndNewlines)
        let file = selectedFilePath.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !repo.isEmpty, !file.isEmpty else { return }
        let relPath = file.replacingOccurrences(of: repo + "/", with: "")
        let json = gitBlame(path: repo, filePath: relPath)
        guard !json.isEmpty else {
            blameError = goLastError()
            return
        }
        do {
            let data = json.data(using: .utf8) ?? Data()
            let decoded = try JSONDecoder().decode(BlameResult.self, from: data)
            blameLines = decoded.lines
            blameError = ""
        } catch {
            blameError = "Parse error: \(error)"
        }
    }

    func fetchAll() {
        let path = gitPath.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !path.isEmpty else { return }
        let ok = gitFetchAll(path: path)
        if ok {
            fetchResult = "Fetched all remotes"
            appendActivity("Fetched all remotes")
            refreshGit()
        } else {
            let err = goLastError()
            fetchResult = "Fetch failed: \(err)"
            appendActivity("Fetch failed: \(err)")
        }
    }

    func gitCloneRepo() {
        let url = cloneURL.trimmingCharacters(in: .whitespacesAndNewlines)
        let path = clonePath.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !url.isEmpty, !path.isEmpty else { return }
        let fm = FileManager.default
        if forceClone && fm.fileExists(atPath: path) {
            try? fm.removeItem(atPath: path)
            appendActivity("Removed existing dir: \(path)")
        }
        let ok = gitClone(url: url, path: path, bare: false)
        if ok {
            cloneResult = "Cloned to \(path)"
            appendActivity("Git clone → \(path)")
            gitPath = path
            refreshGit()
        } else {
            let err = goLastError()
            cloneResult = "Error: \(err)"
            appendActivity("Git clone failed: \(err)")
        }
    }

    func refreshGit() {
        let path = gitPath.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !path.isEmpty else { return }
        gitHeadResult = gitHead(path: path)
        gitBranchesResult = gitBranches(path: path)
        gitRemotesResult = gitRemotes(path: path)
        gitStatusResult = gitStatus(path: path)
        repoTree = buildFileTree(path: path)
        loadCommitHistory()
        loadTags()
        loadReadme()
    }

    func loadCommitHistory() {
        let path = gitPath.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !path.isEmpty else { return }
        let json = gitLog(path: path, maxCount: 50)
        guard !json.isEmpty else { return }
        do {
            let data = json.data(using: .utf8) ?? Data()
            commitHistory = try JSONDecoder().decode([CommitInfo].self, from: data)
        } catch {
            commitHistory = []
        }
    }

    func loadTags() {
        let path = gitPath.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !path.isEmpty else { return }
        let json = gitTags(path: path)
        guard !json.isEmpty else { return }
        do {
            let data = json.data(using: .utf8) ?? Data()
            tagList = try JSONDecoder().decode([String].self, from: data)
        } catch {
            tagList = []
        }
    }

    func scanForRepos() {
        let fm = FileManager.default
        guard let docs = fm.urls(for: .documentDirectory, in: .userDomainMask).first else { return }
        let root = docs.path
        guard let entries = try? fm.contentsOfDirectory(atPath: root) else { return }
        var found: [RepoEntry] = []
        for name in entries {
            let path = (root as NSString).appendingPathComponent(name)
            let gitPath = (path as NSString).appendingPathComponent(".git")
            guard fm.fileExists(atPath: gitPath) else { continue }
            let head = gitHead(path: path)
            let branches = gitBranches(path: path)
            let branch = branches.first ?? "main"
            found.append(RepoEntry(path: path, name: name, head: head, branch: branch))
        }
        repos = found
    }

    func selectRepo(_ entry: RepoEntry) {
        gitPath = entry.path
        refreshGit()
        selection = .repository
    }

    func loadReadme() {
        let path = gitPath.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !path.isEmpty else { readmeContent = ""; return }
        let fm = FileManager.default
        let names = ["README.md", "README", "Readme.md", "readme.md"]
        for name in names {
            let readmePath = (path as NSString).appendingPathComponent(name)
            if fm.fileExists(atPath: readmePath),
               let data = fm.contents(atPath: readmePath),
               let text = String(data: data, encoding: .utf8) {
                readmeContent = text
                return
            }
        }
        readmeContent = ""
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

    // MARK: - Multiple Relay Management (Phase 24)

    func addRelay(url: String) {
        let trimmed = url.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty, !relays.contains(where: { $0.url == trimmed }) else { return }
        var entry = RelayEntry(url: trimmed)
        entry.status = .connecting
        relays.append(entry)
        connectRelayEntry(id: entry.id)
    }

    func removeRelay(id: UUID) {
        if let idx = relays.firstIndex(where: { $0.id == id }) {
            let relay = relays[idx]
            if relay.handle != 0 {
                nostrRelayClose(handle: relay.handle)
            }
            if relay.subHandle != 0 {
                nostrRelayUnsubscribe(subHandle: relay.subHandle)
            }
            relays.remove(at: idx)
            appendActivity("Removed relay: \(relay.url)")
        }
    }

    func connectRelayEntry(id: UUID) {
        guard let idx = relays.firstIndex(where: { $0.id == id }) else { return }
        var relay = relays[idx]
        let handle = nostrRelayConnect(url: relay.url)
        if handle != 0 {
            relay.handle = handle
            relay.status = .connected
            appendActivity("Relay connected: \(relay.url) (#\(handle))")
        } else {
            relay.status = .error
            appendActivity("Relay connect failed: \(relay.url)")
        }
        relays[idx] = relay
    }

    func disconnectRelayEntry(id: UUID) {
        guard let idx = relays.firstIndex(where: { $0.id == id }) else { return }
        var relay = relays[idx]
        if relay.handle != 0 {
            nostrRelayClose(handle: relay.handle)
            relay.handle = 0
        }
        if relay.subHandle != 0 {
            nostrRelayUnsubscribe(subHandle: relay.subHandle)
            relay.subHandle = 0
        }
        relay.status = .disconnected
        relays[idx] = relay
        appendActivity("Relay disconnected: \(relay.url)")
    }

    func reconnectAllRelays() {
        for relay in relays where relay.status != .connected {
            if let idx = relays.firstIndex(where: { $0.id == relay.id }) {
                var r = relays[idx]
                r.status = .connecting
                relays[idx] = r
                connectRelayEntry(id: relay.id)
            }
        }
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

    // MARK: - NIP-94 File Flow (Phase 16)

    func publishPickedFile(url: URL) {
        guard !nostrSecretKey.isEmpty else {
            appendActivity("No Nostr key — generate one first")
            return
        }
        let path = url.path
        let result = hybridPublishFile(
            filePath: path,
            secretKey: nostrSecretKey,
            relayHandle: relayHandle == 0 ? nil : relayHandle,
            gossipTopic: gossipTopic
        )
        if result.isEmpty {
            publishFileResult = "Publish failed"
            appendActivity("NIP-94 publish failed")
        } else {
            publishFileResult = result
            appendActivity("NIP-94 publish → CID: \(shortCID(result))")
        }
    }

    func resolveNip94() {
        let eventJson = nip94ResolveInput.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !eventJson.isEmpty else { return }
        let result = hybridResolveNip94(eventJson: eventJson)
        nip94ResolveResult = result
        appendActivity(result.isEmpty ? "NIP-94 resolve failed" : "NIP-94 resolved (\(result.count) chars)")
    }

    // MARK: - NIP-34 Git Flow (Phase 17)

    func publishRepo() {
        guard !nostrSecretKey.isEmpty else {
            appendActivity("No Nostr key — generate one first")
            return
        }
        let path = gitPath.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !path.isEmpty else {
            appendActivity("No repo loaded")
            return
        }
        let repoName = URL(fileURLWithPath: path).lastPathComponent
        let desc = nip34RepoDescription.trimmingCharacters(in: .whitespacesAndNewlines)
        let urls = nip34RepoCloneURLs.trimmingCharacters(in: .whitespacesAndNewlines)
            .split(separator: ",")
            .map { $0.trimmingCharacters(in: .whitespaces) }
            .filter { !$0.isEmpty }
        let result = hybridPublishRepo(
            repoPath: path,
            repoId: repoName,
            name: repoName,
            description: desc,
            cloneUrls: urls,
            secretKey: nostrSecretKey,
            relayHandle: relayHandle == 0 ? nil : relayHandle,
            gossipTopic: gossipTopic
        )
        nip34PublishResult = result
        appendActivity(result.isEmpty ? "Repo publish failed" : "Published repo: \(repoName)")
    }

    func publishPatch() {
        guard !nostrSecretKey.isEmpty else {
            appendActivity("No Nostr key — generate one first")
            return
        }
        let path = gitPath.trimmingCharacters(in: .whitespacesAndNewlines)
        let oldH = nip34PatchOldHash.trimmingCharacters(in: .whitespacesAndNewlines)
        let newH = nip34PatchNewHash.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !path.isEmpty, !oldH.isEmpty, !newH.isEmpty else {
            appendActivity("Need repo path and both commit hashes")
            return
        }
        let repoRef = gitHeadResult.isEmpty ? "repo" : gitHeadResult
        let result = hybridPublishPatch(
            repoPath: path,
            repoRef: repoRef,
            oldHash: oldH,
            newHash: newH,
            secretKey: nostrSecretKey,
            relayHandle: relayHandle == 0 ? nil : relayHandle,
            gossipTopic: gossipTopic
        )
        nip34PublishResult = result
        appendActivity(result.isEmpty ? "Patch publish failed" : "Published patch")
    }

    func publishIssue() {
        guard !nostrSecretKey.isEmpty else {
            appendActivity("No Nostr key — generate one first")
            return
        }
        let title = nip34IssueTitle.trimmingCharacters(in: .whitespacesAndNewlines)
        let body = nip34IssueBody.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !title.isEmpty else {
            appendActivity("Need issue title")
            return
        }
        let repoRef = gitHeadResult.isEmpty ? "repo" : gitHeadResult
        let result = hybridPublishIssue(
            repoRef: repoRef,
            title: title,
            body: body,
            secretKey: nostrSecretKey,
            relayHandle: relayHandle == 0 ? nil : relayHandle,
            gossipTopic: gossipTopic
        )
        nip34PublishResult = result
        appendActivity(result.isEmpty ? "Issue publish failed" : "Published issue: \(title)")
    }

    // MARK: - Typed Messages (Phase 15)

    func broadcastTypedFile() {
        guard !nostrSecretKey.isEmpty else {
            appendActivity("No Nostr key — generate one first")
            return
        }
        let cid = typedCID.trimmingCharacters(in: .whitespacesAndNewlines)
        let filename = typedFilename.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !cid.isEmpty, !filename.isEmpty else {
            appendActivity("Need CID and filename for NIP-94 file")
            return
        }
        let msg = HybridMessage(
            kind: 1063,
            content: filename,
            tags: [
                ["url", "ipfs://\(cid)"],
                ["m", guessMime(filename)],
            ]
        )
        do {
            let event = try hybridBroadcastTyped(msg: msg, secretKey: nostrSecretKey, relayHandle: relayHandle == 0 ? nil : relayHandle, gossipTopic: gossipTopic)
            appendActivity("Broadcast NIP-94 file: \(shortCID(cid))")
            typedContent = event
        } catch let error as RustyError {
            appendActivity("Broadcast failed: \(error.localizedDescription)")
        } catch {
            appendActivity("Broadcast failed: \(error)")
        }
    }

    func broadcastTypedRepo() {
        guard !nostrSecretKey.isEmpty else {
            appendActivity("No Nostr key — generate one first")
            return
        }
        let repoRef = typedRepoRef.trimmingCharacters(in: .whitespacesAndNewlines)
        let title = typedTitle.trimmingCharacters(in: .whitespacesAndNewlines)
        let body = typedBody.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !repoRef.isEmpty, !title.isEmpty else {
            appendActivity("Need repo ref and title for NIP-34 repo")
            return
        }
        let msg = HybridMessage(
            kind: 30617,
            content: body,
            tags: [
                ["d", repoRef],
                ["name", title],
            ]
        )
        do {
            let event = try hybridBroadcastTyped(msg: msg, secretKey: nostrSecretKey, relayHandle: relayHandle == 0 ? nil : relayHandle, gossipTopic: gossipTopic)
            appendActivity("Broadcast NIP-34 repo: \(repoRef)")
            typedContent = event
        } catch let error as RustyError {
            appendActivity("Broadcast failed: \(error.localizedDescription)")
        } catch {
            appendActivity("Broadcast failed: \(error)")
        }
    }

    func broadcastTypedPatch() {
        guard !nostrSecretKey.isEmpty else {
            appendActivity("No Nostr key — generate one first")
            return
        }
        let repoRef = typedRepoRef.trimmingCharacters(in: .whitespacesAndNewlines)
        let diff = typedDiff.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !repoRef.isEmpty, !diff.isEmpty else {
            appendActivity("Need repo ref and diff for NIP-34 patch")
            return
        }
        let msg = HybridMessage(
            kind: 1617,
            content: diff,
            tags: [
                ["e", repoRef],
                ["a", repoRef],
            ]
        )
        do {
            let event = try hybridBroadcastTyped(msg: msg, secretKey: nostrSecretKey, relayHandle: relayHandle == 0 ? nil : relayHandle, gossipTopic: gossipTopic)
            appendActivity("Broadcast NIP-34 patch: \(repoRef)")
            typedContent = event
        } catch let error as RustyError {
            appendActivity("Broadcast failed: \(error.localizedDescription)")
        } catch {
            appendActivity("Broadcast failed: \(error)")
        }
    }

    func broadcastTypedIssue() {
        guard !nostrSecretKey.isEmpty else {
            appendActivity("No Nostr key — generate one first")
            return
        }
        let repoRef = typedRepoRef.trimmingCharacters(in: .whitespacesAndNewlines)
        let title = typedTitle.trimmingCharacters(in: .whitespacesAndNewlines)
        let body = typedBody.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !repoRef.isEmpty, !title.isEmpty else {
            appendActivity("Need repo ref and title for NIP-34 issue")
            return
        }
        let msg = HybridMessage(
            kind: 1621,
            content: "\(title)\n\n\(body)",
            tags: [
                ["e", repoRef],
                ["a", repoRef],
            ]
        )
        do {
            let event = try hybridBroadcastTyped(msg: msg, secretKey: nostrSecretKey, relayHandle: relayHandle == 0 ? nil : relayHandle, gossipTopic: gossipTopic)
            appendActivity("Broadcast NIP-34 issue: \(repoRef)")
            typedContent = event
        } catch let error as RustyError {
            appendActivity("Broadcast failed: \(error.localizedDescription)")
        } catch {
            appendActivity("Broadcast failed: \(error)")
        }
    }

    func drainTyped() {
        do {
            let messages = try hybridDrainTyped(relaySubHandle: relayHandle == 0 ? nil : relayHandle)
            typedMessages = messages
            appendActivity("Drained \(messages.count) typed messages")
        } catch let error as RustyError {
            appendActivity("Drain failed: \(error.localizedDescription)")
        } catch {
            appendActivity("Drain failed: \(error)")
        }
    }

    func broadcastTyped() {
        switch typedBroadcastCategory {
        case .file: broadcastTypedFile()
        case .repo: broadcastTypedRepo()
        case .patch: broadcastTypedPatch()
        case .issue: broadcastTypedIssue()
        case .other: appendActivity("Select a valid category")
        }
    }

    func categoryFor(_ msg: HybridMessage) -> MessageCategory {
        switch msg.kind {
        case 1063: return .file
        case 30617: return .repo
        case 1617: return .patch
        case 1621: return .issue
        default: return .other
        }
    }

    func categoryName(_ msg: HybridMessage) -> String {
        switch msg.kind {
        case 1063: return "File"
        case 30617: return "Repo"
        case 1617: return "Patch"
        case 1621: return "Issue"
        default: return "Other"
        }
    }

    func filteredTypedMessages() -> [HybridMessage] {
        guard let filter = typedCategoryFilter else { return typedMessages }
        return typedMessages.filter { categoryFor($0) == filter }
    }

    // MARK: - Inbox (Phase 18)

    func refreshInbox() {
        do {
            let messages = try hybridDrainTyped(relaySubHandle: relayHandle == 0 ? nil : relayHandle)
            // Deduplicate by content (simple approach since we don't have event ID in HybridMessage)
            var seen = Set<String>()
            var unique: [HybridMessage] = []
            for msg in messages {
                let key = "\(msg.kind):\(msg.content)"
                if seen.insert(key).inserted {
                    unique.append(msg)
                }
            }
            let newCount = max(0, unique.count - inboxMessages.count)
            inboxMessages = unique
            unreadCount += newCount
            appendActivity("Inbox: \(unique.count) messages (\(newCount) new)")
        } catch let error as RustyError {
            appendActivity("Inbox refresh failed: \(error.localizedDescription)")
        } catch {
            appendActivity("Inbox refresh failed: \(error)")
        }
    }

    func filteredInbox() -> [HybridMessage] {
        guard let filter = inboxFilter else { return inboxMessages }
        return inboxMessages.filter { categoryFor($0) == filter }
    }

    func markInboxRead() {
        unreadCount = 0
    }

    // MARK: - Background Polling (Phase 20 + 24)

    func drainAllRelays() {
        var totalEvents = 0
        for relay in relays where relay.handle != 0 {
            let subHandle = nostrRelaySubscribe(handle: relay.handle, filterJson: "{\"kinds\":[1,1063,1617,1621,30617],\"limit\":20}")
            var events: [String] = []
            while let msg = nostrRelayDrain(subHandle: subHandle) {
                events.append(msg)
            }
            nostrRelayUnsubscribe(subHandle: subHandle)
            totalEvents += events.count
        }
        if totalEvents > 0 {
            appendActivity("Drained \(totalEvents) events from all relays")
        }
    }

    func startPolling() {
        guard !isPolling else { return }
        isPolling = true
        appendActivity("Background polling started")
        pollingTask = Task { [weak self] in
            while let self = self, self.isPolling {
                await Task.yield()
                try? await Task.sleep(nanoseconds: 5_000_000_000) // 5 seconds
                guard self.isPolling else { break }
                await MainActor.run {
                    self.drainAllRelays()
                    self.drainGossip()
                    self.refreshInbox()
                }
            }
        }
    }

    func stopPolling() {
        isPolling = false
        pollingTask?.cancel()
        pollingTask = nil
        appendActivity("Background polling stopped")
    }

    func togglePolling() {
        if isPolling {
            stopPolling()
        } else {
            startPolling()
        }
    }

    // MARK: - Network / DHT

    func dhtFindPeer() {
        let peerID = dhtPeerID.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !peerID.isEmpty else { return }
        let addrs = ipfsDhtFindpeer(peerId: peerID)
        dhtPeerAddrs = addrs
        appendActivity(addrs.isEmpty ? "DHT findpeer: no addrs" : "DHT findpeer: \(addrs.count) addrs")
    }

    func dhtFindProvs() {
        let cid = dhtCID.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !cid.isEmpty else { return }
        let provs = ipfsDhtFindprovs(cid: cid)
        dhtProviders = provs
        appendActivity(provs.isEmpty ? "DHT findprovs: none" : "DHT findprovs: \(provs.count) providers")
    }

    func p2pConnectTo() {
        let addr = p2pConnectAddr.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !addr.isEmpty else { return }
        let ok = p2pConnect(addr: addr)
        appendActivity(ok ? "Connected to \(addr)" : "Connect failed: \(addr)")
    }

    // MARK: - Name / Block

    func namePublish() {
        let cid = namePublishCID.trimmingCharacters(in: .whitespacesAndNewlines)
        let lifetime = Int64(namePublishLifetime) ?? 86400
        guard !cid.isEmpty else { return }
        let result = ipfsNamePublish(cid: cid, lifetimeSec: lifetime)
        namePublishResult = result
        appendActivity(result.isEmpty ? "Name publish failed" : "Name published → \(shortCID(result))")
    }

    func nameResolve() {
        let name = nameResolveName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !name.isEmpty else { return }
        let result = ipfsNameResolve(name: name)
        nameResolveResult = result
        appendActivity(result.isEmpty ? "Name resolve failed" : "Name resolved → \(shortCID(result))")
    }

    func blockPut() {
        let data = blockData.data(using: .utf8) ?? Data()
        guard !data.isEmpty else { return }
        let cid = ipfsBlockPut(data: data)
        blockCID = cid
        appendActivity(cid.isEmpty ? "Block put failed" : "Block put → \(shortCID(cid))")
    }

    func blockGet() {
        let cid = blockCID.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !cid.isEmpty else { return }
        let data = ipfsBlockGet(cid: cid)
        blockResult = String(data: data, encoding: .utf8) ?? "<\(data.count) bytes>"
        appendActivity("Block get → \(shortCID(cid)) (\(data.count) bytes)")
    }

    func blockStat() {
        let cid = blockCID.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !cid.isEmpty else { return }
        blockStatSize = ipfsBlockStat(cid: cid)
        appendActivity("Block stat → \(shortCID(cid)) size=\(blockStatSize)")
    }

    // MARK: - Git extras

    func lookupCommit() {
        let path = gitPath.trimmingCharacters(in: .whitespacesAndNewlines)
        let hash = commitHash.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !path.isEmpty, !hash.isEmpty else { return }
        commitMessageResult = gitCommitMessage(path: path, hash: hash)
        appendActivity(commitMessageResult.isEmpty ? "Commit lookup failed" : "Commit: \(commitMessageResult.prefix(40))…")
    }

    func diffTrees() {
        let path = gitPath.trimmingCharacters(in: .whitespacesAndNewlines)
        let oldH = diffOldHash.trimmingCharacters(in: .whitespacesAndNewlines)
        let newH = diffNewHash.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !path.isEmpty, !oldH.isEmpty, !newH.isEmpty else { return }
        diffResult = gitDiffTrees(path: path, oldHash: oldH, newHash: newH)
        appendActivity(diffResult.isEmpty ? "Diff failed" : "Diff generated (\(diffResult.count) chars)")
    }

    func appendActivity(_ message: String) {
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

    func guessMime(_ fileName: String) -> String {
        let ext = fileName.split(separator: ".").last?.lowercased() ?? ""
        switch ext {
        case "txt": return "text/plain"
        case "html", "htm": return "text/html"
        case "css": return "text/css"
        case "js": return "application/javascript"
        case "json": return "application/json"
        case "png": return "image/png"
        case "jpg", "jpeg": return "image/jpeg"
        case "gif": return "image/gif"
        case "svg": return "image/svg+xml"
        case "mp4": return "video/mp4"
        case "mp3": return "audio/mpeg"
        case "pdf": return "application/pdf"
        case "zip": return "application/zip"
        case "gz": return "application/gzip"
        case "tar": return "application/x-tar"
        case "md": return "text/markdown"
        case "rs": return "text/rust"
        case "go": return "text/x-go"
        case "swift": return "text/x-swift"
        default: return "application/octet-stream"
        }
    }

    var repoExists: Bool {
        let path = gitPath.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !path.isEmpty else { return false }
        return FileManager.default.fileExists(atPath: (path as NSString).appendingPathComponent(".git"))
    }

    func buildFileTree(path: String) -> [FileNode] {
        let fm = FileManager.default
        guard fm.fileExists(atPath: path) else {
            appendActivity("Tree: path does not exist: \(path)")
            return []
        }
        guard let entries = try? fm.contentsOfDirectory(atPath: path) else {
            appendActivity("Tree: cannot read directory: \(path)")
            return []
        }
        let visible = entries.filter { !$0.hasPrefix(".") }
        appendActivity("Tree: \(visible.count) items in \(path)")
        return visible.sorted().compactMap { name in
            let fullPath = (path as NSString).appendingPathComponent(name)
            var isDir: ObjCBool = false
            guard fm.fileExists(atPath: fullPath, isDirectory: &isDir) else { return nil }
            let children = isDir.boolValue ? buildFileTree(path: fullPath) : []
            return FileNode(name: name, path: fullPath, isDirectory: isDir.boolValue, children: children)
        }
    }
}

func kubo_rs_version() -> String {
    let id = hybridIpfsPeerId()
    return id.isEmpty ? "kubo-rs 0.8.1" : "kubo-rs (live)"
}
