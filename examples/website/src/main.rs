use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use layout::{Flex, Offset};
use ratzilla::{
    event::{KeyCode, KeyEvent},
    ratatui::{
        prelude::*,
        widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    },
    utils::open_url,
    widgets::Hyperlink,
    WebRenderer,
};
use examples_shared::backend::{BackendType, MultiBackendBuilder};
use tachyonfx::{
    fx::{self, RepeatMode},
    CenteredShrink, Duration, Effect, EffectRenderer, EffectTimer, Interpolation, Motion, 
};
use ratzilla::backend::webgl2::{SelectionMode, WebGl2BackendOptions};
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
        .unwrap_or_else(|| "http://127.0.0.1:5001".to_string())
}

struct State {
    intro_effect: Effect,
    menu_effect: Effect,
    info: Rc<RefCell<NodeInfo>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            intro_effect: fx::sequence(&[
                fx::ping_pong(fx::sweep_in(
                    Motion::LeftToRight,
                    10,
                    0,
                    Color::Black,
                    EffectTimer::from_ms(3000, Interpolation::QuadIn),
                )),
                fx::coalesce((3000, Interpolation::SineOut)),
                fx::sleep(1000),
                fx::repeat(
                    fx::hsl_shift(
                        Some([120.0, 25.0, 25.0]),
                        None,
                        (5000, Interpolation::Linear),
                    ),
                    RepeatMode::Forever,
                ),
            ]),
            menu_effect: fx::sequence(&[
                fx::coalesce((3000, Interpolation::SineOut)),
                fx::sleep(1000),
            ]),
            info: Rc::new(RefCell::new(NodeInfo::default())),
        }
    }
}

fn main() -> io::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    
    let mut terminal = MultiBackendBuilder::with_fallback(BackendType::Dom)
        .webgl2_options(WebGl2BackendOptions::new()
            .enable_hyperlinks()
            .enable_mouse_selection_with_mode(SelectionMode::default())
        )
        .build_terminal()?;

    let mut state = State::default();
    let api_base = api_base();
    state.info.borrow_mut().api_base = api_base.clone();

    let info_poll = state.info.clone();
    spawn_local(async move {
        let mut interval = IntervalStream::new(2000);
        while interval.next().await.is_some() {
            poll_api(&info_poll, &api_base).await;
        }
    });

    terminal.on_key_event(move |key| handle_key_event(key))?;
    terminal.draw_web(move |f| ui(f, &mut state));
    Ok(())
}

fn ui(f: &mut Frame<'_>, state: &mut State) {
    render_intro(f, state);
    //if state.intro_effect.running() {
    //    render_intro(f, state);
    //} else {
    //    render_menu(f, state);
    //}
}

fn handle_key_event(key: KeyEvent) {
    match key.code {
        KeyCode::Char('g') => {
            open_url("https://github.com/ratatui/ratzilla", true).unwrap();
        }
        KeyCode::Char('d') => {
            open_url("https://ratatui.github.io/ratzilla/demo", false).unwrap();
        }
        _ => {}
    }
}

fn _render_text(f: &mut Frame<'_>, state: &mut State) {

    Clear.render(f.area(), f.buffer_mut());
    let area = f.area().centered(Constraint::Length(33), Constraint::Length(10));
    let main_text = Text::from(vec![
        Line::from("| R A T Z I L L A |").bold(),
        Line::from("Stomping through the web").italic(),
    ]);
    f.render_widget(main_text.light_green().centered(), area);
    let link = Hyperlink::new("https://github.com/ratatui/ratzilla".red());
    f.render_widget(link, area.offset(Offset { x: 0, y: 4 }));
    f.render_effect(&mut state.intro_effect, area, Duration::from_millis(40));
}
fn render_intro(f: &mut Frame<'_>, state: &mut State) {

   //_render_text(f, state);

    //Clear.render(f.area(), f.buffer_mut());
    let _area = f.area().centered(Constraint::Length(33), Constraint::Length(10));
    //let main_text = Text::from(vec![
    //    Line::from("| R A T Z I L L A |").bold(),
    //    Line::from("Stomping through the web").italic(),
    //]);
    //f.render_widget(main_text.light_green().centered(), area);
    //let link = Hyperlink::new("https://github.com/ratatui/ratzilla".red());
    //f.render_widget(link, area.offset(Offset { x: 0, y: 4 }));
    //f.render_effect(&mut state.intro_effect, area, Duration::from_millis(40));


    let info = state.info.borrow();


	//your pallette
	//your pallette
	//your pallette


    let ipfs_area = Layout::vertical([
        Constraint::Percentage(2),
        Constraint::Percentage(96),
        Constraint::Percentage(2),
    ]).split(f.area())[1];


	//your pallette end
	//your pallette end
	//your pallette end


    let status_text = if info.connected {
        format!("Connected to {}", info.api_base)
    } else if let Some(ref err) = info.error {
        format!("Error: {err}")
    } else {
        format!("Connecting to {} ...", info.api_base)
    };

    let status = Paragraph::new(vec![
        Line::from(vec![Span::raw("Status: "), Span::raw(status_text)]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Peer ID:  "),
            Span::raw(if info.peer_id.is_empty() { "-".to_string() } else { info.peer_id.clone() }),
        ]),
        Line::from(vec![
            Span::raw("Version:  "),
            Span::raw(if info.version.is_empty() { "-".to_string() } else { info.version.clone() }),
        ]),
    ])
    .block(Block::default().title(" Node Identity ").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    let addrs_text = if info.addresses.is_empty() {
        "No addresses available".to_string()
    } else {
        info.addresses.join("\n")
    };
    let addrs = Paragraph::new(addrs_text)
        .block(Block::default().title(" Listening Addresses ").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    let chunks = Layout::horizontal(
		[
		//first in
		Constraint::Percentage(2), //spacing correction




		//your pallette
		//your pallette
		//your pallette
		//your pallette
		Constraint::Percentage(50),//spacing correction
		Constraint::Percentage(60) //spacing correction
		//your pallette end
		//your pallette end
		//your pallette end
		//your pallette end




		]
		)
        .split(ipfs_area);

    f.render_widget(status, chunks[1]);
    f.render_widget(addrs, chunks[2]);
}

fn render_menu(f: &mut Frame<'_>, state: &mut State) {
    let vertical = Layout::vertical([Constraint::Percentage(20)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(20)]).flex(Flex::Center);
    let [area] = vertical.areas(f.area());
    let [area] = horizontal.areas(area);

    let text = Text::from(vec![
        Line::default(),
        Line::from(vec![
            "[".into(),
            "g".light_green(),
            "] GitHub Repository".into(),
        ]),
        Line::from(vec!["[".into(), "d".light_green(), "] Demo".into()]),
    ]);

    //f.render_widget(
    //    Paragraph::new(text)
    //        .alignment(Alignment::Center)
    //        .wrap(Wrap { trim: false })
    //        .block(
    //            Block::bordered()
    //                .border_type(BorderType::Rounded)
    //                .title(" Welcome to Ratzilla ")
    //                .title_alignment(Alignment::Center),
    //        ),
    //    area,
    //);
    //f.render_effect(&mut state.menu_effect, area, Duration::from_millis(100));
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
            i.error = Some(format!(
                "Cannot connect to {}/api/v0/id. Error: {e:?}",
                api_base
            ));
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
