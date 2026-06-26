//! The run harness shared by every example binary.
//!
//! An example implements [`Example`] (advance the feed, handle a key, draw a
//! frame). [`run`] then does one of two things based on the command line:
//!
//! - **interactive** (default): a normal ratatui/crossterm loop on the real
//!   terminal — animates the feed, `q` quits.
//! - **headless** (`--headless`): advance the feed a fixed number of frames,
//!   render exactly one frame into an off-screen [`TestBackend`], print it to
//!   stdout as plain text, and exit. No TTY needed — this is the quick check a
//!   scripted agent or CI runs.

use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{CrosstermBackend, TestBackend},
    buffer::Buffer,
    Frame, Terminal,
};

/// One runnable widget demo. Keep implementations tiny: state in the struct,
/// behaviour in these three methods.
pub trait Example {
    /// Shown in the title bar / headless header.
    fn title(&self) -> &str;
    /// Advance the synthetic feed (and any per-frame animation) one step.
    fn on_tick(&mut self);
    /// Handle a key press. Interactive only; `q`/Esc/Ctrl-C are handled for you.
    fn on_key(&mut self, _key: KeyCode) {}
    /// Draw the current state into the frame. Takes `&mut self` because stateful
    /// widgets (e.g. a scrolling `DataTable`) update their view state on render.
    fn draw(&mut self, frame: &mut Frame);
    /// One-line hint shown under the headless snapshot (controls, etc.).
    fn hint(&self) -> &str {
        "press q to quit"
    }
}

/// Parsed from the command line by [`Config::from_args`].
struct Config {
    headless: bool,
    /// Headless: how many feed steps to advance before snapshotting.
    frames: usize,
    /// Headless snapshot size (columns × rows).
    width: u16,
    height: u16,
    /// Interactive: feed steps per second.
    fps: u64,
}

impl Config {
    fn from_args() -> Self {
        // Tiny hand-rolled parser — no clap, so the example has one less dep to
        // explain. Recognises: --headless, --frames N, --fps N, --size WxH.
        let mut cfg = Config {
            headless: false,
            frames: 120,
            width: 110,
            height: 32,
            fps: 12,
        };
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--headless" => cfg.headless = true,
                "--frames" => cfg.frames = next_num(&mut args, cfg.frames),
                "--fps" => cfg.fps = next_num(&mut args, cfg.fps).max(1),
                "--size" => {
                    if let Some((w, h)) = args.next().and_then(|s| parse_size(&s)) {
                        cfg.width = w;
                        cfg.height = h;
                    }
                }
                _ => {}
            }
        }
        cfg
    }
}

fn next_num<T: std::str::FromStr>(args: &mut impl Iterator<Item = String>, default: T) -> T {
    args.next().and_then(|s| s.parse().ok()).unwrap_or(default)
}

fn parse_size(s: &str) -> Option<(u16, u16)> {
    let (w, h) = s.split_once(['x', 'X'])?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

/// Entry point each example binary calls from `main`.
pub fn run(mut example: impl Example) -> io::Result<()> {
    let cfg = Config::from_args();
    if cfg.headless {
        run_headless(&mut example, &cfg)
    } else {
        run_interactive(&mut example, &cfg)
    }
}

/// Advance the feed, render one frame off-screen, print it as text. No terminal.
fn run_headless(example: &mut impl Example, cfg: &Config) -> io::Result<()> {
    for _ in 0..cfg.frames {
        example.on_tick();
    }

    let backend = TestBackend::new(cfg.width, cfg.height);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|frame| example.draw(frame))?;

    println!(
        "── {} · headless snapshot after {} frames ({}×{}) ──",
        example.title(),
        cfg.frames,
        cfg.width,
        cfg.height
    );
    print_buffer(terminal.backend().buffer());
    println!("── {} ──", example.hint());
    Ok(())
}

/// Dump a ratatui buffer as plain text (symbols only — deterministic & diffable).
fn print_buffer(buf: &Buffer) {
    let area = buf.area();
    let mut line = String::with_capacity(area.width as usize);
    for y in 0..area.height {
        line.clear();
        for x in 0..area.width {
            line.push_str(buf.get(x, y).symbol());
        }
        // Trim trailing blanks so the snapshot isn't a wall of spaces.
        println!("{}", line.trim_end());
    }
}

/// The live terminal loop: draw, then wait for the next key or tick.
fn run_interactive(example: &mut impl Example, cfg: &Config) -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let tick = Duration::from_millis(1000 / cfg.fps);
    let mut last_tick = Instant::now();

    let result = loop {
        if let Err(e) = terminal.draw(|frame| example.draw(frame)) {
            break Err(e);
        }

        // Wait for input, but never longer than the time left until the next tick.
        let timeout = tick.saturating_sub(last_tick.elapsed());
        match event::poll(timeout) {
            Ok(true) => match event::read() {
                Ok(Event::Key(key)) => {
                    let quit = matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                        || (key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL));
                    if quit {
                        break Ok(());
                    }
                    example.on_key(key.code);
                }
                Ok(_) => {}
                Err(e) => break Err(e),
            },
            Ok(false) => {}
            Err(e) => break Err(e),
        }

        if last_tick.elapsed() >= tick {
            example.on_tick();
            last_tick = Instant::now();
        }
    };

    restore_terminal(&mut terminal)?;
    result
}

type Tui = Terminal<CrosstermBackend<Stdout>>;

fn setup_terminal() -> io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal(terminal: &mut Tui) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}
