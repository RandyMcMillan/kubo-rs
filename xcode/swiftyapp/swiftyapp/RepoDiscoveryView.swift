//
//  RepoDiscoveryView.swift
//  swiftyapp
//
//  Phase 23: Dedicated Repo Discovery view for kind:30617 (Repo) announcements.
//

import SwiftUI
import RustyLib

struct RepoDiscoveryView: View {
    @ObservedObject var store: HybridNodeStore

    private var repoMessages: [HybridMessage] {
        store.inboxMessages.filter { store.categoryFor($0) == .repo }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            HStack(spacing: 12) {
                Text("Repo Discovery")
                    .font(.title2.weight(.semibold))
                Spacer()
                Text("\(repoMessages.count) repo\(repoMessages.count == 1 ? "" : "s")")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
                Button {
                    store.refreshInbox()
                } label: {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .buttonStyle(.borderedProminent)
                .disabled(store.isRefreshing)
            }

            if repoMessages.isEmpty {
                VStack(spacing: 12) {
                    Image(systemName: "globe")
                        .font(.system(size: 48))
                        .foregroundStyle(.secondary)
                    Text("No repos discovered")
                        .font(.headline)
                        .foregroundStyle(.secondary)
                    Text("Incoming kind:30617 repo announcements will appear here.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .frame(maxWidth: .infinity, minHeight: 200)
                .background(Color.secondary.opacity(0.05))
                .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
            } else {
                LazyVStack(spacing: 16) {
                    ForEach(Array(repoMessages.enumerated()), id: \.offset) { _, msg in
                        RepoAnnouncementCard(msg: msg)
                    }
                }
            }
        }
    }
}

private struct RepoAnnouncementCard: View {
    let msg: HybridMessage

    private var repoMeta: (name: String, description: String, cloneURL: String, branch: String, commits: String) {
        var name = "Unknown Repo"
        var description = ""
        var cloneURL = ""
        var branch = "main"
        var commits = "0"

        for tag in msg.tags {
            if tag.count >= 2 {
                switch tag[0] {
                case "name": name = tag[1]
                case "description": description = tag[1]
                case "clone": cloneURL = tag[1]
                case "branch": branch = tag[1]
                case "commits": commits = tag[1]
                default: break
                }
            }
        }

        if let data = msg.content.data(using: .utf8),
           let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] {
            if let n = json["name"] as? String { name = n }
            if let d = json["description"] as? String { description = d }
            if let c = json["clone_url"] as? String { cloneURL = c }
        }

        return (name, description, cloneURL, branch, commits)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(spacing: 12) {
                VStack(alignment: .leading, spacing: 4) {
                    Text(repoMeta.name)
                        .font(.headline)
                    Text("\(repoMeta.branch) • \(repoMeta.commits) commits")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Text("kind:\(msg.kind)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Capsule(style: .continuous).fill(Color.secondary.opacity(0.1)))
            }
            if !repoMeta.description.isEmpty {
                Text(repoMeta.description)
                    .font(.body)
                    .foregroundStyle(.secondary)
            }
            if !repoMeta.cloneURL.isEmpty {
                HStack(spacing: 8) {
                    Image(systemName: "link")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    Text(repoMeta.cloneURL)
                        .font(.caption)
                        .foregroundStyle(.accent)
                        .lineLimit(1)
                    Spacer()
                }
            }
            HStack(spacing: 12) {
                Button {
                    // Clone action would go here
                } label: {
                    Label("Clone", systemImage: "arrow.down.circle")
                }
                .buttonStyle(.borderedProminent)

                Button {
                    // Subscribe action would go here
                } label: {
                    Label("Subscribe", systemImage: "bell")
                }
                .buttonStyle(.bordered)

                Spacer()
            }
        }
        .padding(16)
        .background(Color.secondary.opacity(0.05))
        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
    }
}
