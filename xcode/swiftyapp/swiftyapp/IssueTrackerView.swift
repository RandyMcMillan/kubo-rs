//
//  IssueTrackerView.swift
//  swiftyapp
//
//  Phase 22: Dedicated Issue Tracker view for kind:1621 (Issue) messages.
//

import SwiftUI
import RustyLib

struct IssueTrackerView: View {
    @ObservedObject var store: HybridNodeStore

    private var issueMessages: [HybridMessage] {
        store.inboxMessages.filter { store.categoryFor($0) == .issue }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            HStack(spacing: 12) {
                Text("Issue Tracker")
                    .font(.title2.weight(.semibold))
                Spacer()
                Text("\(issueMessages.count) issue\(issueMessages.count == 1 ? "" : "s")")
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

            if issueMessages.isEmpty {
                VStack(spacing: 12) {
                    Image(systemName: "exclamationmark.bubble")
                        .font(.system(size: 48))
                        .foregroundStyle(.secondary)
                    Text("No issues tracked")
                        .font(.headline)
                        .foregroundStyle(.secondary)
                    Text("Incoming kind:1621 issue messages will appear here.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .frame(maxWidth: .infinity, minHeight: 200)
                .background(Color.secondary.opacity(0.05))
                .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
            } else {
                LazyVStack(spacing: 16) {
                    ForEach(Array(issueMessages.enumerated()), id: \.offset) { _, msg in
                        IssueCard(msg: msg)
                    }
                }
            }
        }
    }
}

private struct IssueCard: View {
    let msg: HybridMessage

    private var issueMeta: (title: String, body: String, author: String, status: String) {
        var title = "Untitled Issue"
        var body = msg.content
        var author = "anonymous"
        var status = "open"

        for tag in msg.tags {
            if tag.count >= 2 {
                switch tag[0] {
                case "title": title = tag[1]
                case "author": author = tag[1]
                case "status": status = tag[1]
                default: break
                }
            }
        }

        if let data = msg.content.data(using: .utf8),
           let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] {
            if let t = json["title"] as? String { title = t }
            if let b = json["content"] as? String { body = b }
        }

        return (title, body, author, status)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(spacing: 12) {
                VStack(alignment: .leading, spacing: 4) {
                    Text(issueMeta.title)
                        .font(.headline)
                    Text("by \(issueMeta.author)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Text(issueMeta.status)
                    .font(.caption.weight(.semibold))
                    .padding(.horizontal, 10)
                    .padding(.vertical, 6)
                    .background(Capsule(style: .continuous).fill(issueMeta.status == "open" ? Color.green.opacity(0.15) : Color.red.opacity(0.15)))
            }
            Text(issueMeta.body)
                .font(.body)
                .lineLimit(4)
            HStack {
                Spacer()
                Text("kind:\(msg.kind)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(16)
        .background(Color.secondary.opacity(0.05))
        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
    }
}
