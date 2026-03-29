use crate::events::{Event, Events, PlayNoteEvent, WeakSender};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::thread;
use syntoniq_common::pitch::Pitch;
use tokio::task::JoinHandle;

pub fn run(events: Events) -> JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        let (line_tx, line_rx) = flume::unbounded();
        let h = thread::spawn(move || repl(line_tx));
        let weak_tx = events.sender().await;
        let r = handle_lines(weak_tx, line_rx).await;
        if let Err(e) = &r {
            eprintln!("error from repl: {e:?}");
        }
        events.shutdown().await;
        h.join().unwrap()?;
        r
    })
}

fn repl(line_ch: flume::Sender<String>) -> anyhow::Result<()> {
    const HISTORY_FILE: &str = "syntoniq-repl.txt";
    let mut rl = DefaultEditor::new()?;
    _ = rl.load_history(HISTORY_FILE);
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                println!("Line: {}", line);
                line_ch.send(line).unwrap();
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

async fn handle_lines(weak_tx: WeakSender, line_rx: flume::Receiver<String>) -> anyhow::Result<()> {
    // TODO: real implementation, including clearing MIDI notes on shutdown
    while let Ok(line) = line_rx.recv_async().await {
        let Some(tx) = weak_tx.upgrade() else {
            break;
        };
        let Ok(pitch) = Pitch::parse(&line) else {
            continue;
        };
        tx.send(Event::PlayNote(PlayNoteEvent {
            pitch,
            velocity: 127,
            note: None,
        }))?;
    }
    Ok(())
}
