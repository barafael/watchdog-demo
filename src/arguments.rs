use clap::{Parser, Subcommand};

use crate::{client::Client, server::Server};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Arguments {
    #[clap(subcommand)]
    pub mode: Mode,
}

#[derive(Subcommand)]
pub enum Mode {
    Server(Server),
    Client(Client),
}
