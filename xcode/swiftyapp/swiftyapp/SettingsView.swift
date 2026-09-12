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
                MetricCard(title: "Relay", value: store.relayHandle != 0 ? "Connected" : "Disconnected", symbol: "network", subtitle: "Nostr relay state")
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

                    if !store.nodeError.isEmpty {
                        Text(store.nodeError)
                            .font(.caption)
                            .foregroundStyle(.red)
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
                            if store.relayHandle != 0 {
                                nostrRelayClose(handle: store.relayHandle)
                                store.relayHandle = 0
                                store.appendActivity("Relay disconnected")
                            }
                        } label: {
                            Label("Disconnect", systemImage: "network.slash")
                        }
                        .buttonStyle(.bordered)
                        .disabled(store.relayHandle == 0)

                        Spacer()
                    }
                }
            }

            DashboardCard(title: "GossipSub Topic") {
                VStack(alignment: .leading, spacing: 12) {
                    TextField("Topic name", text: $store.gossipTopic)
                        .textFieldStyle(.roundedBorder)
                    Text("Current topic: \(store.gossipTopic)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
        }
    }
}
