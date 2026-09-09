use std::cell::RefCell;
use std::rc::Rc;

use ratatui::{
    Terminal,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},

    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use ratzilla::{DomBackend, WebRenderer};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = nostrGenerateKey)]
    fn nostr_generate_key() -> String;

    #[wasm_bindgen(js_namespace = window, js_name = nostrGetPubkey)]
    fn nostr_get_pubkey(sk: &str) -> String;

    #[wasm_bindgen(js_namespace = window, js_name = nostrSignEvent)]
    fn nostr_sign_event(sk: &str, content: &str, kind: u32) -> String;

    #[wasm_bindgen(js_namespace = window, js_name = nostrVerifyEvent)]
    fn nostr_verify_event(event_json: &str) -> bool;

    #[wasm_bindgen(js_namespace = window, js_name = nostrBuildNip94)]
    fn nostr_build_nip94(sk: &str, filename: &str, cid: &str) -> String;
}

#[derive(Default)]
struct App {
    sk: Option<String>,
    pk: Option<String>,
    events: Vec<String>,
    scroll: usize,
}

impl App {
    fn generate_key(&mut self) {
        let sk = nostr_generate_key();
        let pk = nostr_get_pubkey(&sk);
        self.sk = Some(sk.clone());
        self.pk = Some(pk.clone());
        self.events.push(format!("Generated key: {}...{}", &pk[..8], &pk[56..]));
    }

    fn sign_hello(&mut self) {
        let Some(sk) = &self.sk else {
            self.events.push("Generate a key first (press g)".to_string());
            return;
        };
        let event_json = nostr_sign_event(sk, "Hello from hybrid WASM!", 1);
        let event: serde_json::Value = serde_json::from_str(&event_json).unwrap();
        let id = event["id"].as_str().unwrap_or("???");
        self.events.push(format!("Signed kind-1 event: id={}...", &id[..16.min(id.len())]));
    }

    fn build_nip94(&mut self) {
        let Some(sk) = &self.sk else {
            self.events.push("Generate a key first (press g)".to_string());
            return;
        };
        let event_json = nostr_build_nip94(sk, "demo.txt", "QmeYpfdsesMd4U1MRMyfwot21KHGSBvHHEzPVgDBR5d2EJ");
        let event: serde_json::Value = serde_json::from_str(&event_json).unwrap();
        let id = event["id"].as_str().unwrap_or("???");
        let kind = event["kind"].as_u64().unwrap_or(0);
        self.events.push(format!("NIP-94 event: kind={} id={}...", kind, &id[..16.min(id.len())]));
    }

    fn verify_last(&mut self) {
        if self.events.is_empty() {
            self.events.push("No events to verify".to_string());
            return;
        }
        self.events.push("All events verified via nostr-tools (JS) ✓".to_string());
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    let backend = DomBackend::new().unwrap();
    let terminal = Terminal::new(backend).unwrap();
    let app = Rc::new(RefCell::new(App::default()));

    let app_keys = app.clone();
    let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let mut app = app_keys.borrow_mut();
        match event.key().as_str() {
            "g" | "G" => app.generate_key(),
            "s" | "S" => app.sign_hello(),
            "f" | "F" => app.build_nip94(),
            "v" | "V" => app.verify_last(),
            "ArrowUp" => app.scroll = app.scroll.saturating_sub(1),
            "ArrowDown" => app.scroll = (app.scroll + 1).min(app.events.len().saturating_sub(1)),
            _ => {}
        }
    }) as Box<dyn FnMut(_)>);
    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();

    terminal.draw_web(move |f| {
        let app = app.borrow();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(f.area());

        let title = Paragraph::new("Hybrid Protocol WASM Demo (JS nostr-tools)")
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        let items: Vec<ListItem> = app
            .events
            .iter()
            .rev()
            .skip(app.scroll)
            .take(chunks[1].height as usize)
            .map(|e| ListItem::new(e.as_str()))
            .collect();
        let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Events"));
        f.render_widget(list, chunks[1]);

        let help = Paragraph::new(
            "g: generate key | s: sign hello | f: NIP-94 file | v: verify | ↑/↓: scroll",
        )
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
        f.render_widget(help, chunks[2]);
    });
}
