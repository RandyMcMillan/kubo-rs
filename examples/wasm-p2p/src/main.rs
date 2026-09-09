use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use futures::StreamExt;
use libp2p_core::{muxing::StreamMuxerBox, upgrade::Version, Transport};
use libp2p_identity::Keypair;
use libp2p_swarm::{Config, NetworkBehaviour, Swarm, SwarmEvent};
use ratatui::{
    Terminal,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use ratzilla::{DomBackend, WebRenderer};
use wasm_bindgen_futures::spawn_local;

#[derive(NetworkBehaviour)]
#[behaviour(prelude = "libp2p_swarm::derive_prelude")]
struct Behaviour {
    ping: libp2p_ping::Behaviour,
}

#[derive(Clone, Default)]
struct AppState {
    peer_id: String,
    listen_addrs: Vec<String>,
    logs: Vec<String>,
}

fn main() -> io::Result<()> {
    let keypair = Keypair::generate_ed25519();
    let peer_id = keypair.public().to_peer_id();

    let ws = libp2p_websocket_websys::Transport::default()
        .upgrade(Version::V1)
        .authenticate(libp2p_noise::Config::new(&keypair).unwrap())
        .multiplex(libp2p_yamux::Config::default())
        .map(|(peer_id, muxer), _| (peer_id, StreamMuxerBox::new(muxer)))
        .boxed();

    let webrtc = libp2p_webrtc_websys::Transport::new(
        libp2p_webrtc_websys::Config::new(&keypair),
    )
    .boxed();

    let transport = ws.or_transport(webrtc).map(|either, _| either.into_inner()).boxed();

    let behaviour = Behaviour {
        ping: libp2p_ping::Behaviour::new(libp2p_ping::Config::new()),
    };

    let mut swarm = Swarm::new(transport, behaviour, peer_id, Config::with_wasm_executor());

    let _ = swarm.listen_on("/dns4/localhost/tcp/0/ws".parse().unwrap());

    let state = Rc::new(RefCell::new(AppState {
        peer_id: peer_id.to_string(),
        ..AppState::default()
    }));

    let state_events = state.clone();
    spawn_local(async move {
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    let addr = address.to_string();
                    web_sys::console::log_1(&format!("Listening on {}", addr).into());
                    state_events.borrow_mut().listen_addrs.push(addr);
                }
                SwarmEvent::Behaviour(BehaviourEvent::Ping(event)) => {
                    let msg = format!("Ping from {:?}: {:?}", event.peer, event.result);
                    web_sys::console::log_1(&msg.clone().into());
                    state_events.borrow_mut().logs.push(msg);
                }
                SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                    let msg = format!(
                        "Connected to {} via {}",
                        peer_id,
                        endpoint.get_remote_address()
                    );
                    web_sys::console::log_1(&msg.clone().into());
                    state_events.borrow_mut().logs.push(msg);
                }
                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    let msg = format!("Disconnected from {}: {:?}", peer_id, cause);
                    web_sys::console::log_1(&msg.clone().into());
                    state_events.borrow_mut().logs.push(msg);
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    let msg = format!("Outgoing error to {:?}: {}", peer_id, error);
                    web_sys::console::log_1(&msg.clone().into());
                    state_events.borrow_mut().logs.push(msg);
                }
                _ => {}
            }
        }
    });

    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;

    terminal.draw_web(move |f| {
        let s = state.borrow();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(6),
                Constraint::Length(8),
                Constraint::Min(0),
            ])
            .split(f.area());

        let title = Paragraph::new("libp2p WebRTC + WebSocket (WASM)")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(title, chunks[0]);

        let status = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Peer ID: ", Style::default().fg(Color::Yellow)),
                Span::styled(s.peer_id.clone(), Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Listening: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if s.listen_addrs.is_empty() {
                        "starting...".to_string()
                    } else {
                        s.listen_addrs.join(", ")
                    },
                    Style::default().fg(Color::White),
                ),
            ]),
        ])
        .block(Block::default().title(" Node ").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
        f.render_widget(status, chunks[1]);

        let addrs_text = if s.logs.is_empty() {
            "No events yet. Use the browser console to see detailed logs.\n\nTo test:\n1. Open this page in two browser tabs\n2. Copy the Peer ID from one tab\n3. Use console to dial: swarm.dial(...)"
                .to_string()
        } else {
            s.logs.join("\n")
        };
        let logs = Paragraph::new(addrs_text)
            .block(Block::default().title(" Events ").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        f.render_widget(logs, chunks[2]);

        let help = Paragraph::new(vec![
            Line::from(""),
            Line::from("Pure Rust libp2p in the browser — no CGO required."),
            Line::from("WebSocket transport listens; WebRTC dials peers."),
        ])
        .block(Block::default().title(" Info ").borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true });
        f.render_widget(help, chunks[3]);
    });

    Ok(())
}
