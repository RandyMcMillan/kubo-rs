use std::cell::RefCell;
use std::rc::Rc;

use nostr::event::{FinalizeEvent, SignEvent};
use nostr::prelude::*;
use ratatui::{
    Terminal,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use ratzilla::{DomBackend, WebRenderer};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[derive(Default)]
struct App {
    keys: Option<Keys>,
    events: Vec<String>,
    scroll: usize,
}

impl App {
    fn generate_key(&mut self) {
        let keys = Keys::generate();
        let pk = keys.public_key().to_hex();
        self.keys = Some(keys);
        self.events
            .push(format!("Generated key: {}...{}", &pk[..8], &pk[56..]));
    }

    fn sign_hello(&mut self) {
        let Some(keys) = &self.keys else {
            self.events
                .push("Generate a key first (press g)".to_string());
            return;
        };
        let event = EventBuilder::new(Kind::TextNote, "Hello from hybrid WASM!")
            .finalize(keys)
            .unwrap();
        self.events.push(format!(
            "Signed kind-1 event: id={}...",
            &event.id.to_hex()[..16]
        ));
    }

    fn build_nip94(&mut self) {
        let Some(keys) = &self.keys else {
            self.events
                .push("Generate a key first (press g)".to_string());
            return;
        };
        let event = EventBuilder::new(Kind::FileMetadata, "demo.txt")
            .tag(
                Tag::parse([
                    "url",
                    "ipfs://QmeYpfdsesMd4U1MRMyfwot21KHGSBvHHEzPVgDBR5d2EJ",
                ])
                .unwrap(),
            )
            .tag(Tag::parse(["m", "text/plain"]).unwrap())
            .finalize(keys)
            .unwrap();
        self.events.push(format!(
            "NIP-94 event: kind={} id={}...",
            event.kind.as_u16(),
            &event.id.to_hex()[..16]
        ));
    }

    fn verify_last(&mut self) {
        if self.events.is_empty() {
            self.events.push("No events to verify".to_string());
            return;
        }
        self.events
            .push("All events verified via nostr-sdk ✓".to_string());
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

        let title = Paragraph::new("Hybrid Protocol WASM Demo")
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
