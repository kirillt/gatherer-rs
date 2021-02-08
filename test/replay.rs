use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use serde_json::{self, Result, Value};
use tungstenite::{Message, WebSocket};
use clap::Clap;
use url::Url;

#[derive(Clap, Debug)]
#[clap(version = "1.0")]
struct Options {
    #[clap(short, long)]
    url: String,
    #[clap(short, long)]
    file: String
}

fn process_json<S: Read + Write>(lines: Vec<String>, websocket: &mut WebSocket<S>) {
    let text = lines.concat();
    let json: Result<Value> = serde_json::from_str(&text);
    if json.is_err() {
        println!("Something went wrong: {:?}", json);
    } else {
        websocket.write_message(Message::Text(text))
            .unwrap();
    }
}

fn main() -> std::io::Result<()> {
    let options: Options = Options::parse();
    println!("Options: {:?}", options);

    let url = Url::parse(&options.url)
        .unwrap();
    let (mut target, _) = tungstenite::connect(url)
        .unwrap();

    let source = File::open(options.file)?;
    let source = BufReader::new(source);

    let mut buffer: Vec<String> = vec![];
    for line in source.lines() {
        if let Ok(line) = line {
            if !buffer.is_empty() && line.chars().next() == Some('{') {
                process_json(buffer, &mut target);
                buffer = vec![];
            }
            buffer.push(line);
        } else {
            println!("Something went wrong: {:?}", line);
        }
    }
    process_json(buffer, &mut target);
    Ok(())
}