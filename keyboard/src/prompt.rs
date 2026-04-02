use crate::events::Events;
use crate::prompt::player::Player;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::thread;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

mod player;

struct LineData {
    line: String,
    _done: oneshot::Sender<()>,
}
impl LineData {
    fn make(s: String) -> (Self, oneshot::Receiver<()>) {
        let (tx, rx) = oneshot::channel();
        (Self { line: s, _done: tx }, rx)
    }
}

pub fn run(events: Events) -> JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        let (line_tx, line_rx) = mpsc::channel(100);
        let h = thread::spawn(move || repl(line_tx));
        let weak_tx = events.sender().await;
        let mut p = Player::new(weak_tx);
        let r = p.handle_lines(line_rx).await;
        if let Err(e) = &r {
            eprintln!("error from prompt line reading interface: {e:?}");
        }
        p.clear().await;
        events.shutdown().await;
        h.join().unwrap()?;
        r
    })
}

fn repl(line_ch: mpsc::Sender<LineData>) -> anyhow::Result<()> {
    const HISTORY_FILE: &str = "syntoniq-prompt-history.txt";
    let mut rl = DefaultEditor::new()?;
    _ = rl.load_history(HISTORY_FILE);
    print!("{}", player::HELP);
    loop {
        // Also considered as prompt: 𝄆, but it's harder to see and recognize, and fermata seems
        // fitting since we hold notes indefinitely.
        let readline = rl.readline("𝄐 ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let (line_data, rx) = LineData::make(line);
                line_ch.blocking_send(line_data).unwrap();
                _ = rx.blocking_recv();
            }
            Err(e) => {
                match e {
                    ReadlineError::Interrupted | ReadlineError::Eof => {}
                    _ => return Err(e.into()),
                }
                break;
            }
        }
    }
    rl.save_history(HISTORY_FILE)?;
    Ok(())
}
