//
//  Models.swift
//  swiftyapp
//

import Foundation

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

struct FileNode: Identifiable {
    let id = UUID()
    var name: String
    var path: String
    var isDirectory: Bool
    var children: [FileNode]
}

struct BlameLine: Codable {
    let author: String
    let name: String
    let text: String
    let date: Int64
    let hash: String
}

struct BlameResult: Codable {
    let path: String
    let rev: String
    let lines: [BlameLine]
}

struct CommitInfo: Codable, Identifiable {
    let hash: String
    let message: String
    let author: String
    let email: String
    let date: Int64
    var id: String { hash }
}

struct RepoEntry: Identifiable {
    let id = UUID()
    var path: String
    var name: String
    var head: String
    var branch: String
}

struct RelayEntry: Identifiable {
    let id = UUID()
    var url: String
    var handle: UInt64 = 0
    var status: RelayStatus = .disconnected
    var subHandle: UInt64 = 0
}

enum RelayStatus: String {
    case connected = "Connected"
    case connecting = "Connecting"
    case disconnected = "Disconnected"
    case error = "Error"
}

struct TopicEntry: Identifiable {
    let id = UUID()
    var name: String
    var joined: Bool = false
}
