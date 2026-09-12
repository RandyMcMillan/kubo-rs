//
//  CodeReviewView.swift
//  swiftyapp
//
//  Phase 21: Dedicated Code Review view for kind:1617 (Patch) messages.
//

import SwiftUI
import RustyLib

struct CodeReviewView: View {
    @ObservedObject var store: HybridNodeStore
    @State private var selectedPatch: HybridMessage? = nil
    @State private var approvedPatches: Set<String> = []
    @State private var rejectedPatches: Set<String> = []

    private var patchMessages: [HybridMessage] {
        store.inboxMessages.filter { store.categoryFor($0) == .patch }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            HStack(spacing: 12) {
                Text("Code Review")
                    .font(.title2.weight(.semibold))
                Spacer()
                Text("\(patchMessages.count) patch\(patchMessages.count == 1 ? "" : "es")")
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

            if patchMessages.isEmpty {
                VStack(spacing: 12) {
                    Image(systemName: "doc.text.magnifyingglass")
                        .font(.system(size: 48))
                        .foregroundStyle(.secondary)
                    Text("No patches to review")
                        .font(.headline)
                        .foregroundStyle(.secondary)
                    Text("Incoming kind:1617 patch messages will appear here.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .frame(maxWidth: .infinity, minHeight: 200)
                .background(Color.secondary.opacity(0.05))
                .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
            } else {
                LazyVStack(spacing: 16) {
                    ForEach(Array(patchMessages.enumerated()), id: \.offset) { _, msg in
                        PatchCard(
                            msg: msg,
                            isApproved: approvedPatches.contains(msg.content),
                            isRejected: rejectedPatches.contains(msg.content),
                            isSelected: selectedPatch?.content == msg.content,
                            onSelect: { selectedPatch = msg },
                            onApprove: { approvedPatches.insert(msg.content) },
                            onReject: { rejectedPatches.insert(msg.content) }
                        )
                    }
                }
            }
        }
    }
}

private struct PatchCard: View {
    let msg: HybridMessage
    let isApproved: Bool
    let isRejected: Bool
    let isSelected: Bool
    let onSelect: () -> Void
    let onApprove: () -> Void
    let onReject: () -> Void

    private var patchMeta: (repo: String, branch: String, author: String, diff: String) {
        var repo = "unknown"
        var branch = "main"
        var author = "anonymous"
        var diff = msg.content

        for tag in msg.tags {
            if tag.count >= 2 {
                switch tag[0] {
                case "repo": repo = tag[1]
                case "branch": branch = tag[1]
                case "author": author = tag[1]
                default: break
                }
            }
        }

        // If content looks like a JSON patch event, try to extract diff
        if let data = msg.content.data(using: .utf8),
           let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] {
            if let content = json["content"] as? String {
                diff = content
            }
        }

        return (repo, branch, author, diff)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(spacing: 12) {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Patch: \(patchMeta.repo)")
                        .font(.headline)
                    Text("\(patchMeta.branch) by \(patchMeta.author)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                HStack(spacing: 8) {
                    if isApproved {
                        Label("Approved", systemImage: "checkmark.circle.fill")
                            .font(.caption.weight(.semibold))
                            .foregroundStyle(.green)
                    } else if isRejected {
                        Label("Rejected", systemImage: "xmark.circle.fill")
                            .font(.caption.weight(.semibold))
                            .foregroundStyle(.red)
                    }
                    Text("kind:\(msg.kind)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(Capsule(style: .continuous).fill(Color.secondary.opacity(0.1)))
                }
            }

            if isSelected {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Diff")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.secondary)
                    ScrollView(.horizontal, showsIndicators: false) {
                        Text(patchMeta.diff)
                            .font(.system(.caption, design: .monospaced))
                            .lineLimit(nil)
                            .frame(minWidth: 400, alignment: .leading)
                    }
                    .frame(maxHeight: 300)
                    .padding(8)
                    .background(Color.secondary.opacity(0.05))
                    .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))

                    HStack(spacing: 12) {
                        Button {
                            onApprove()
                        } label: {
                            Label("Approve", systemImage: "checkmark.circle")
                        }
                        .buttonStyle(.borderedProminent)
                        .tint(.green)
                        .disabled(isApproved || isRejected)

                        Button {
                            onReject()
                        } label: {
                            Label("Reject", systemImage: "xmark.circle")
                        }
                        .buttonStyle(.bordered)
                        .tint(.red)
                        .disabled(isApproved || isRejected)

                        Spacer()
                    }
                }
            } else {
                Text(patchMeta.diff)
                    .font(.system(.caption, design: .monospaced))
                    .lineLimit(3)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(16)
        .background(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(isSelected ? Color.accentColor.opacity(0.08) : Color.secondary.opacity(0.05))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .stroke(isSelected ? Color.accentColor.opacity(0.3) : Color.clear, lineWidth: 1)
        )
        .onTapGesture {
            onSelect()
        }
    }
}
