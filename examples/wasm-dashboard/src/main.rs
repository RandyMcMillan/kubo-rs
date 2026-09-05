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

#[derive(Clone, Copy, PartialEq, Default)]
enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    fn detect_system() -> Self {
        let dark = web_sys::window()
            .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
            .map(|m| m.matches())
            .unwrap_or(true);
        if dark { Theme::Dark } else { Theme::Light }
    }

    fn toggle(self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }

    // Solarized palette
    fn bg(self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(0, 43, 54),    // base03
            Theme::Light => Color::Rgb(253, 246, 227), // base3
        }
    }
    fn fg(self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(131, 148, 150), // base0
            Theme::Light => Color::Rgb(101, 123, 131), // base00
        }
    }
    fn fg_secondary(self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(147, 161, 161), // base1
            Theme::Light => Color::Rgb(88, 110, 117),  // base01
        }
    }
    fn accent_yellow(self) -> Color { Color::Rgb(181, 137, 0) }
    fn accent_orange(self) -> Color { Color::Rgb(203, 75, 22) }
    fn accent_red(self) -> Color { Color::Rgb(220, 50, 47) }
    fn accent_magenta(self) -> Color { Color::Rgb(211, 54, 130) }
    fn accent_violet(self) -> Color { Color::Rgb(108, 113, 196) }
    fn accent_blue(self) -> Color { Color::Rgb(38, 139, 210) }
    fn accent_cyan(self) -> Color { Color::Rgb(42, 161, 152) }
    fn accent_green(self) -> Color { Color::Rgb(133, 153, 0) }
}

#[derive(Clone, Default)]
struct NodeInfo {
    peer_id: String,
    version: String,
    addresses: Vec<String>,
    connected: bool,
    error: Option<String>,
    api_base: String,
    tab: usize,
    theme: Theme,
}

fn api_base() -> String {
    web_sys::window()
        .and_then(|w| w.location().href().ok())
        .and_then(|href| web_sys::Url::new(&href).ok())
        .and_then(|url| url.search_params().get("api"))
        .unwrap_or_else(|| "http://127.0.0.1:5001".to_string())
}

fn apply_theme_to_body(theme: Theme) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(body) = document.body() {
                match theme {
                    Theme::Light => {
                        let _ = body.class_list().add_1("light");
                    }
                    Theme::Dark => {
                        let _ = body.class_list().remove_1("light");
                    }
                }
            }
        }
    }
}

fn listen_system_theme(info: Rc<RefCell<NodeInfo>>) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(media)) = window.match_media("(prefers-color-scheme: dark)") {
            let info_media = info.clone();
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let theme = Theme::detect_system();
                info_media.borrow_mut().theme = theme;
                apply_theme_to_body(theme);
            }) as Box<dyn FnMut(_)>);
            let _ = media.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    }
}

