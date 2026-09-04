use std::env;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Block, Borders, Cell, Paragraph, Row, Sparkline, Table, Tabs},
};

use kubo_rs::{Node, init_repo};

struct App {
    node: Node,
    peer_id: String,
    version: String,
    addrs: Vec<String>,
    log: Vec<(String, String)>,
    bandwidth_in: Vec<u64>,
    bandwidth_out: Vec<u64>,
    selected_tab: usize,
    last_update: Instant,
    repo_path: PathBuf,
}

impl App {
    fn new(node: Node, repo_path: PathBuf) -> io::Result<Self> {
        let peer_id = node.peer_id().unwrap_or_else(|_| "unknown".to_string());
        let addrs = node.listening_addrs().unwrap_or_default();
        Ok(Self {
            node,
            peer_id,
            version: kubo_rs::version(),
            addrs,
            log: Vec::new(),
            bandwidth_in: vec![0; 60],
            bandwidth_out: vec![0; 60],
            selected_tab: 0,
            last_update: Instant::now(),
            repo_path,
        })
    }

    fn tick(&mut self) {
        if self.last_update.elapsed() >= Duration::from_secs(1) {
            self.last_update = Instant::now();
            if let Ok(addrs) = self.node.listening_addrs() {
                self.addrs = addrs;
            }
            // Simulate bandwidth data for visualization
            self.bandwidth_in.rotate_left(1);
            self.bandwidth_out.rotate_left(1);
            self.bandwidth_in[59] = rand::random::<u64>() % 1000;
            self.bandwidth_out[59] = rand::random::<u64>() % 800;
        }
    }

    fn add_log(&mut self, action: &str, detail: &str) {
        let time = format!(
            "{:02}:{:02}:{:02}",
            (self.last_update.elapsed().as_secs() / 3600) % 24,
            (self.last_update.elapsed().as_secs() / 60) % 60,
            self.last_update.elapsed().as_secs() % 60,
        );
        self.log.push((time, format!("{action}: {detail}")));
        if self.log.len() > 100 {
            self.log.remove(0);
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
    let mut app = App::new(node, repo)?;

    // Demo operations
    let data = b"dashboard demo content";
    let cid = app.node.add_bytes(data)?;
    app.add_log("add", &format!("cid={cid}"));

    let block_cid = app.node.block_put(b"raw block")?;
    app.add_log("block-put", &format!("cid={block_cid}"));

    let block_size = app.node.block_stat(&block_cid)?;
    app.add_log("block-stat", &format!("size={block_size}"));

    let mut terminal = setup_terminal()?;
    let res = run_app(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;

    app.node.stop()?;

    if let Err(err) = res {
        eprintln!("{err:?}");
    }

    Ok(())
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

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| ui(f, app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('1') => app.selected_tab = 0,
                        KeyCode::Char('2') => app.selected_tab = 1,
                        KeyCode::Char('3') => app.selected_tab = 2,
                        KeyCode::Right => app.selected_tab = (app.selected_tab + 1) % 3,
                        KeyCode::Left => {
                            app.selected_tab = (app.selected_tab + 2) % 3;
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(f.area());

    let titles = vec!["Node Info", "Network", "Activity Log"];
    let tabs = Tabs::new(titles)
        .select(app.selected_tab)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        )
        .divider(symbols::line::VERTICAL);
    f.render_widget(tabs, chunks[0]);

    match app.selected_tab {
        0 => render_node_info(f, app, chunks[1]),
        1 => render_network(f, app, chunks[1]),
        2 => render_log(f, app, chunks[1]),
        _ => {}
    }
}

fn render_node_info(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Min(0),
        ])
        .split(area);

    let info_text = format!(
        "Version:    {}\nPeer ID:    {}\nRepo:       {}\nOnline:     true",
        app.version,
        app.peer_id,
        app.repo_path.display(),
    );
    let info = Paragraph::new(info_text)
        .block(
            Block::default()
                .title(" Node Information ")
                .borders(Borders::ALL)
                .border_style(Color::Green),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(info, chunks[0]);

    let addrs_text = if app.addrs.is_empty() {
        "No listening addresses".to_string()
    } else {
        app.addrs.join("\n")
    };
    let addrs = Paragraph::new(addrs_text)
        .block(
            Block::default()
                .title(" Listening Addresses ")
                .borders(Borders::ALL)
                .border_style(Color::Yellow),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(addrs, chunks[1]);

    let help = Paragraph::new("Press 1/2/3 or Left/Right to switch tabs | q to quit")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(help, chunks[2]);
}

fn render_network(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Min(0),
        ])
        .split(area);

    let in_spark = Sparkline::default()
        .block(
            Block::default()
                .title(" Bandwidth In (bytes/sec) ")
                .borders(Borders::ALL),
        )
        .data(&app.bandwidth_in)
        .style(Style::default().fg(Color::Green))
        .max(1000);
    f.render_widget(in_spark, chunks[0]);

    let out_spark = Sparkline::default()
        .block(
            Block::default()
                .title(" Bandwidth Out (bytes/sec) ")
                .borders(Borders::ALL),
        )
        .data(&app.bandwidth_out)
        .style(Style::default().fg(Color::Blue))
        .max(1000);
    f.render_widget(out_spark, chunks[1]);

    let addrs_count = app.addrs.len();
    let stats_text = format!(
        "Connected addresses: {}\n\nUse this tab to monitor swarm activity.\n\nIn a full implementation this would show:\n- Connected peers\n- Protocol handlers\n- DHT routing table size",
        addrs_count
    );
    let stats = Paragraph::new(stats_text)
        .block(
            Block::default()
                .title(" Swarm Stats ")
                .borders(Borders::ALL)
                .border_style(Color::Magenta),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(stats, chunks[2]);
}

fn render_log(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let rows: Vec<Row> = app
        .log
        .iter()
        .rev()
        .take(area.height as usize - 4)
        .map(|(time, msg)| {
            Row::new(vec![
                Cell::from(Span::styled(
                    time.clone(),
                    Style::default().fg(Color::Yellow),
                )),
                Cell::from(Span::styled(msg.clone(), Style::default().fg(Color::White))),
            ])
        })
        .collect();

    let table = Table::new(rows, &[Constraint::Length(10), Constraint::Min(20)])
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
