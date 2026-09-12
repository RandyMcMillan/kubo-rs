//
//  DashboardSection.swift
//  swiftyapp
//

import SwiftUI

enum DashboardSection: String, CaseIterable, Identifiable {
    case overview
    case repos
    case repository
    case network
    case chat
    case codeReview
    case issueTracker
    case repoDiscovery
    case activity
    case settings

    var id: String { rawValue }

    var title: String {
        switch self {
        case .overview: return "Overview"
        case .repos: return "Repos"
        case .repository: return "Repository"
        case .network: return "Network"
        case .chat: return "Chat"
        case .codeReview: return "Code Review"
        case .issueTracker: return "Issues"
        case .repoDiscovery: return "Repos"
        case .activity: return "Activity"
        case .settings: return "Settings"
        }
    }

    var subtitle: String {
        switch self {
        case .overview: return "Status at a glance"
        case .repos: return "All hosted repositories"
        case .repository: return "Local repo snapshot"
        case .network: return "Peer and CID details"
        case .chat: return "Gossip pubsub messages"
        case .codeReview: return "kind:1617 patch review"
        case .issueTracker: return "kind:1621 issue tracking"
        case .repoDiscovery: return "kind:30617 repo discovery"
        case .activity: return "Recent actions"
        case .settings: return "Relays, topics, identity"
        }
    }

    var icon: String {
        switch self {
        case .overview: return "square.grid.2x2"
        case .repos: return "folder.circle"
        case .repository: return "externaldrive.connected.to.line.below"
        case .network: return "point.3.connected.trianglepath.dotted"
        case .chat: return "bubble.left.and.bubble.right"
        case .codeReview: return "doc.text.magnifyingglass"
        case .issueTracker: return "exclamationmark.bubble"
        case .repoDiscovery: return "globe"
        case .activity: return "clock.arrow.circlepath"
        case .settings: return "gear"
        }
    }
}
