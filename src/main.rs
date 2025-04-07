use std::{io, time::Duration};
use tokio::{sync::mpsc, time::sleep};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
    text::Text,
};

mod commands;
use commands::*;

struct App {
    speed: u32,
    direction: bool, // true = forward, false = backward
    max_speed: u32,
    status_message: String,
}

impl App {
    fn new() -> App {
        App {
            speed: 0,
            direction: true,
            max_speed: 65535, // Adjust based on the motor's capabilities
            status_message: String::from("Ready"),
        }
    }

    fn increase_speed(&mut self, amount: u32) {
        self.speed = (self.speed + amount).min(self.max_speed);
    }

    fn decrease_speed(&mut self, amount: u32) {
        self.speed = self.speed.saturating_sub(amount);
    }

    fn toggle_direction(&mut self) {
        self.direction = !self.direction;
    }
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, mut rx) = mpsc::channel::<ActuatorCommand>(100);
    let (status_tx, mut status_rx) = mpsc::channel::<String>(100);
    
    // REPLACE WITH ACTUAL PATH
    let port_path = "/dev/tnt1";
    let port = tokio_serial::new(port_path, 9600).open_native_async()
        .expect("Failed to open serial port");
    
    let status_tx_clone = status_tx.clone();
    tokio::spawn(async move {
        let mut port = port;
        while let Some(cmd) = rx.recv().await {
            match cmd {
                ActuatorCommand::SetSpeed(speed) => {
                    let bytes = ActuatorCommand::SetSpeed(speed).serialize();
                    if let Err(e) = port.try_write(&bytes) {
                        let _ = status_tx_clone.send(format!("Serial error: {}", e)).await;
                    } else {
                        let _ = status_tx_clone.send(format!("Set speed to {}", speed)).await;
                    }
                }
                ActuatorCommand::SetDirection(forward) => {
                    let bytes = ActuatorCommand::SetDirection(forward).serialize();
                    if let Err(e) = port.try_write(&bytes) {
                        let _ = status_tx_clone.send(format!("Serial error: {}", e)).await;
                    } else {
                        let dir_str = if forward == commands::Direction::Forward { "forward" } else { "backward" };
                        let _ = status_tx_clone.send(format!("Set direction to {}", dir_str)).await;
                    }
                }
            }
            sleep(Duration::from_millis(50)).await;
        }
    });

    let mut app = App::new();

    loop {
        if let Ok(msg) = status_rx.try_recv() {
            app.status_message = msg;
        }
        
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ].as_ref())
                .split(f.size());

            let dir_str = if app.direction { "Forward" } else { "Backward" };
            
            let speed_text = Text::from(format!("Speed: {} / {}", app.speed, app.max_speed));
            let speed_paragraph = Paragraph::new(speed_text)
                .block(Block::default().title("Motor Speed").borders(Borders::ALL));
            f.render_widget(speed_paragraph, chunks[0]);
            
            let dir_text = Text::from(format!("Direction: {}", dir_str));
            let dir_paragraph = Paragraph::new(dir_text)
                .block(Block::default().title("Motor Direction").borders(Borders::ALL));
            f.render_widget(dir_paragraph, chunks[1]);
            
            let status_text = Text::from(format!("Status: {}", app.status_message));
            let status_paragraph = Paragraph::new(status_text)
                .block(Block::default().title("Status").borders(Borders::ALL));
            f.render_widget(status_paragraph, chunks[2]);
            
            let help_text = Text::from(
                "↑/↓: Change speed | ←/→: Change direction | q: Quit\n\
                 s: Stop motor | +/-: Increase/decrease speed by 5000"
            );
            let help_paragraph = Paragraph::new(help_text)
                .block(Block::default().title("Controls").borders(Borders::ALL));
            f.render_widget(help_paragraph, chunks[3]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('s') => {
                        app.speed = 0;
                        let _ = tx.send(ActuatorCommand::SetSpeed(0)).await;
                    },
                    KeyCode::Up => {
                        app.increase_speed(1000);
                        let _ = tx.send(ActuatorCommand::SetSpeed(app.speed as u16)).await;
                    },
                    KeyCode::Down => {
                        app.decrease_speed(1000);
                        let _ = tx.send(ActuatorCommand::SetSpeed(app.speed as u16)).await;
                    },
                    KeyCode::Left | KeyCode::Right => {
                        app.toggle_direction();
                        let _ = tx.send(ActuatorCommand::SetDirection(commands::Direction::Forward)).await;
                    },
                    KeyCode::Char('+') => {
                        app.increase_speed(5000);
                        let _ = tx.send(ActuatorCommand::SetSpeed(app.speed as u16)).await;
                    },
                    KeyCode::Char('-') => {
                        app.decrease_speed(5000);
                        let _ = tx.send(ActuatorCommand::SetSpeed(app.speed as u16)).await;
                    },
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}