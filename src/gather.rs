#![feature(drain_filter)]

mod sfu;
mod metrics;
mod storage;
mod server;

use clap::Clap;
use native_tls::{Identity, TlsAcceptor, TlsStream};
use influx_db_client::UdpClient;

use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::fs::File;

#[derive(Clap)]
#[clap(version = "1.0")]
struct Options {
    #[clap(short, long)]
    port: u16,
    #[clap(long, short)]
    tls_keystore: Option<String>,
    #[clap(long)]
    tls_password: Option<String>,
    #[clap(short, long)]
    database_url: Option<String>,
    // #[clap(short, long)]
    // unix_path: String,
    #[clap(long, default_value = "60")]
    prune_period: u64,
    #[clap(short, long, default_value = "0")]
    countdown: u64,
}

fn main() -> std::io::Result<()> {
    let options: Options = Options::parse();

    let tls_acceptor = options.tls_keystore.as_ref()
        .map(|pkcs12_path| {
            let mut file = File::open(pkcs12_path)
                .unwrap();
            let password = options.tls_password
                .as_ref().map(|password| &password[..])
                .unwrap_or("");

            let mut identity = vec![];
            file.read_to_end(&mut identity)
                .unwrap();
            let identity = Identity::from_pkcs12(&identity, password)
                .unwrap();

            TlsAcceptor::new(identity)
                .unwrap()
        });

    let tcp_listener = TcpListener::bind(
        format!("0.0.0.0:{}", options.port))?;
    tcp_listener.set_nonblocking(true)?;

    let db: Option<UdpClient> = options.database_url
        .map(|url| UdpClient::new(url.parse().unwrap()));

    if let Some(tls_acceptor) = tls_acceptor {
        server::listen::<_, _, TlsStream<TcpStream>>(db, move || {
            let (tcp_stream, _) = tcp_listener.accept()?;
            let tcp_stream_hacked: &TcpStream = unsafe { std::mem::transmute(&tcp_stream) };
            tls_acceptor.accept(tcp_stream)
                .map_err(|_| std::io::Error::last_os_error())
                .and_then(move |tls_stream| {
                    let websocket = tungstenite::accept(tls_stream)
                        .map_err(|_| std::io::Error::last_os_error());

                    tcp_stream_hacked.set_nonblocking(true)?;
                    websocket
                })
        }, options.prune_period,
           options.countdown)
    } else {
        server::listen::<_, _, TcpStream>(db, move || {
            let (tcp_stream, _) = tcp_listener.accept()?;
            let tcp_stream_hacked: &TcpStream = unsafe { std::mem::transmute(&tcp_stream) };
            let websocket = tungstenite::accept(tcp_stream)
                .map_err(|_| std::io::Error::last_os_error());

            tcp_stream_hacked.set_nonblocking(true)?;
            websocket
        }, options.prune_period,
           options.countdown)
    }
}