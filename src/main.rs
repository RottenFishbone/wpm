#![allow(dead_code)]
#![allow(unused_imports)]
mod app;

use app::{UIEvent, Controller};

use std::{
    error::Error,
    io::{ Write, stdout },
    panic::{self, PanicInfo},
    sync::mpsc::Sender,
    thread,
    time:: { Duration, SystemTime },
};

use crossterm::{
    event::{ self, DisableMouseCapture, KeyModifiers,
        EnableMouseCapture, Event, KeyCode },
    terminal::{ EnterAlternateScreen, LeaveAlternateScreen,
        enable_raw_mode, disable_raw_mode},
    execute,
};
use tui::{Terminal, backend::CrosstermBackend};

type Result<T> = std::result::Result<T, UIError>;
type CrossTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

#[derive(Debug)]
enum UIError {}

fn main() {
    panic::set_hook(Box::new(|info|{
        panic_hook(info);
    }));
    
    enable_raw_mode().unwrap();
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // Spawn a thread to handle UI events
    let (event_tx, event_rx) = std::sync::mpsc::channel::<UIEvent>();
    spawn_event_loop(event_tx, 250).unwrap();
   
    // Main loop
    let (mut controller, exit_rx) = Controller::new();
    loop {
        terminal.draw(|f| app::view::render(f, &controller.model)).unwrap();
       
        // Blocking read on events, this causes a redraw on new UIEvents ONLY
        if let UIEvent::Input(key_ev) = event_rx.recv().unwrap() {
            // Handle <Ctrl+C>
            if let KeyModifiers::CONTROL = key_ev.modifiers {
                if key_ev.code == KeyCode::Char('c') { break; }
            }
            controller.handle_key_event(key_ev);
        } else {
            controller.update();
        }
       
        // Non-blocking read on exit signals
        match exit_rx.try_recv() {
            Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => { 
                break; 
            },
            _=> {}
        }


    }

    kill_terminal();

}

/// Spawn a thread that hooks into user events as well as emits a tick event
/// at a given interval. In the event that the Reciever is dropped, the thread
/// will close itself, effectively acting as a kill command.
fn spawn_event_loop(event_tx: Sender<UIEvent>, tick_rate: u64) -> Result<()> {
    thread::spawn(move || {
        // Declare tick_rate as a Duration
        let mut last_tick = SystemTime::now();
        let tick_rate = Duration::from_millis(tick_rate);
        loop {
            let elapsed = last_tick.elapsed().unwrap();
            // Poll for new events
            if event::poll(tick_rate).unwrap() {
                // Check for key events
                if let Event::Key(key) = event::read().unwrap() {
                    // Send the key event through the channel, closing
                    // the thread on error
                    if let Err(_) = event_tx.send(UIEvent::Input(key)) {
                        break;
                    }
                }
            }

            if elapsed >= tick_rate {
                // Send the tick event, closing thread on error
                if let Err(_) = event_tx.send(UIEvent::Tick) { break; }
                last_tick = SystemTime::now();
            }
        }
    });

    Ok(())
}

/// Revert the terminal session to a normal, usable, state.
fn kill_terminal(){
    execute!(stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture).unwrap();
    disable_raw_mode().unwrap();
}

/// Provides a hook that allows the program to return the terminal
/// to a usable state before exiting.
fn panic_hook(info: &PanicInfo<'_>) { 
    kill_terminal();
    eprintln!("Caught panic hook: {:?}", info);
}
