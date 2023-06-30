use anyhow::Context;
use arguments::Mode;

mod arguments;
mod client;
mod server;

pub use arguments::Arguments;

pub async fn run<const SHOW_PROGRESS_BAR: bool>(cli: &Arguments) -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    match &cli.mode {
        Mode::Server(s) => {
            s.run::<SHOW_PROGRESS_BAR>().await?;
        }
        Mode::Client(c) => {
            c.run().await.context("Failed to run client")?;
        }
    }
    Ok(())
}
