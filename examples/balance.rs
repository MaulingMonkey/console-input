use maulingmonkey_console_art::*;
use maulingmonkey_console_escape_codes::*;
use maulingmonkey_console_input::*;

use std::sync::mpsc;
use std::io::Write;



struct State {
    increasing: bool,
    value:      u8,
    max_value:  u8,
}

impl Default for State {
    fn default() -> Self {
        Self {
            increasing: true,
            value:      39,
            max_value:  78,
        }
    }
}



enum Event {
    Stdin(StdinEvent),
    Heartbeat,
}



fn main() {
    let (send_events, recv_events) = mpsc::channel();

    let events = send_events.clone();
    spawn_stdin_thread(move |ev| drop(events.send(Event::Stdin(ev))));

    let events = send_events.clone();
    std::thread::spawn(move || {
        while let Ok(_) = events.send(Event::Heartbeat) {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    let art_mode = ArtMode::enable().unwrap();
    print!("{}", vt100::alternate_screen_buffer());
    print!("{}", vt100::erase_in_display(..));
    print!("{}", vt100::cursor_show(false));
    let _ = std::io::stdout().flush();

    let thread = std::thread::spawn(move ||{
        let mut state = State::default();
        'gameloop: loop {
            'events: loop {
                match recv_events.try_recv() {
                    Ok(Event::Stdin(input)) => {
                        match input.data() {
                            b"\x1B[A" | b"\x1BOA" => {}, // Up
                            b"\x1B[B" | b"\x1BOB" => {}, // Down
                            b"\x1B[C" | b"\x1BOC" => state.increasing = true,  // Right
                            b"\x1B[D" | b"\x1BOD" => state.increasing = false, // Left
                            b"\x1B" | b" " | b"Q" | b"q" => { // Escape, Space, Q/q
                                input.terminate_stdin_thread();
                                break 'gameloop;
                            },
                            _other => {},
                        }
                    },
                    Ok(Event::Heartbeat) => {
                        if state.increasing {
                            if state.value < state.max_value { state.value += 1; }
                        } else {
                            if state.value > 0 { state.value -= 1; }
                        }
                    },
                    Err(mpsc::TryRecvError::Empty) => break 'events,
                    Err(mpsc::TryRecvError::Disconnected) => break 'gameloop,
                }
            }

            // redraw
            print!("\r{empty: >value$}@{erase}", empty="", value=state.value.into(), erase=vt100::erase_in_line(()..));
            let _ = std::io::stdout().flush();
        }
    });

    let thread = thread.join();
    print!("{}", vt100::cursor_show(true));
    print!("{}", vt100::main_screen_buffer());
    let _ = std::io::stdout().flush();
    art_mode.disable();
    thread.unwrap();
}
