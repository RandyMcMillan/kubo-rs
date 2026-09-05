//
//  ContentView.swift
//  swiftyapp
//
//  Created by Jonathan McKenzie on 7/9/24.
//

import Foundation
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

struct ContentView: View {
    @StateObject private var store = DashboardStore()

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
        LazyVGrid(columns: adaptiveColumns, spacing: 16) {
            MetricCard(title: "Connectivity", value: "Offline demo node", symbol: "antenna.radiowaves.left.and.right", subtitle: "Keeps the sample self-contained")
            MetricCard(title: "Peer ID", value: store.snapshot.peerID, symbol: "person.circle", subtitle: "Useful when you later connect the GUI to the full node API")
            MetricCard(title: "Shared state", value: "Temporary repo", symbol: "folder.badge.gearshape", subtitle: "Created on demand and cleaned up after use")
            MetricCard(title: "Transport", value: "Local FFI bridge", symbol: "cable.connector", subtitle: "SwiftUI calls into Rust, Rust calls into kubo-rs")
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
