use std::{net::SocketAddr, time::Duration};

use anyhow::Context;
use clap::Args;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader},
    net::{TcpListener, TcpStream},
    select,
    sync::mpsc,
};
use tracing::{info, warn};
use watchdog::Signal;

const DEFAULT_DURATION: Duration = Duration::from_secs(10);

#[derive(Args)]
pub struct Server {
    #[clap(short, long, default_value = "0.0.0.0:8000")]
    socket: SocketAddr,

    #[clap(short, long, default_value_t = DEFAULT_DURATION.into())]
    timeout: humantime::Duration,
}

impl Server {
    pub async fn run<const SHOW_PROGRESS_BAR: bool>(&self) -> anyhow::Result<()> {
        info!(
            "Starting server, please notify at a rate of more than {} to keep it alive.",
            self.timeout
        );
        let watchdog = watchdog::Watchdog::with_timeout(self.timeout.into());
        let (watchdog, mut expiration) = watchdog.run();

        let listener = TcpListener::bind(self.socket).await?;

        if let Err(e) = loop {
            select! {
                conn = listener.accept() => {
                    match conn {
                        Ok((conn, client)) => {
                            let watchdog = watchdog.clone();
                            tokio::spawn(handle_connection(conn, client, watchdog));
                        },
                        e => {
                            break e.context("Failed to accept new connection");
                        }
                    }
                }
                e = &mut expiration => {
                    match e {
                        Ok(watchdog::Expired) => break Err(anyhow::anyhow!("Watchdog expired")),
                        Err(e) => {
                            break Err(e).context("Watchdog failed");
                        }
                    }
                }
            }
        } {
            warn!("Shutting down server: {e:?}.");
        } else {
            warn!("Shutting down server.");
        }
        Ok(())
    }
}

async fn handle_connection(
    mut conn: TcpStream,
    client: SocketAddr,
    watchdog: mpsc::Sender<Signal>,
) -> anyhow::Result<()> {
    let (reader, writer) = conn.split();
    if let Err(e) = handle_client(reader, writer, watchdog).await {
        warn!("Dropping connection to {client}: {e:?}");
    }
    Ok(())
}

async fn handle_client<R, W>(
    reader: R,
    _writer: W, // TODO try half duplex later
    watchdog: mpsc::Sender<Signal>,
) -> anyhow::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    while let Ok(n) = reader.read_line(&mut line).await {
        if n == 0 {
            return Ok(());
        }
        let Ok(signal) = serde_json::from_str::<Signal>(&line[..n]) else {
            anyhow::bail!("Invalid message from client: {:?}", &line[..n]);
        };
        info!("Got signal \"{signal:?}\", forwarding to watchdog");
        let Ok(()) = watchdog.send(signal).await else {
            anyhow::bail!("Watchdog is down");
        };
    }
    anyhow::bail!("Connection closed");
}
