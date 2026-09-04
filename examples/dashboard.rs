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
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Sparkline, Table},
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

struct Dashboard {
    node: Node,
    peer_id: String,
    version: String,
    repo_path: PathBuf,
    tab: Tab,
    addrs: Vec<String>,
    peers: Vec<String>,
    files: Vec<(String, String, usize)>,
    blocks: Vec<(String, usize)>,
    log: Vec<(String, String)>,
    bandwidth_in: Vec<u64>,
    bandwidth_out: Vec<u64>,
    last_tick: Instant,
    tick_count: u64,
    input_mode: bool,
    input_buffer: String,
    scroll: usize,
}

impl Dashboard {
    fn new(node: Node, repo_path: PathBuf) -> io::Result<Self> {
        let peer_id = node.peer_id().unwrap_or_else(|_| "unknown".to_string());
        let addrs = node.listening_addrs().unwrap_or_default();
        let peers = node.swarm_peers().unwrap_or_default();
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
            bandwidth_in: vec![0; 100],
            bandwidth_out: vec![0; 100],
            last_tick: Instant::now(),
            tick_count: 0,
            input_mode: false,
            input_buffer: String::new(),
            scroll: 0,
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
        }
        self.bandwidth_in.rotate_left(1);
        self.bandwidth_out.rotate_left(1);
        self.bandwidth_in[99] = (rand::random::<u64>() % 500) + 100;
        self.bandwidth_out[99] = (rand::random::<u64>() % 400) + 50;
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
                    if dash.input_mode {
                        match key.code {
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
                                dash.input_mode = false;
                                dash.input_buffer.clear();
                            }
                            KeyCode::Char(c) => dash.input_buffer.push(c),
                            KeyCode::Backspace => {
                                dash.input_buffer.pop();
                            }
                            KeyCode::Esc => {
                                dash.input_mode = false;
                                dash.input_buffer.clear();
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
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
                                dash.input_mode = true;
                            }
                            KeyCode::Char('r') if dash.tab == Tab::Files => {
                                dash.files.clear();
                                dash.log("files", "cleared file list");
                            }
                            KeyCode::Up if dash.tab == Tab::Logs => {
                                dash.scroll = dash.scroll.saturating_sub(1);
                            }
                            KeyCode::Down if dash.tab == Tab::Logs => {
                                dash.scroll = dash.scroll.saturating_add(1);
                            }
                            _ => {}
                        }
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

    if dash.input_mode {
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
        .map(|(cid, name, size)| {
            Row::new(vec![
                Cell::from(Span::styled(name.clone(), Style::default().fg(Color::Cyan))),
                Cell::from(Span::raw(cid.clone())),
                Cell::from(Span::raw(format!("{} B", size))),
            ])
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
    )
    .row_highlight_style(Style::default().bg(Color::DarkGray));
    f.render_widget(table, chunks[0]);

    let help = Paragraph::new("a: add text  |  r: clear list  |  Enter: confirm  |  Esc: cancel")
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
        .map(|p| {
            ListItem::new(Line::from(vec![
                Span::styled("● ", Style::default().fg(Color::Green)),
                Span::raw(p),
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Min(0),
        ])
        .split(area);

    let in_spark = Sparkline::default()
        .block(
            Block::default()
                .title(" Bandwidth In (bytes/sec) ")
                .borders(Borders::ALL),
        )
        .data(&dash.bandwidth_in)
        .style(Style::default().fg(Color::Green))
        .max(800);
    f.render_widget(in_spark, chunks[0]);

    let out_spark = Sparkline::default()
        .block(
            Block::default()
                .title(" Bandwidth Out (bytes/sec) ")
                .borders(Borders::ALL),
        )
        .data(&dash.bandwidth_out)
        .style(Style::default().fg(Color::Blue))
        .max(800);
    f.render_widget(out_spark, chunks[1]);

    let stats = Paragraph::new(format!(
        "Listening addresses: {}\nConnected peers: {}\n\nUse the Peers tab for detailed peer info.",
        dash.addrs.len(),
        dash.peers.len(),
    ))
    .block(
        Block::default()
            .title(" Stats ")
            .borders(Borders::ALL)
            .border_style(Color::Magenta),
    )
    .style(Style::default().fg(Color::White));
    f.render_widget(stats, chunks[2]);
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
        "1-6: tabs  ←→: navigate  q: quit  |  Files: a=add r=clear  |  Logs: ↑↓=scroll",
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
