//
//  ContentView.swift
//  swiftyapp
//
//  Created by Jonathan McKenzie on 7/9/24.
//

import Foundation
import SwiftUI
import RustyLib

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
                case .repos:
                    reposContent
                case .repository:
                    repositoryContent
                case .network:
                    networkContent
                case .chat:
                    chatContent
                case .codeReview:
                    CodeReviewView(store: store)
                case .issueTracker:
                    IssueTrackerView(store: store)
                case .repoDiscovery:
                    RepoDiscoveryView(store: store)
                case .activity:
                    activityContent
                case .settings:
                    SettingsView(store: store)
                }
            }
            .padding(24)
        }
        .fileImporter(
            isPresented: $store.showFileImporter,
            allowedContentTypes: [.data],
            allowsMultipleSelection: false
        ) { result in
            switch result {
            case .success(let urls):
                if let url = urls.first {
                    store.publishPickedFile(url: url)
                }
            case .failure(let error):
                store.appendActivity("File picker error: \(error.localizedDescription)")
            }
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
                        MetricPill(title: "Peer ID", value: store.snapshot.peerID, symbol: "person.crop.circle")
                        MetricPill(title: "CID", value: store.snapshot.cid, symbol: "link")
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
                MetricCard(title: "IPFS Peer", value: store.snapshot.peerID, symbol: "person.2.circle", subtitle: "HybridNode IPFS identity")
                MetricCard(title: "P2P Peer", value: hybridP2pPeerId(), symbol: "network", subtitle: "HybridNode libp2p identity")
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
                            Text("CID: \(store.lastAddedCID)")
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

            DashboardCard(title: "Block Operations") {
                VStack(alignment: .leading, spacing: 12) {
                    TextField("Block data…", text: $store.blockData, axis: .vertical)
                        .textFieldStyle(.roundedBorder)
                        .lineLimit(1...3)
                    HStack(spacing: 12) {
                        Button {
                            store.blockPut()
                        } label: {
                            Label("Put", systemImage: "square.and.arrow.down")
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(store.blockData.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                        if !store.blockCID.isEmpty {
                            Button {
                                store.blockGet()
                            } label: {
                                Label("Get", systemImage: "square.and.arrow.up")
                            }
                            .buttonStyle(.bordered)

                            Button {
                                store.blockStat()
                            } label: {
                                Label("Stat", systemImage: "info.circle")
                            }
                            .buttonStyle(.bordered)
                        }

                        Spacer()
                    }
                    if !store.blockCID.isEmpty {
                        Text("CID: \(store.blockCID)")
                            .font(.system(.body, design: .monospaced))
                            .textSelection(.enabled)
                    }
                    if !store.blockResult.isEmpty {
                        Text(store.blockResult)
                            .font(.system(.body, design: .monospaced))
                            .textSelection(.enabled)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    if store.blockStatSize > 0 {
                        Text("Size: \(store.blockStatSize) bytes")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }

            DashboardCard(title: "Publish File (NIP-94)") {
                VStack(alignment: .leading, spacing: 12) {
                    if store.nostrSecretKey.isEmpty {
                        Text("Generate a Nostr key in Settings or Network > Nostr first.")
                            .foregroundStyle(.secondary)
                    } else {
                        Button {
                            store.showFileImporter = true
                        } label: {
                            Label("Pick file & publish", systemImage: "doc.badge.plus")
                        }
                        .buttonStyle(.borderedProminent)

                        if !store.publishFileResult.isEmpty {
                            Text("CID: \(store.publishFileResult)")
                                .font(.system(.body, design: .monospaced))
                                .textSelection(.enabled)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }
            }

            DashboardCard(title: "Resolve NIP-94") {
                VStack(alignment: .leading, spacing: 12) {
                    TextField("Paste NIP-94 event JSON…", text: $store.nip94ResolveInput, axis: .vertical)
                        .textFieldStyle(.roundedBorder)
                        .lineLimit(2...6)
                    Button {
                        store.resolveNip94()
                    } label: {
                        Label("Resolve", systemImage: "arrow.down.circle")
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(store.nip94ResolveInput.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                    if !store.nip94ResolveResult.isEmpty {
                        Text(store.nip94ResolveResult)
                            .font(.system(.body, design: .monospaced))
                            .textSelection(.enabled)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
            }
        }
    }

    private var reposContent: some View {
        VStack(alignment: .leading, spacing: 20) {
            DashboardCard(title: "Hosted Repositories") {
                VStack(alignment: .leading, spacing: 12) {
                    HStack(spacing: 12) {
                        Button {
                            store.scanForRepos()
                        } label: {
                            Label("Scan", systemImage: "arrow.clockwise")
                        }
                        .buttonStyle(.bordered)
                        Spacer()
                    }
                    if store.repos.isEmpty {
                        Text("No repositories found. Clone or init a repo to get started.")
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(store.repos) { repo in
                            Button {
                                store.selectRepo(repo)
                            } label: {
                                HStack(spacing: 12) {
                                    Image(systemName: "folder.fill")
                                        .foregroundStyle(.accent)
                                    VStack(alignment: .leading, spacing: 2) {
                                        Text(repo.name)
                                            .font(.headline)
                                        Text(repo.branch)
                                            .font(.caption)
                                            .foregroundStyle(.secondary)
                                        Text(repo.head.prefix(7))
                                            .font(.caption)
                                            .foregroundStyle(.secondary)
                                    }
                                    Spacer()
                                    Image(systemName: "chevron.right")
                                        .foregroundStyle(.secondary)
                                }
                            }
                            .buttonStyle(.plain)
                            if repo.id != store.repos.last?.id {
                                Divider()
                            }
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
                    Toggle("Force (delete existing)", isOn: $store.forceClone)
                        .font(.caption)
                    Button {
                        store.gitCloneRepo()
                    } label: {
                        Label("Clone", systemImage: "arrow.down.doc")
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(store.cloneURL.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || store.clonePath.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                    if !store.cloneResult.isEmpty {
                        Text(store.cloneResult)
                            .font(.system(.caption, design: .monospaced))
                            .textSelection(.enabled)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    if !store.gitPath.isEmpty {
                        Button {
                            store.selection = .repository
                        } label: {
                            Label("View Repository", systemImage: "externaldrive.connected.to.line.below")
                        }
                        .buttonStyle(.bordered)
                    }
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
                HStack(spacing: 12) {
                    Button {
                        store.fetchAll()
                    } label: {
                        Label("Fetch all", systemImage: "arrow.down.circle")
                    }
                    .buttonStyle(.bordered)
                    .disabled(!store.repoExists)
                    Spacer()
                }
                if !store.fetchResult.isEmpty {
                    Text(store.fetchResult)
                        .font(.system(.caption, design: .monospaced))
                        .textSelection(.enabled)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            }

            if !store.readmeContent.isEmpty {
                DashboardCard(title: "README") {
                    VStack(alignment: .leading, spacing: 8) {
                        ScrollView {
                            Text(store.readmeContent)
                                .font(.system(.body, design: .monospaced))
                                .textSelection(.enabled)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                        .frame(maxHeight: 300)
                    }
                }
            }

            DashboardCard(title: "Commits") {
                VStack(alignment: .leading, spacing: 8) {
                    if store.commitHistory.isEmpty {
                        Text("No commits found.")
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(store.commitHistory) { commit in
                            VStack(alignment: .leading, spacing: 2) {
                                HStack(spacing: 8) {
                                    Text(commit.hash.prefix(7))
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                    Text(commit.author)
                                        .font(.caption)
                                        .foregroundStyle(.accent)
                                    Spacer()
                                }
                                Text(commit.message.trimmingCharacters(in: .whitespacesAndNewlines))
                                    .font(.system(.body, design: .monospaced))
                                    .lineLimit(2)
                            }
                            .padding(.vertical, 2)
                        }
                    }
                }
            }

            if !store.tagList.isEmpty {
                DashboardCard(title: "Tags") {
                    FlowLayout(spacing: 8) {
                        ForEach(store.tagList, id: \.self) { tag in
                            Text(tag)
                                .font(.caption)
                                .padding(.horizontal, 10)
                                .padding(.vertical, 6)
                                .background(Capsule(style: .continuous).fill(Color.accentColor.opacity(0.15)))
                        }
                    }
                }
            }

            DashboardCard(title: "Repo tree") {
                VStack(alignment: .leading, spacing: 8) {
                    HStack(spacing: 12) {
                        Text(store.gitPath.isEmpty ? "No path" : store.gitPath)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                        Spacer()
                        Button {
                            store.refreshGit()
                        } label: {
                            Label("Refresh", systemImage: "arrow.clockwise")
                        }
                        .buttonStyle(.bordered)
                        .disabled(store.gitPath.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                    }
                    if store.repoTree.isEmpty {
                        Text("No files found. Init or clone a repository, then refresh.")
                            .foregroundStyle(.secondary)
                    } else {
                        FileTreeView(nodes: store.repoTree) { path in
                            store.viewFile(path: path)
                        }
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }

            if !store.selectedFilePath.isEmpty {
                DashboardCard(title: "File: \(URL(fileURLWithPath: store.selectedFilePath).lastPathComponent)") {
                    VStack(alignment: .leading, spacing: 8) {
                        HStack(spacing: 12) {
                            Text(store.selectedFilePath)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                                .lineLimit(1)
                            Spacer()
                            Button {
                                store.viewBlame()
                            } label: {
                                Label("Blame", systemImage: "person.text.rectangle")
                            }
                            .buttonStyle(.bordered)
                            Button {
                                store.selectedFilePath = ""
                                store.selectedFileContent = ""
                                store.blameLines = []
                                store.blameError = ""
                            } label: {
                                Label("Close", systemImage: "xmark")
                            }
                            .buttonStyle(.bordered)
                        }
                        if !store.blameError.isEmpty {
                            Text(store.blameError)
                                .font(.caption)
                                .foregroundStyle(.red)
                        }
                        if store.blameLines.isEmpty {
                            ScrollView {
                                Text(store.selectedFileContent)
                                    .font(.system(.body, design: .monospaced))
                                    .textSelection(.enabled)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                            }
                            .frame(maxHeight: 400)
                        } else {
                            ScrollView {
                                VStack(alignment: .leading, spacing: 2) {
                                    ForEach(Array(store.blameLines.enumerated()), id: \.offset) { _, line in
                                        HStack(spacing: 8) {
                                            Text(line.hash.prefix(7))
                                                .font(.caption)
                                                .foregroundStyle(.secondary)
                                                .frame(width: 60, alignment: .leading)
                                            Text(line.name)
                                                .font(.caption)
                                                .foregroundStyle(.accent)
                                                .frame(width: 100, alignment: .leading)
                                                .lineLimit(1)
                                            Text(line.text)
                                                .font(.system(.body, design: .monospaced))
                                            Spacer()
                                        }
                                    }
                                }
                            }
                            .frame(maxHeight: 400)
                        }
                    }
                }
            }

            DashboardCard(title: "Commit lookup") {
                VStack(alignment: .leading, spacing: 12) {
                    TextField("Commit hash…", text: $store.commitHash)
                        .textFieldStyle(.roundedBorder)
                    Button {
                        store.lookupCommit()
                    } label: {
                        Label("Lookup", systemImage: "doc.text.magnifyingglass")
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(store.commitHash.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || store.gitPath.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                    if !store.commitMessageResult.isEmpty {
                        Text(store.commitMessageResult)
                            .font(.system(.body, design: .monospaced))
                            .textSelection(.enabled)
                    }
                }
            }

            DashboardCard(title: "Diff trees") {
                VStack(alignment: .leading, spacing: 12) {
                    HStack(spacing: 12) {
                        TextField("Old hash…", text: $store.diffOldHash)
                            .textFieldStyle(.roundedBorder)
                        TextField("New hash…", text: $store.diffNewHash)
                            .textFieldStyle(.roundedBorder)
                    }
                    Button {
                        store.diffTrees()
                    } label: {
                        Label("Diff", systemImage: "doc.text.below.ecg")
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(store.diffOldHash.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || store.diffNewHash.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || store.gitPath.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                    if !store.diffResult.isEmpty {
                        Text(store.diffResult)
                            .font(.system(.body, design: .monospaced))
                            .textSelection(.enabled)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
            }

            DashboardCard(title: "Publish Repo (NIP-34)") {
                VStack(alignment: .leading, spacing: 12) {
                    if store.nostrSecretKey.isEmpty {
                        Text("Generate a Nostr key in Settings first.")
                            .foregroundStyle(.secondary)
                    } else {
                        TextField("Description", text: $store.nip34RepoDescription)
                            .textFieldStyle(.roundedBorder)
                        TextField("Clone URLs (comma-separated)", text: $store.nip34RepoCloneURLs)
                            .textFieldStyle(.roundedBorder)
                        Button {
                            store.publishRepo()
                        } label: {
                            Label("Publish repo", systemImage: "globe")
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(store.gitPath.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                    }
                }
            }

            DashboardCard(title: "Publish Patch (NIP-34)") {
                VStack(alignment: .leading, spacing: 12) {
                    HStack(spacing: 12) {
                        TextField("Old hash…", text: $store.nip34PatchOldHash)
                            .textFieldStyle(.roundedBorder)
                        TextField("New hash…", text: $store.nip34PatchNewHash)
                            .textFieldStyle(.roundedBorder)
                    }
                    Button {
                        store.publishPatch()
                    } label: {
                        Label("Publish patch", systemImage: "doc.text.below.ecg")
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(store.nip34PatchOldHash.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || store.nip34PatchNewHash.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || store.gitPath.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || store.nostrSecretKey.isEmpty)
                }
            }

            DashboardCard(title: "Publish Issue (NIP-34)") {
                VStack(alignment: .leading, spacing: 12) {
                    TextField("Title", text: $store.nip34IssueTitle)
                        .textFieldStyle(.roundedBorder)
                    TextField("Body", text: $store.nip34IssueBody, axis: .vertical)
                        .textFieldStyle(.roundedBorder)
                        .lineLimit(2...6)
                    Button {
                        store.publishIssue()
                    } label: {
                        Label("Publish issue", systemImage: "exclamationmark.bubble")
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(store.nip34IssueTitle.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || store.nostrSecretKey.isEmpty)
                }
            }

            if !store.nip34PublishResult.isEmpty {
                DashboardCard(title: "NIP-34 Result") {
                    Text(store.nip34PublishResult)
                        .font(.system(.caption, design: .monospaced))
                        .textSelection(.enabled)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            }
        }
    }

    private var networkContent: some View {
        VStack(alignment: .leading, spacing: 20) {
            Picker("Network", selection: $store.networkTab) {
                ForEach(NetworkTab.allCases, id: \.self) { tab in
                    Text(tab.rawValue).tag(tab)
                }
            }
            .pickerStyle(.segmented)

            if store.networkTab == .ipfs {
                ipfsNetworkContent
            } else if store.networkTab == .nostr {
                nostrNetworkContent
            } else {
                p2pNetworkContent
            }
        }
    }

    private var ipfsNetworkContent: some View {
        VStack(alignment: .leading, spacing: 20) {
            LazyVGrid(columns: adaptiveColumns, spacing: 16) {
                MetricCard(title: "IPFS Peer", value: store.snapshot.peerID, symbol: "network", subtitle: "HybridNode IPFS identity")
                MetricCard(title: "P2P Peer", value: hybridP2pPeerId(), symbol: "point.3.connected.trianglepath.dotted", subtitle: "HybridNode libp2p identity")
                MetricCard(title: "Pins", value: "\(store.pins.count)", symbol: "pin", subtitle: "Locally pinned CIDs")
                MetricCard(title: "Status", value: store.snapshot.status, symbol: "power.circle", subtitle: "HybridNode online state")
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

                    Text("Dial peer")
                        .font(.headline)
                        .padding(.top, 4)
                    TextField("Multiaddr…", text: $store.p2pConnectAddr)
                        .textFieldStyle(.roundedBorder)
                    Button {
                        store.p2pConnectTo()
                    } label: {
                        Label("Connect", systemImage: "network.badge.shield.half.filled")
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(store.p2pConnectAddr.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                }
            }

            DashboardCard(title: "IPFS Network") {
                VStack(alignment: .leading, spacing: 16) {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("DHT FindPeer")
                            .font(.headline)
                        TextField("Peer ID…", text: $store.dhtPeerID)
                            .textFieldStyle(.roundedBorder)
                        Button {
                            store.dhtFindPeer()
                        } label: {
                            Label("Find peer", systemImage: "magnifyingglass.circle")
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(store.dhtPeerID.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                        if !store.dhtPeerAddrs.isEmpty {
                            ForEach(store.dhtPeerAddrs, id: \.self) { addr in
                                Text(addr)
                                    .font(.system(.caption, design: .monospaced))
                            }
                        }
                    }

                    Divider()

                    VStack(alignment: .leading, spacing: 8) {
                        Text("DHT FindProvs")
                            .font(.headline)
                        TextField("CID…", text: $store.dhtCID)
                            .textFieldStyle(.roundedBorder)
                        Button {
                            store.dhtFindProvs()
                        } label: {
                            Label("Find providers", systemImage: "magnifyingglass.circle.fill")
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(store.dhtCID.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                        if !store.dhtProviders.isEmpty {
                            ForEach(store.dhtProviders, id: \.self) { prov in
                                Text(prov)
                                    .font(.system(.caption, design: .monospaced))
                            }
                        }
                    }

                    Divider()

                    VStack(alignment: .leading, spacing: 8) {
                        Text("IPNS Publish")
                            .font(.headline)
                        HStack(spacing: 12) {
                            TextField("CID…", text: $store.namePublishCID)
                                .textFieldStyle(.roundedBorder)
                            TextField("Lifetime (s)", text: $store.namePublishLifetime)
                                .textFieldStyle(.roundedBorder)
                                .frame(width: 100)
                        }
                        Button {
                            store.namePublish()
                        } label: {
                            Label("Publish", systemImage: "globe")
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(store.namePublishCID.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                        if !store.namePublishResult.isEmpty {
                            Text(store.namePublishResult)
                                .font(.system(.caption, design: .monospaced))
                                .textSelection(.enabled)
                        }
                    }

                    Divider()

                    VStack(alignment: .leading, spacing: 8) {
                        Text("IPNS Resolve")
                            .font(.headline)
                        TextField("Name / IPNS key…", text: $store.nameResolveName)
                            .textFieldStyle(.roundedBorder)
                        Button {
                            store.nameResolve()
                        } label: {
                            Label("Resolve", systemImage: "globe.badge.chevron.backward")
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(store.nameResolveName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                        if !store.nameResolveResult.isEmpty {
                            Text(store.nameResolveResult)
                                .font(.system(.caption, design: .monospaced))
                                .textSelection(.enabled)
                        }
                    }
                }
            }
        }
    }

    private var nostrNetworkContent: some View {
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

            DashboardCard(title: "Typed Messages") {
                VStack(alignment: .leading, spacing: 12) {
                    Picker("Category", selection: $store.typedBroadcastCategory) {
                        Text("File").tag(MessageCategory.file)
                        Text("Repo").tag(MessageCategory.repo)
                        Text("Patch").tag(MessageCategory.patch)
                        Text("Issue").tag(MessageCategory.issue)
                    }
                    .pickerStyle(.segmented)

                    if store.typedBroadcastCategory == .file {
                        TextField("CID", text: $store.typedCID)
                            .textFieldStyle(.roundedBorder)
                        TextField("Filename", text: $store.typedFilename)
                            .textFieldStyle(.roundedBorder)
                    } else if store.typedBroadcastCategory == .repo {
                        TextField("Repo ref", text: $store.typedRepoRef)
                            .textFieldStyle(.roundedBorder)
                        TextField("Title", text: $store.typedTitle)
                            .textFieldStyle(.roundedBorder)
                        TextField("Body", text: $store.typedBody, axis: .vertical)
                            .textFieldStyle(.roundedBorder)
                            .lineLimit(2...4)
                    } else if store.typedBroadcastCategory == .patch {
                        TextField("Repo ref", text: $store.typedRepoRef)
                            .textFieldStyle(.roundedBorder)
                        TextField("Diff", text: $store.typedDiff, axis: .vertical)
                            .textFieldStyle(.roundedBorder)
                            .lineLimit(3...6)
                    } else if store.typedBroadcastCategory == .issue {
                        TextField("Repo ref", text: $store.typedRepoRef)
                            .textFieldStyle(.roundedBorder)
                        TextField("Title", text: $store.typedTitle)
                            .textFieldStyle(.roundedBorder)
                        TextField("Body", text: $store.typedBody, axis: .vertical)
                            .textFieldStyle(.roundedBorder)
                            .lineLimit(2...4)
                    }

                    HStack(spacing: 12) {
                        Button {
                            store.broadcastTyped()
                        } label: {
                            Label("Broadcast", systemImage: "dot.radiowaves.left.and.right")
                        }
                        .buttonStyle(.borderedProminent)

                        Button {
                            store.drainTyped()
                        } label: {
                            Label("Drain", systemImage: "arrow.down")
                        }
                        .buttonStyle(.bordered)

                        Spacer()
                    }

                    Divider()

                    HStack(spacing: 12) {
                        Text("Filter:")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        Picker("Filter", selection: $store.typedCategoryFilter) {
                            Text("All").tag(MessageCategory?.none)
                            Text("File").tag(MessageCategory?.some(.file))
                            Text("Repo").tag(MessageCategory?.some(.repo))
                            Text("Patch").tag(MessageCategory?.some(.patch))
                            Text("Issue").tag(MessageCategory?.some(.issue))
                        }
                        .pickerStyle(.segmented)
                    }

                    let filtered = store.filteredTypedMessages()
                    if filtered.isEmpty {
                        Text("No typed messages yet.")
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(Array(filtered.enumerated()), id: \.offset) { _, msg in
                            VStack(alignment: .leading, spacing: 4) {
                                HStack(spacing: 8) {
                                    Text(store.categoryName(msg))
                                        .font(.caption.weight(.semibold))
                                        .padding(.horizontal, 8)
                                        .padding(.vertical, 4)
                                        .background(Capsule(style: .continuous).fill(Color.accentColor.opacity(0.15)))
                                    Text("kind:\(msg.kind)")
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                    Spacer()
                                }
                                Text(msg.content)
                                    .font(.system(.body, design: .monospaced))
                                    .lineLimit(3)
                                if !msg.tags.isEmpty {
                                    Text(msg.tags.map { $0.joined(separator: ":") }.joined(separator: ", "))
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                        .lineLimit(1)
                                }
                            }
                            .padding(.vertical, 4)
                        }
                    }
                }
            }

            DashboardCard(title: "Inbox") {
                VStack(alignment: .leading, spacing: 12) {
                    HStack(spacing: 12) {
                        Button {
                            store.refreshInbox()
                            store.markInboxRead()
                        } label: {
                            Label("Refresh", systemImage: "arrow.clockwise")
                        }
                        .buttonStyle(.borderedProminent)

                        if store.unreadCount > 0 {
                            Text("\(store.unreadCount) new")
                                .font(.caption.weight(.semibold))
                                .padding(.horizontal, 8)
                                .padding(.vertical, 4)
                                .background(Capsule(style: .continuous).fill(Color.red.opacity(0.2)))
                        }

                        Spacer()

                        Picker("Filter", selection: $store.inboxFilter) {
                            Text("All").tag(MessageCategory?.none)
                            Text("File").tag(MessageCategory?.some(.file))
                            Text("Repo").tag(MessageCategory?.some(.repo))
                            Text("Patch").tag(MessageCategory?.some(.patch))
                            Text("Issue").tag(MessageCategory?.some(.issue))
                        }
                        .pickerStyle(.segmented)
                    }

                    let filtered = store.filteredInbox()
                    if filtered.isEmpty {
                        Text("No messages in inbox.")
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(Array(filtered.enumerated()), id: \.offset) { _, msg in
                            VStack(alignment: .leading, spacing: 4) {
                                HStack(spacing: 8) {
                                    Text(store.categoryName(msg))
                                        .font(.caption.weight(.semibold))
                                        .padding(.horizontal, 8)
                                        .padding(.vertical, 4)
                                        .background(Capsule(style: .continuous).fill(Color.accentColor.opacity(0.15)))
                                    Text("kind:\(msg.kind)")
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                    Spacer()
                                }
                                Text(msg.content)
                                    .font(.system(.body, design: .monospaced))
                                    .lineLimit(3)
                                if !msg.tags.isEmpty {
                                    Text(msg.tags.map { $0.joined(separator: ":") }.joined(separator: ", "))
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                        .lineLimit(1)
                                }
                            }
                            .padding(.vertical, 4)
                        }
                    }
                }
            }
        }
    }

    private var p2pNetworkContent: some View {
        VStack(alignment: .leading, spacing: 20) {
            LazyVGrid(columns: adaptiveColumns, spacing: 16) {
                MetricCard(title: "Local peer", value: peers.localPeerName, symbol: "person.crop.circle", subtitle: "Unique instance name advertised on the LAN")
                MetricCard(title: "Discovery", value: peers.connectionStatus, symbol: "antenna.radiowaves.left.and.right", subtitle: "Browsing and advertising via MultipeerConnectivity")
                MetricCard(title: "libp2p peer", value: peers.libp2pPeerID, symbol: "network", subtitle: "Rust host with relay and hole-punch support enabled")
                MetricCard(title: "Gossip topic", value: peers.gossipTopic, symbol: "bubble.left.and.bubble.right", subtitle: "Shared pubsub topic joined by the libp2p host")
                MetricCard(title: "Connected peers", value: "\(peers.connectedPeers.count)", symbol: "person.2.circle", subtitle: "Peers with an active session")
                MetricCard(title: "Last message", value: peers.lastMessage, symbol: "message", subtitle: "Latest p2p status or broadcast")
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

#Preview {
    ContentView()
}
