use clap::Parser;
use watchdog_demo::{run, Arguments};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();
    run::<false>(&args).await
}
