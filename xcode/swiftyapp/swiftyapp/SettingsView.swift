//
//  SettingsView.swift
//  swiftyapp
//

import SwiftUI
import RustyLib

struct SettingsView: View {
    @ObservedObject var store: HybridNodeStore
    @State private var newRelayURL: String = ""
    @State private var newTopic: String = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 220))], spacing: 16) {
                MetricCard(title: "Node status", value: store.nodeStarted ? "Online" : "Offline", symbol: "power.circle", subtitle: "HybridNode state")
                MetricCard(title: "Peer ID", value: store.snapshot.peerID, symbol: "person.crop.circle", subtitle: "IPFS identity")
                let connectedRelays = store.relays.filter { $0.status == .connected }.count
                MetricCard(title: "Relays", value: "\(connectedRelays)/\(store.relays.count)", symbol: "network", subtitle: "Nostr relay connections")
            }

            DashboardCard(title: "Identity") {
                VStack(alignment: .leading, spacing: 12) {
                    if store.nostrPublicKey.isEmpty {
                        Text("No Nostr key generated.")
                            .foregroundStyle(.secondary)
                        Button {
                            store.generateNostrKey()
                        } label: {
                            Label("Generate key", systemImage: "key")
                        }
                        .buttonStyle(.borderedProminent)
                    } else {
                        Text("Public Key")
                            .font(.headline)
                        Text(store.nostrPublicKey)
                            .font(.system(.body, design: .monospaced))
                            .textSelection(.enabled)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        Text("Secret Key")
                            .font(.headline)
                            .padding(.top, 4)
                        Text(store.nostrSecretKey)
                            .font(.system(.caption, design: .monospaced))
                            .textSelection(.enabled)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
            }

            DashboardCard(title: "Node Control") {
                VStack(alignment: .leading, spacing: 12) {
                    HStack(spacing: 12) {
                        Button {
                            store.startNode()
                        } label: {
                            Label("Start", systemImage: "play.circle")
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(store.nodeStarted)

                        Button {
                            store.stopNode()
                        } label: {
                            Label("Stop", systemImage: "stop.circle")
                        }
                        .buttonStyle(.bordered)
                        .disabled(!store.nodeStarted)

                        Spacer()
                    }

                    Toggle("Background polling", isOn: Binding(
                        get: { store.isPolling },
                        set: { _ in store.togglePolling() }
                    ))
                    .font(.caption)

                    if !store.nodeError.isEmpty {
                        Text(store.nodeError)
                            .font(.caption)
                            .foregroundStyle(.red)
                    }
                }
            }

            DashboardCard(title: "Relays") {
                VStack(alignment: .leading, spacing: 12) {
                    HStack(spacing: 12) {
                        TextField("wss://relay.example.com", text: $newRelayURL)
                            .textFieldStyle(.roundedBorder)
                        Button {
                            store.addRelay(url: newRelayURL)
                            newRelayURL = ""
                        } label: {
                            Label("Add", systemImage: "plus.circle")
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(newRelayURL.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                    }

                    if store.relays.isEmpty {
                        Text("No relays configured. Add a relay to connect to the Nostr network.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(store.relays) { relay in
                            HStack(spacing: 12) {
                                Circle()
                                    .fill(relay.status == .connected ? Color.green : (relay.status == .error ? Color.red : Color.orange))
                                    .frame(width: 8, height: 8)
                                VStack(alignment: .leading, spacing: 2) {
                                    Text(relay.url)
                                        .font(.system(.body, design: .monospaced))
                                        .lineLimit(1)
                                    Text(relay.status.rawValue)
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                }
                                Spacer()
                                if relay.status != .connected {
                                    Button {
                                        store.connectRelayEntry(id: relay.id)
                                    } label: {
                                        Image(systemName: "arrow.clockwise")
                                    }
                                    .buttonStyle(.borderless)
                                }
                                Button {
                                    store.removeRelay(id: relay.id)
                                } label: {
                                    Image(systemName: "trash")
                                        .foregroundStyle(.red)
                                }
                                .buttonStyle(.borderless)
                            }
                            .padding(.vertical, 4)
                            if relay.id != store.relays.last?.id {
                                Divider()
                            }
                        }
                    }

                    HStack(spacing: 12) {
                        Button {
                            store.reconnectAllRelays()
                        } label: {
                            Label("Reconnect all", systemImage: "arrow.clockwise")
                        }
                        .buttonStyle(.bordered)
                        .disabled(store.relays.isEmpty)

                        Spacer()
                    }
                }
            }

            DashboardCard(title: "GossipSub Topics") {
                VStack(alignment: .leading, spacing: 12) {
                    HStack(spacing: 12) {
                        TextField("topic-name", text: $newTopic)
                            .textFieldStyle(.roundedBorder)
                        Button {
                            store.addTopic(name: newTopic)
                            newTopic = ""
                        } label: {
                            Label("Add", systemImage: "plus.circle")
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(newTopic.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                    }

                    if store.topics.isEmpty {
                        Text("No topics configured. Add a topic to join GossipSub conversations.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    } else {
                        ForEach(store.topics) { topic in
                            HStack(spacing: 12) {
                                Circle()
                                    .fill(topic.joined ? Color.green : Color.secondary.opacity(0.3))
                                    .frame(width: 8, height: 8)
                                Text(topic.name)
                                    .font(.system(.body, design: .monospaced))
                                Spacer()
                                if topic.joined {
                                    Button {
                                        store.leaveTopic(id: topic.id)
                                    } label: {
                                        Label("Leave", systemImage: "xmark.circle")
                                    }
                                    .buttonStyle(.bordered)
                                    .controlSize(.small)
                                } else {
                                    Button {
                                        store.joinTopic(name: topic.name)
                                    } label: {
                                        Label("Join", systemImage: "checkmark.circle")
                                    }
                                    .buttonStyle(.borderedProminent)
                                    .controlSize(.small)
                                }
                                Button {
                                    store.removeTopic(id: topic.id)
                                } label: {
                                    Image(systemName: "trash")
                                        .foregroundStyle(.red)
                                }
                                .buttonStyle(.borderless)
                            }
                            .padding(.vertical, 4)
                            if topic.id != store.topics.last?.id {
                                Divider()
                            }
                        }
                    }
                }
            }
        }
    }
}
