use std::env;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
};

use kubo_rs::{Node, init_repo};

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Status,
    Files,
    Peers,
    Network,
    Blocks,
    Logs,
}

impl Tab {
    fn title(self) -> &'static str {
        match self {
            Tab::Status => "Status",
            Tab::Files => "Files",
            Tab::Peers => "Peers",
            Tab::Network => "Network",
            Tab::Blocks => "Blocks",
            Tab::Logs => "Logs",
        }
    }

    fn all() -> Vec<Tab> {
        vec![
            Tab::Status,
            Tab::Files,
            Tab::Peers,
            Tab::Network,
            Tab::Blocks,
            Tab::Logs,
        ]
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Modal {
    None,
    AddContent,
    ConnectPeer,
    ViewContent,
}

struct Dashboard {
    node: Node,
    peer_id: String,
    version: String,
    repo_path: PathBuf,
    tab: Tab,
    addrs: Vec<String>,
    peers: Vec<(String, String)>,
    files: Vec<(String, String, usize)>,
    blocks: Vec<(String, usize)>,
    log: Vec<(String, String)>,
    last_tick: Instant,
    tick_count: u64,
    modal: Modal,
    input_buffer: String,
    scroll: usize,
    selected_file: usize,
    view_content: String,
    node_id: String,
}

impl Dashboard {
    fn new(node: Node, repo_path: PathBuf) -> io::Result<Self> {
        let peer_id = node.peer_id().unwrap_or_else(|_| "unknown".to_string());
        let addrs = node.listening_addrs().unwrap_or_default();
        let peers = node.swarm_peers().unwrap_or_default();
        let node_id = node.id().unwrap_or_else(|_| "{}".to_string());
        Ok(Self {
            node,
            peer_id,
            version: kubo_rs::version(),
            repo_path,
            tab: Tab::Status,
            addrs,
            peers,
            files: Vec::new(),
            blocks: Vec::new(),
            log: Vec::new(),
            last_tick: Instant::now(),
            tick_count: 0,
            modal: Modal::None,
            input_buffer: String::new(),
            scroll: 0,
            selected_file: 0,
            view_content: String::new(),
            node_id,
        })
    }

    fn tick(&mut self) {
        self.tick_count += 1;
        if self.tick_count % 4 == 0 {
            if let Ok(addrs) = self.node.listening_addrs() {
                self.addrs = addrs;
            }
            if let Ok(peers) = self.node.swarm_peers() {
                self.peers = peers;
            }
            if let Ok(id) = self.node.id() {
                self.node_id = id;
            }
        }
    }

    fn log(&mut self, action: &str, detail: &str) {
        let now = self.tick_count / 4;
        let time = format!(
            "{:02}:{:02}:{:02}",
            (now / 3600) % 24,
            (now / 60) % 60,
            now % 60
        );
        self.log.push((time, format!("{action}: {detail}")));
        if self.log.len() > 200 {
            self.log.remove(0);
        }
    }

    fn add_demo_content(&mut self) {
        let data = b"dashboard demo - hello ipfs";
        if let Ok(cid) = self.node.add_bytes(data) {
            self.files
                .push((cid.clone(), "hello.txt".to_string(), data.len()));
            self.log("add", &format!("{cid} ({len} bytes)", len = data.len()));
        }
        let block_data = b"raw block for dashboard";
        if let Ok(cid) = self.node.block_put(block_data) {
            if let Ok(size) = self.node.block_stat(&cid) {
                self.blocks.push((cid.clone(), size));
                self.log("block-put", &format!("{cid} ({size} bytes)"));
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo = env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| {
        let tmp = env::temp_dir();
        tmp.join("kubo-rs-dashboard")
    });

    if !repo.join("config").exists() {
        println!("initializing repo at {}...", repo.display());
        init_repo(&repo)?;
    }

    let node = Node::start(&repo, true)?;
    let mut dash = Dashboard::new(node, repo)?;
    dash.add_demo_content();

    let mut terminal = setup_terminal()?;
    let res = run_app(&mut terminal, &mut dash);
    restore_terminal(&mut terminal)?;
    dash.node.stop()?;
    res.map_err(|e| e.into())
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        event::EnableMouseCapture
    )?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    dash: &mut Dashboard,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(250);
    loop {
        terminal.draw(|f| ui(f, dash))?;
        let timeout = tick_rate.saturating_sub(dash.last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match dash.modal {
                        Modal::AddContent => match key.code {
                            KeyCode::Enter => {
                                if !dash.input_buffer.is_empty() {
                                    let data = dash.input_buffer.clone().into_bytes();
                                    if let Ok(cid) = dash.node.add_bytes(&data) {
                                        dash.files.push((
                                            cid.clone(),
                                            "typed".to_string(),
                                            data.len(),
                                        ));
                                        dash.log("add", &format!("{cid} ({} bytes)", data.len()));
                                    }
                                }
                                dash.modal = Modal::None;
                                dash.input_buffer.clear();
                            }
                            KeyCode::Char(c) => dash.input_buffer.push(c),
                            KeyCode::Backspace => {
                                dash.input_buffer.pop();
                            }
                            KeyCode::Esc => {
                                dash.modal = Modal::None;
                                dash.input_buffer.clear();
                            }
                            _ => {}
                        },
                        Modal::ConnectPeer => match key.code {
                            KeyCode::Enter => {
                                if !dash.input_buffer.is_empty() {
                                    let addr = dash.input_buffer.clone();
                                    if let Err(e) = dash.node.connect(&addr) {
                                        dash.log("connect", &format!("failed: {e}"));
                                    } else {
                                        dash.log("connect", &addr);
                                    }
                                }
                                dash.modal = Modal::None;
                                dash.input_buffer.clear();
                            }
                            KeyCode::Char(c) => dash.input_buffer.push(c),
                            KeyCode::Backspace => {
                                dash.input_buffer.pop();
                            }
                            KeyCode::Esc => {
                                dash.modal = Modal::None;
                                dash.input_buffer.clear();
                            }
                            _ => {}
                        },
                        Modal::ViewContent => {
                            dash.modal = Modal::None;
                        }
                        Modal::None => match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('1') => dash.tab = Tab::Status,
                            KeyCode::Char('2') => dash.tab = Tab::Files,
                            KeyCode::Char('3') => dash.tab = Tab::Peers,
                            KeyCode::Char('4') => dash.tab = Tab::Network,
                            KeyCode::Char('5') => dash.tab = Tab::Blocks,
                            KeyCode::Char('6') => dash.tab = Tab::Logs,
                            KeyCode::Right | KeyCode::Tab => {
                                let tabs = Tab::all();
                                let idx = tabs.iter().position(|&t| t == dash.tab).unwrap_or(0);
                                dash.tab = tabs[(idx + 1) % tabs.len()];
                            }
                            KeyCode::Left | KeyCode::BackTab => {
                                let tabs = Tab::all();
                                let idx = tabs.iter().position(|&t| t == dash.tab).unwrap_or(0);
                                dash.tab = tabs[(idx + tabs.len() - 1) % tabs.len()];
                            }
                            KeyCode::Char('a') if dash.tab == Tab::Files => {
                                dash.modal = Modal::AddContent;
                            }
                            KeyCode::Char('r') if dash.tab == Tab::Files => {
                                dash.files.clear();
                                dash.log("files", "cleared file list");
                            }
                            KeyCode::Char('v')
                                if dash.tab == Tab::Files && !dash.files.is_empty() =>
                            {
                                let idx = dash.selected_file.min(dash.files.len() - 1);
                                let cid = dash.files[idx].0.clone();
                                match dash.node.cat(&cid) {
                                    Ok(data) => {
                                        dash.view_content =
                                            String::from_utf8_lossy(&data).to_string();
                                        dash.modal = Modal::ViewContent;
                                    }
                                    Err(e) => dash.log("cat", &format!("{cid} failed: {e}")),
                                }
                            }
                            KeyCode::Up if dash.tab == Tab::Files && dash.selected_file > 0 => {
                                dash.selected_file -= 1;
                            }
                            KeyCode::Down
                                if dash.tab == Tab::Files
                                    && dash.selected_file + 1 < dash.files.len() =>
                            {
                                dash.selected_file += 1;
                            }
                            KeyCode::Char('c') if dash.tab == Tab::Peers => {
                                dash.modal = Modal::ConnectPeer;
                            }
                            KeyCode::Up if dash.tab == Tab::Logs => {
                                dash.scroll = dash.scroll.saturating_sub(1);
                            }
                            KeyCode::Down if dash.tab == Tab::Logs => {
                                dash.scroll = dash.scroll.saturating_add(1);
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
        if dash.last_tick.elapsed() >= tick_rate {
            dash.tick();
            dash.last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut Frame, dash: &Dashboard) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    render_header_bar(f, dash, chunks[0]);
    render_tabs(f, dash, chunks[1]);
    render_main(f, dash, chunks[2]);
    render_footer(f, dash, chunks[3]);

    match dash.modal {
        Modal::AddContent => {
            let area = centered_rect(60, 20, f.area());
            f.render_widget(Clear, area);
            let popup = Paragraph::new(format!("Add text to IPFS:\n> {}", dash.input_buffer))
                .block(
                    Block::default()
                        .title(" Add Content ")
                        .borders(Borders::ALL)
                        .border_style(Color::Cyan),
                )
                .style(Style::default().fg(Color::White));
            f.render_widget(popup, area);
        }
        Modal::ConnectPeer => {
            let area = centered_rect(70, 20, f.area());
            f.render_widget(Clear, area);
            let popup = Paragraph::new(format!(
                "Connect to peer multiaddr:\n> {}",
                dash.input_buffer
            ))
            .block(
                Block::default()
                    .title(" Connect Peer ")
                    .borders(Borders::ALL)
                    .border_style(Color::Green),
            )
            .style(Style::default().fg(Color::White));
            f.render_widget(popup, area);
        }
        Modal::ViewContent => {
            let area = centered_rect(80, 70, f.area());
            f.render_widget(Clear, area);
            let popup = Paragraph::new(dash.view_content.as_str())
                .block(
                    Block::default()
                        .title(" File Content ")
                        .borders(Borders::ALL)
                        .border_style(Color::Yellow),
                )
                .style(Style::default().fg(Color::White))
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(popup, area);
        }
        Modal::None => {}
    }
}

fn render_header_bar(f: &mut Frame, dash: &Dashboard, area: Rect) {
    let status_color = Color::Green;
    let status_text = "● ONLINE";
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            status_text,
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "  kubo-rs {}  |  Peer: {}  |  Repo: {}",
                dash.version,
                &dash.peer_id[..std::cmp::min(16, dash.peer_id.len())],
                dash.repo_path.display()
            ),
            Style::default().fg(Color::Gray),
        ),
    ]));
    f.render_widget(header, area);
}

fn render_tabs(f: &mut Frame, dash: &Dashboard, area: Rect) {
    let titles: Vec<Line> = Tab::all()
        .iter()
        .map(|&t| {
            let style = if t == dash.tab {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            Line::from(Span::styled(format!(" {} ", t.title()), style))
        })
        .collect();
    let tabs = ratatui::widgets::Tabs::new(titles)
        .select(Tab::all().iter().position(|&t| t == dash.tab).unwrap_or(0))
        .style(Style::default().fg(Color::Cyan))
        .divider(symbols::line::VERTICAL);
    f.render_widget(tabs, area);
}

fn render_main(f: &mut Frame, dash: &Dashboard, area: Rect) {
    match dash.tab {
        Tab::Status => render_status_tab(f, dash, area),
        Tab::Files => render_files_tab(f, dash, area),
        Tab::Peers => render_peers_tab(f, dash, area),
        Tab::Network => render_network_tab(f, dash, area),
        Tab::Blocks => render_blocks_tab(f, dash, area),
        Tab::Logs => render_logs_tab(f, dash, area),
    }
}

fn render_status_tab(f: &mut Frame, dash: &Dashboard, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(chunks[0]);

    let info = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Version:     ", Style::default().fg(Color::Yellow)),
            Span::raw(&dash.version),
        ]),
        Line::from(vec![
            Span::styled("Peer ID:     ", Style::default().fg(Color::Yellow)),
            Span::raw(&dash.peer_id),
        ]),
        Line::from(vec![
            Span::styled("Online:      ", Style::default().fg(Color::Yellow)),
            Span::styled("true", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("Addresses:   ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", dash.addrs.len())),
        ]),
        Line::from(vec![
            Span::styled("Peers:       ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", dash.peers.len())),
        ]),
    ])
    .block(
        Block::default()
            .title(" Node Info ")
            .borders(Borders::ALL)
            .border_style(Color::Blue),
    )
    .style(Style::default().fg(Color::White));
    f.render_widget(info, left[0]);

    let addrs_text = if dash.addrs.is_empty() {
        "No listening addresses".to_string()
    } else {
        dash.addrs.join("\n")
    };
    let addrs = Paragraph::new(addrs_text)
        .block(
            Block::default()
                .title(" Listening Addresses ")
                .borders(Borders::ALL)
                .border_style(Color::Magenta),
        )
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(addrs, left[1]);

    let health = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Daemon:   ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "RUNNING",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("API:      ", Style::default().fg(Color::Yellow)),
            Span::styled("available", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("Gateway:  ", Style::default().fg(Color::Yellow)),
            Span::styled("available", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("Swarm:    ", Style::default().fg(Color::Yellow)),
            Span::styled("listening", Style::default().fg(Color::Green)),
        ]),
    ])
    .block(
        Block::default()
            .title(" Health ")
            .borders(Borders::ALL)
            .border_style(Color::Green),
    )
    .style(Style::default().fg(Color::White))
    .alignment(Alignment::Center);
    f.render_widget(health, chunks[1]);
}

