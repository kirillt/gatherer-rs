use super::metrics::Digest;
use super::storage::Storage;

use tungstenite::WebSocket;

use std::time::Instant;
use std::io::{Read, Write};
use std::fmt::Debug;

#[derive(Debug)]
struct RemoteSource<Stream> {
    websocket: WebSocket<Stream>,
    last_read: Instant,
    buffer: Vec<u8>,
}

pub fn listen<D, A, S>(db: D, accept: A, prune_period: u64, countdown: u64) -> std::io::Result<()>
    where A: Fn() -> std::io::Result<WebSocket<S>>,
          S: Read + Write + Debug,
          D: Storage {
    if countdown > 0 {
        println!("Exiting after {} seconds", countdown);
    }

    let mut remote_sources: Vec<RemoteSource<S>> = vec![];

    let start = Instant::now();
    while countdown == 0 || start.elapsed().as_secs() < countdown {
        if let Ok(websocket) = accept() {
            println!("Incoming WebSocket connection: {:?}", websocket);

            let last_read = Instant::now();
            remote_sources.push(RemoteSource {
                websocket,
                last_read,
                buffer: vec![]
            });
        }

        remote_sources.drain_filter(|remote_source| {
            if let Ok(message) = remote_source.websocket.read_message() {
                let message = message.into_text()
                    .unwrap();

                print!("[{} bytes] ", message.as_bytes().len());

                match serde_json::from_str::<Digest>(&message) {
                    Err(error) => println!("Something went wrong: {}", error),
                    Ok(metrics) => db.process(metrics)
                }

                false
            } else {
                if prune_period != 0
                    && remote_source.last_read.elapsed().as_secs() > prune_period {
                    println!("Closing WebSocket connection: {:?}", remote_source.websocket);
                    match remote_source.websocket.close(None) {
                        Err(e) => println!("{:?}", e),
                        Ok(()) => println!("Closed."),
                    }
                    true
                } else {
                    false
                }
            }
        });
    }

    Ok(())
}
