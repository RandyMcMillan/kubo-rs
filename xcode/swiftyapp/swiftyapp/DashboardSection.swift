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
    case activity

    var id: String { rawValue }

    var title: String {
        switch self {
        case .overview: return "Overview"
        case .repos: return "Repos"
        case .repository: return "Repository"
        case .network: return "Network"
        case .chat: return "Chat"
        case .activity: return "Activity"
        }
    }

    var subtitle: String {
        switch self {
        case .overview: return "Status at a glance"
        case .repos: return "All hosted repositories"
        case .repository: return "Local repo snapshot"
        case .network: return "Peer and CID details"
        case .chat: return "Gossip pubsub messages"
        case .activity: return "Recent actions"
        }
    }

    var icon: String {
        switch self {
        case .overview: return "square.grid.2x2"
        case .repos: return "folder.circle"
        case .repository: return "externaldrive.connected.to.line.below"
        case .network: return "point.3.connected.trianglepath.dotted"
        case .chat: return "bubble.left.and.bubble.right"
        case .activity: return "clock.arrow.circlepath"
        }
    }
}
