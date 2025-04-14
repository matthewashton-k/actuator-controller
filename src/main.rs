use std::{env::args, io, time::Duration};
use tokio::{sync::mpsc, time::sleep};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
    text::Text,
};

mod commands;
use commands::*;

struct App {
    speed: u32,
    direction: commands::Direction, // true = forward, false = backward
    max_speed: u32,
    status_message: String,
    actuator: commands::Actuator
}

impl App {
    fn new() -> App {
        App {
            speed: 0,
            direction: commands::Direction::Forward,
            max_speed: 65535, // Adjust based on the motor's capabilities
            status_message: String::from("Ready"),
            actuator: commands::Actuator::M1
        }
    }

    fn increase_speed(&mut self, amount: u32) {
        self.speed = (self.speed + amount).min(self.max_speed);
    }

    fn decrease_speed(&mut self, amount: u32) {
        self.speed = self.speed.saturating_sub(amount);
    }

    fn set_direction(&mut self, dir: commands::Direction) {
        self.direction = dir;
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

    let binding = args().collect::<Vec<String>>();
    let Some(port_path) = binding.get(1) else {
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        eprintln!("supply path argument. Example: /dev/ttyACM0");
        return Ok(());
    };

    
    let port = match tokio_serial::new(port_path, 9600).open_native_async() {
        Ok(p) => p,
        Err(e) => {
            // Restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
        terminal.show_cursor()?;
            eprintln!("Couldn't open {port_path}: {e}");
            return Ok(());
        }
    };
    
    let status_tx_clone = status_tx.clone();
    tokio::spawn(async move {
        let mut port = port;
        while let Some(cmd) = rx.recv().await {
            match cmd {
                ActuatorCommand::SetSpeed(speed, actuator) => {
                    let bytes = ActuatorCommand::SetSpeed(speed, actuator).serialize();
                    if let Err(e) = port.try_write(&bytes) {
                        let _ = status_tx_clone.send(format!("Serial error: {}", e)).await;
                    } else {
                        let _ = status_tx_clone.send(format!("Set speed to {}", speed)).await;
                    }
                    let mut buf = [234,4,04];
                }
                ActuatorCommand::SetDirection(dir, actuator) => {
                    let bytes = ActuatorCommand::SetDirection(dir, actuator).serialize();
                    if let Err(e) = port.try_write(&bytes) {
                        let _ = status_tx_clone.send(format!("Serial error: {}", e)).await;
                    } else {
                        let dir_str = if dir == commands::Direction::Forward { "forward" } else { "backward" };
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
                .direction(ratatui::layout::Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ].as_ref())
                .split(f.area());

            let dir_str = if app.direction == Direction::Forward {"Forward"} else {"Backward"};
            
            let speed_text = Text::from(format!("Speed: {} / {}", app.speed, app.max_speed));
            let speed_paragraph = Paragraph::new(speed_text)
                .block(Block::default().title("Motor Speed").borders(Borders::ALL));
            f.render_widget(speed_paragraph, chunks[0]);
            
            let dir_text = Text::from(format!("Direction: {}", dir_str));
            let dir_paragraph = Paragraph::new(dir_text)
                .block(Block::default().title("Motor Direction").borders(Borders::ALL));
            f.render_widget(dir_paragraph, chunks[1]);
            
            let status_text = Text::from(format!("Status: {} | {:?}", app.status_message, app.actuator));
            let status_paragraph = Paragraph::new(status_text)
                .block(Block::default().title("Status").borders(Borders::ALL));
            
            f.render_widget(status_paragraph, chunks[2]);
            
            let help_text = Text::from(
                "↑/↓: Change speed | ←/→: Switch Direction | q: Quit\n\
                 s: Stop motor | +/-: Increase/decrease speed by 5000 | a: Change actuator (bucket or lift)"
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
                        let _ = tx.send(ActuatorCommand::SetSpeed(0, app.actuator)).await;
                    },
                    KeyCode::Up => {
                        app.increase_speed(1000);
                        let _ = tx.send(ActuatorCommand::SetSpeed(app.speed as u16, app.actuator)).await;
                    },
                    KeyCode::Down => {
                        app.decrease_speed(1000);
                        let _ = tx.send(ActuatorCommand::SetSpeed(app.speed as u16, app.actuator)).await;
                    },
                    KeyCode::Left => {
                        app.set_direction(commands::Direction::Backward);
                        let _ = tx.send(ActuatorCommand::SetDirection(
                            commands::Direction::Backward,
                            app.actuator
                        )).await; 
                    }
                    
                    KeyCode::Right => {
                        app.set_direction(commands::Direction::Forward);
                        let _ = tx.send(ActuatorCommand::SetDirection(
                            commands::Direction::Forward,
                            app.actuator
                        )).await;
                    },
                    KeyCode::Char('+') => {
                        app.increase_speed(5000);
                        let _ = tx.send(ActuatorCommand::SetSpeed(app.speed as u16, app.actuator)).await;
                    },
                    KeyCode::Char('-') => {
                        app.decrease_speed(5000);
                        let _ = tx.send(ActuatorCommand::SetSpeed(app.speed as u16, app.actuator)).await;
                    },
                    KeyCode::Char('a') => {
                        app.speed = 0;
                        let _ = tx.send(ActuatorCommand::SetSpeed(
                            app.speed as u16,
                            app.actuator
                        )).await;
                        if app.actuator == Actuator::M1 {
                            app.actuator = Actuator::M2;
                        } else {
                            app.actuator = Actuator::M1;
                        }
                        app.status_message = format!("Switched to {:?}",app.actuator);
                    }
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
