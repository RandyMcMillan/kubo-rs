use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use ratzilla::{DomBackend, WebRenderer};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use gloo_timers::future::IntervalStream;
use futures::StreamExt;

#[derive(Clone, Default)]
struct NodeInfo {
    peer_id: String,
    version: String,
    addresses: Vec<String>,
    connected: bool,
    error: Option<String>,
    api_base: String,
}

fn api_base() -> String {
    web_sys::window()
        .and_then(|w| w.location().href().ok())
        .and_then(|href| web_sys::Url::new(&href).ok())
        .and_then(|url| url.search_params().get("api"))
        .unwrap_or_else(|| {
            let hostname = web_sys::window()
                .and_then(|w| w.location().hostname().ok())
                .filter(|h| !h.is_empty())
                .unwrap_or_else(|| "localhost".to_string());
            format!("http://{}:5001", hostname)
        })
}

fn main() -> io::Result<()> {
    let api_base = api_base();
    let info = Rc::new(RefCell::new(NodeInfo {
        api_base: api_base.clone(),
        ..NodeInfo::default()
    }));
    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;

    // Poll the Kubo HTTP API in the background
    let info_poll = info.clone();
    spawn_local(async move {
        let mut interval = IntervalStream::new(2000);
        while interval.next().await.is_some() {
            poll_api(&info_poll, &api_base).await;
        }
    });

    terminal.draw_web(move |f| {
        let info = info.borrow();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(12),
                Constraint::Length(8),
                Constraint::Min(0),
            ])
            .split(f.area());

        let title = Paragraph::new("Kubo IPFS Dashboard (WASM)")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(title, chunks[0]);

        let status_color = if info.connected { Color::Green } else { Color::Red };
        let status_text = if info.connected {
            format!("Connected to {}", info.api_base)
        } else if let Some(ref err) = info.error {
            format!("Error: {err}")
        } else {
            format!("Connecting to {} ...", info.api_base)
        };

        let status = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                Span::styled(status_text, Style::default().fg(status_color)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Peer ID:  ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if info.peer_id.is_empty() {
                        "—".to_string()
                    } else {
                        info.peer_id.clone()
                    },
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Version:  ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if info.version.is_empty() {
                        "—".to_string()
                    } else {
                        info.version.clone()
                    },
                    Style::default().fg(Color::White),
                ),
            ]),
        ])
        .block(
            Block::default()
                .title(" Node Identity ")
                .borders(Borders::ALL)
                .border_style(Color::Blue),
        )
        .wrap(Wrap { trim: true });
        f.render_widget(status, chunks[1]);

        let addrs_text = if info.addresses.is_empty() {
            "No addresses available".to_string()
        } else {
            info.addresses.join("\n")
        };
        let addrs = Paragraph::new(addrs_text)
            .block(
                Block::default()
                    .title(" Listening Addresses ")
                    .borders(Borders::ALL)
                    .border_style(Color::Magenta),
            )
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        f.render_widget(addrs, chunks[2]);

        let help = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Quickstart:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("git clone https://github.com/RandyMcMillan/kubo-rs.git"),
            Line::from("cd kubo-rs && git submodule update --init --recursive"),
            Line::from("make build-go   # builds go/kubo-sys/cmd/ipfs/ipfs"),
            Line::from("make run-wasm-dashboard   # starts daemon + serves dashboard"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Manual setup:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("1. Start a standard Kubo daemon on port 5001:"),
            Line::from("   ipfs daemon --api /ip4/127.0.0.1/tcp/5001"),
            Line::from("2. Enable CORS:"),
            Line::from("   ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin"),
            Line::from("     '[\"http://localhost:8080\"]'"),
            Line::from("3. Restart daemon and reload this page"),
            Line::from(""),
            Line::from("Note: this dashboard uses the Kubo HTTP API (port 5001)."),
            Line::from("The kubo-rs embedded node does not expose this API."),
            Line::from(""),
            Line::from(vec![
                Span::styled("Custom API:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("Add ?api=<url> to use a different endpoint."),
            Line::from("Example: ?api=http://192.168.1.100:5001"),
        ])
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Color::Gray),
        )
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true });
        f.render_widget(help, chunks[3]);
    });

    Ok(())
}

fn is_mixed_content(api_base: &str) -> bool {
    web_sys::window()
        .and_then(|w| w.location().protocol().ok())
        .map(|proto| proto == "https:" && api_base.starts_with("http:"))
        .unwrap_or(false)
}

async fn poll_api(info: &RefCell<NodeInfo>, api_base: &str) {
    if is_mixed_content(api_base) {
        let mut i = info.borrow_mut();
        i.connected = false;
        i.error = Some(
            "Mixed content blocked: HTTPS page cannot fetch HTTP API. \
             Use http://localhost:8080 from 'trunk serve' instead of GitHub Pages."
                .to_string(),
        );
        return;
    }

    match fetch_id(api_base).await {
        Ok(json) => {
            let mut i = info.borrow_mut();
            i.connected = true;
            i.error = None;
            if let Some(id) = json.get("ID").and_then(|v| v.as_str()) {
                i.peer_id = id.to_string();
            }
            if let Some(ver) = json.get("AgentVersion").and_then(|v| v.as_str()) {
                i.version = ver.to_string();
            }
            if let Some(addrs) = json.get("Addresses").and_then(|v| v.as_array()) {
                i.addresses = addrs
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
        }
        Err(e) => {
            let mut i = info.borrow_mut();
            i.connected = false;
            i.error = Some(
                format!(
                    "Cannot connect to {}/api/v0/id. Error: {e:?}",
                    api_base
                ),
            );
        }
    }
}

async fn fetch_id(api_base: &str) -> Result<serde_json::Value, JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(web_sys::RequestMode::Cors);

    let url = format!("{}/api/v0/id", api_base);
    let request = web_sys::Request::new_with_str_and_init(&url, &opts)?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp_value.dyn_into()?;
    let text = JsFuture::from(resp.text()?).await?;
    let text_str = text.as_string().ok_or("invalid text")?;

    serde_json::from_str(&text_str).map_err(|e| JsValue::from_str(&e.to_string()))
}
