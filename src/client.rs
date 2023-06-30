use std::net::SocketAddr;

use anyhow::Context;
use clap::Args;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use watchdog::Signal;

#[derive(Debug, Args)]
pub struct Client {
    #[clap(short, long, default_value = "0.0.0.0:8000")]
    socket: SocketAddr,

    #[clap(short, long, arg_enum, default_value_t)]
    command: Signal,
}

impl Client {
    pub async fn run(&self) -> anyhow::Result<()> {
        let message =
            serde_json::to_string(&self.command).context("Failed to serialize command")?;
        let mut stream = TcpStream::connect(self.socket)
            .await
            .with_context(|| format!("Watchdog connection on {} failed", self.socket))?;
        stream
            .write_all(message.as_bytes())
            .await
            .context("Failed to send signal")?;
        Ok(())
    }
}
