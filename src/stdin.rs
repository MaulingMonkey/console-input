use std::convert::*;
use std::io::{self, Read};
use std::sync::mpsc::*;
use std::thread::JoinHandle;



pub fn spawn_stdin_thread(mut f: impl Send + FnMut(StdinEvent) + 'static) -> JoinHandle<io::Result<()>> {
    std::thread::spawn(move || -> io::Result<()> {
        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();

        let (reset, check_reset) = sync_channel(1);
        loop {
            let mut buffer = [0u8; 15];
            let read = stdin.read(&mut buffer[..])?;
            f(StdinEvent {
                data:       buffer,
                data_len:   read.try_into().expect("bug: `read` shouldn't be able to overflow `data_len`"),
                reset:      reset.clone(),
            });
            match check_reset.recv().expect("bug: `reset` should still be in scope") {
                Reset::Continue     => {},
                Reset::Terminate    => break Ok(()),
            }
        }
    })
}

pub struct StdinEvent {
    data:       [u8; 15],
    data_len:   u8,
    reset:      SyncSender<Reset>,
}

impl StdinEvent {
    pub fn data(&self) -> &[u8] {
        &self.data[..self.data_len.into()]
    }

    pub fn terminate_stdin_thread(&self) {
        let _ = self.reset.send(Reset::Terminate);
    }
}

impl Drop for StdinEvent {
    fn drop(&mut self) {
        let _ = self.reset.send(Reset::Continue);
    }
}

enum Reset {
    Continue,
    Terminate,
}
