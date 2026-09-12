//
//  Components.swift
//  swiftyapp
//

import SwiftUI

var adaptiveColumns: [GridItem] {
    [GridItem(.adaptive(minimum: 220, maximum: 360), spacing: 16, alignment: .top)]
}

struct FlowLayout: Layout {
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

struct DashboardCard<Content: View>: View {
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

struct MetricCard: View {
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

struct MetricPill: View {
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
                    .textSelection(.enabled)
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

struct StatusBadge: View {
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

struct SidebarStatusCard: View {
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
                .textSelection(.enabled)
            Spacer(minLength: 0)
        }
    }
}

struct FileTreeView: View {
    let nodes: [FileNode]
    var onSelect: (String) -> Void

    var body: some View {
        ForEach(nodes) { node in
            FileTreeRow(node: node, onSelect: onSelect)
        }
    }
}

struct FileTreeRow: View {
    let node: FileNode
    var onSelect: (String) -> Void
    @State private var isExpanded = true

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 4) {
                if node.isDirectory {
                    Image(systemName: isExpanded ? "chevron.down" : "chevron.right")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .frame(width: 16)
                        .onTapGesture { isExpanded.toggle() }
                    Image(systemName: "folder")
                        .foregroundStyle(.accent)
                } else {
                    Spacer().frame(width: 16)
                    Image(systemName: "doc.text")
                        .foregroundStyle(.secondary)
                }
                Text(node.name)
                    .font(.system(.body, design: .monospaced))
                Spacer()
            }
            .contentShape(Rectangle())
            .onTapGesture {
                if node.isDirectory {
                    isExpanded.toggle()
                } else {
                    onSelect(node.path)
                }
            }

            if node.isDirectory && isExpanded && !node.children.isEmpty {
                FileTreeView(nodes: node.children, onSelect: onSelect)
                    .padding(.leading, 20)
            }
        }
    }
}