fn maybe_show_gh_pages_badge() {
    if let Some(window) = web_sys::window() {
        if let Ok(hostname) = window.location().hostname() {
            let is_gh_pages = hostname == "kubo-rs.randymcmillan.net"
                || hostname.ends_with(".github.io");
            if is_gh_pages {
                if let Some(document) = window.document() {
                    if let Some(body) = document.body() {
                        let div = document.create_element("div").unwrap();
                        let _ = div.set_attribute(
                            "style",
                            "position:fixed;top:8px;right:8px;z-index:1000;",
                        );
                        let a = document.create_element("a").unwrap();
                        let _ = a.set_attribute(
                            "href",
                            "https://github.com/RandyMcMillan/kubo-rs/actions/workflows/gh-pages.yml",
                        );
                        let _ = a.set_attribute("target", "_blank");
                        let img = document.create_element("img").unwrap();
                        let _ = img.set_attribute(
                            "src",
                            "https://github.com/RandyMcMillan/kubo-rs/actions/workflows/gh-pages.yml/badge.svg",
                        );
                        let _ = img.set_attribute("alt", "GitHub Pages");
                        let _ = img.set_attribute("style", "height:20px;border-radius:4px;");
                        let _ = a.append_child(&img);
                        let _ = div.append_child(&a);
                        let _ = body.append_child(&div);
                    }
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    maybe_show_gh_pages_badge();

    let api_base = api_base();
    let initial_theme = Theme::detect_system();
    apply_theme_to_body(initial_theme);

    let info = Rc::new(RefCell::new(NodeInfo {
        api_base: api_base.clone(),
        tab: 0,
        theme: initial_theme,
        ..NodeInfo::default()
    }));

    listen_system_theme(info.clone());

    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;

    // Keyboard: 1/2/3 for tabs, t for theme toggle
    let info_keys = info.clone();
    let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        match event.key().as_str() {
            "1" => info_keys.borrow_mut().tab = 0,
            "2" => info_keys.borrow_mut().tab = 1,
            "3" => info_keys.borrow_mut().tab = 2,
            "4" => info_keys.borrow_mut().tab = 3,
            "Tab" => {
                let mut i = info_keys.borrow_mut();
                i.tab = (i.tab + 1) % 4;
            }
            "t" | "T" => {
                let new_theme = info_keys.borrow().theme.toggle();
                info_keys.borrow_mut().theme = new_theme;
                apply_theme_to_body(new_theme);
            }
            _ => {}
        }
    }) as Box<dyn FnMut(_)>);
    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();

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
        let t = info.theme;
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

        let title_text = format!(
            "Kubo IPFS Dashboard (WASM)  [press 't' for {} mode]",
            if t == Theme::Dark { "light" } else { "dark" }
        );
        let title = Paragraph::new(title_text)
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(t.accent_cyan())
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::BOTTOM).border_style(t.fg_secondary()));
        f.render_widget(title, chunks[0]);

        let status_color = if info.connected {
            t.accent_green()
        } else {
            t.accent_red()
        };
        let status_text = if info.connected {
            format!("Connected to {}", info.api_base)
        } else if let Some(ref err) = info.error {
            format!("Error: {err}")
        } else {
            format!("Connecting to {} ...", info.api_base)
        };

        let status = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(t.accent_yellow())),
                Span::styled(status_text, Style::default().fg(status_color)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Peer ID:  ", Style::default().fg(t.accent_yellow())),
                Span::styled(
                    if info.peer_id.is_empty() {
                        "—".to_string()
                    } else {
                        info.peer_id.clone()
                    },
                    Style::default().fg(t.fg()),
                ),
            ]),
            Line::from(vec![
                Span::styled("Version:  ", Style::default().fg(t.accent_yellow())),
                Span::styled(
                    if info.version.is_empty() {
                        "—".to_string()
                    } else {
                        info.version.clone()
                    },
                    Style::default().fg(t.fg()),
                ),
            ]),
        ])
        .block(
            Block::default()
                .title(" Node Identity ")
                .borders(Borders::ALL)
                .border_style(t.accent_blue()),
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
                    .border_style(t.accent_violet()),
            )
            .style(Style::default().fg(t.fg()))
            .wrap(Wrap { trim: true });
        f.render_widget(addrs, chunks[2]);

        let tab_style = |n: usize, label: &str| {
            if info.tab == n {
                Span::styled(
                    format!(" [{}] ", label),
                    Style::default()
                        .fg(t.bg())
                        .bg(t.accent_yellow())
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    format!("  {}  ", label),
                    Style::default().fg(t.fg_secondary()),
                )
            }
        };

        let tabs = Paragraph::new(Line::from(vec![
            tab_style(0, "1 Quickstart"),
            tab_style(1, "2 HTTPS"),
            tab_style(2, "3 Custom API"),
            tab_style(3, "4 Plugins"),
        ]));

        let help_content: Vec<Line> = match info.tab {
            1 => vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("HTTPS on GitHub Pages", Style::default().fg(t.accent_yellow()).add_modifier(Modifier::BOLD)),
                ]),
                Line::from("Browsers block HTTP API calls from HTTPS pages."),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Option A: Local HTTPS proxy (all browsers)", Style::default().fg(t.accent_cyan()).add_modifier(Modifier::BOLD)),
                ]),
                Line::from("1. Install Caddy and mkcert:"),
                Line::from("   brew install caddy mkcert"),
                Line::from("2. Create a trusted local cert:"),
                Line::from("   mkcert -install && mkcert localhost 127.0.0.1 ::1"),
                Line::from("3. Run Caddy reverse proxy:"),
                Line::from("   caddy reverse-proxy --from localhost:5443 --to 127.0.0.1:5001"),
                Line::from("4. Open dashboard with HTTPS API:"),
                Line::from("   ?api=https://localhost:5443"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Option B: Firefox", Style::default().fg(t.accent_cyan()).add_modifier(Modifier::BOLD)),
                ]),
                Line::from("Firefox shows a permission prompt for mixed content."),
                Line::from("Allow it when prompted to connect to the HTTP API."),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Option C: Local HTTP (no HTTPS needed)", Style::default().fg(t.accent_cyan()).add_modifier(Modifier::BOLD)),
                ]),
                Line::from("make run-wasm-dashboard"),
                Line::from("Serves on http://localhost:8080 — no mixed content."),
            ],
            2 => vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("Custom API Endpoint", Style::default().fg(t.accent_yellow()).add_modifier(Modifier::BOLD)),
                ]),
                Line::from("Override the default API with a query parameter:"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Examples:", Style::default().fg(t.accent_cyan())),
                ]),
                Line::from("  ?api=http://192.168.1.100:5001"),
                Line::from("  ?api=https://localhost:5443"),
                Line::from("  ?api=http://127.0.0.1:5001"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Current:", Style::default().fg(t.accent_cyan())),
                ]),
                Line::from(format!("  {}", info.api_base)),
                Line::from(""),
                Line::from("The kubo-rs embedded node does not expose the HTTP API."),
                Line::from("Use a standard Kubo daemon for dashboard connectivity."),
            ],
            3 => vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("Browser Extensions", Style::default().fg(t.accent_yellow()).add_modifier(Modifier::BOLD)),
                ]),
                Line::from("IPFS Companion bridges HTTPS pages to your local IPFS node."),
                Line::from("Verified official extension:"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Chrome / Edge", Style::default().fg(t.accent_cyan())),
                ]),
                Line::from("  chromewebstore.google.com/detail/ipfs-companion"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Firefox", Style::default().fg(t.accent_cyan())),
                ]),
                Line::from("  addons.mozilla.org/firefox/addon/ipfs-companion"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Source", Style::default().fg(t.accent_cyan())),
                ]),
                Line::from("  github.com/ipfs/ipfs-companion"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Brave Browser", Style::default().fg(t.accent_cyan())),
                ]),
                Line::from("  Native IPFS support built-in."),
            ],
            _ => vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("Quickstart:", Style::default().fg(t.accent_yellow()).add_modifier(Modifier::BOLD)),
                ]),
                Line::from("git clone https://github.com/RandyMcMillan/kubo-rs.git"),
                Line::from("cd kubo-rs && git submodule update --init --recursive"),
                Line::from("make build-go   # builds go/kubo-sys/cmd/ipfs/ipfs"),
                Line::from("make run-wasm-dashboard   # starts daemon + serves dashboard"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Manual setup:", Style::default().fg(t.accent_yellow()).add_modifier(Modifier::BOLD)),
                ]),
                Line::from("1. Start a standard Kubo daemon on port 5001:"),
                Line::from("   ipfs daemon --api /ip4/127.0.0.1/tcp/5001"),
                Line::from("2. Enable CORS:"),
                Line::from("   ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin"),
                Line::from("     '[\"http://localhost:8080\"]'"),
                Line::from("3. Restart daemon and reload this page"),
            ],
        };

        let help = Paragraph::new(help_content)
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(t.fg_secondary()),
            )
            .style(Style::default().fg(t.fg_secondary()))
            .wrap(Wrap { trim: true });

        let help_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(chunks[3]);

        f.render_widget(tabs, help_chunks[0]);
        f.render_widget(help, help_chunks[1]);
    });

    Ok(())
}

async fn poll_api(info: &RefCell<NodeInfo>, api_base: &str) {
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