fn render_files_tab(f: &mut Frame, dash: &Dashboard, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let rows: Vec<Row> = dash
        .files
        .iter()
        .enumerate()
        .map(|(i, (cid, name, size))| {
            let style = if i == dash.selected_file {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(Span::styled(name.clone(), Style::default().fg(Color::Cyan))),
                Cell::from(Span::raw(cid.clone())),
                Cell::from(Span::raw(format!("{} B", size))),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        &[
            Constraint::Length(12),
            Constraint::Min(40),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["Name", "CID", "Size"])
            .style(Style::default().add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .title(" Files ")
            .borders(Borders::ALL)
            .border_style(Color::Cyan),
    );
    f.render_widget(table, chunks[0]);

    let help = Paragraph::new("a: add  v: view  r: clear  ↑↓: select")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(help, chunks[1]);
}

fn render_peers_tab(f: &mut Frame, dash: &Dashboard, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let header = Paragraph::new(format!("Connected peers: {}", dash.peers.len()))
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let items: Vec<ListItem> = dash
        .peers
        .iter()
        .map(|(id, addr)| {
            let text = if addr.is_empty() {
                id.clone()
            } else {
                format!("{}  →  {}", id, addr)
            };
            ListItem::new(Line::from(vec![
                Span::styled("● ", Style::default().fg(Color::Green)),
                Span::raw(text),
            ]))
            .style(Style::default().fg(Color::White))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Peers ")
                .borders(Borders::ALL)
                .border_style(Color::Green),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));
    f.render_widget(list, chunks[1]);
}

fn render_network_tab(f: &mut Frame, dash: &Dashboard, area: Rect) {
    let id_text = if dash.node_id.len() > 120 {
        format!("{}...", &dash.node_id[..120])
    } else {
        dash.node_id.clone()
    };
    let identity = Paragraph::new(id_text)
        .block(
            Block::default()
                .title(" Node Identity (JSON) ")
                .borders(Borders::ALL)
                .border_style(Color::Blue),
        )
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(identity, area);
}

fn render_blocks_tab(f: &mut Frame, dash: &Dashboard, area: Rect) {
    let rows: Vec<Row> = dash
        .blocks
        .iter()
        .map(|(cid, size)| {
            Row::new(vec![
                Cell::from(Span::raw(cid.clone())),
                Cell::from(Span::raw(format!("{} B", size))),
            ])
        })
        .collect();

    let table = Table::new(rows, &[Constraint::Min(50), Constraint::Length(10)])
        .header(
            Row::new(vec!["CID", "Size"])
                .style(Style::default().add_modifier(Modifier::BOLD))
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .title(" Blocks ")
                .borders(Borders::ALL)
                .border_style(Color::Yellow),
        );
    f.render_widget(table, area);
}

fn render_logs_tab(f: &mut Frame, dash: &Dashboard, area: Rect) {
    let visible: Vec<Row> = dash
        .log
        .iter()
        .rev()
        .skip(dash.scroll)
        .take(area.height as usize - 2)
        .map(|(time, msg)| {
            Row::new(vec![
                Cell::from(Span::styled(
                    time.clone(),
                    Style::default().fg(Color::Yellow),
                )),
                Cell::from(Span::raw(msg.clone())),
            ])
        })
        .collect();

    let table = Table::new(visible, &[Constraint::Length(10), Constraint::Min(20)])
        .header(
            Row::new(vec!["Time", "Event"])
                .style(Style::default().add_modifier(Modifier::BOLD))
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .title(" Activity Log ")
                .borders(Borders::ALL)
                .border_style(Color::Cyan),
        );
    f.render_widget(table, area);
}

fn render_footer(f: &mut Frame, _dash: &Dashboard, area: Rect) {
    let footer = Paragraph::new(
        "1-6: tabs  ←→: navigate  q: quit  |  Files: a=add v=view r=clear  |  Peers: c=connect  |  Logs: ↑↓=scroll",
    )
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::DarkGray).bg(Color::Black));
    f.render_widget(footer, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
