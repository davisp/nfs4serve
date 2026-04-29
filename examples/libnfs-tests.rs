use anyhow::{Context as _, Result};
use nfs41server::NFSv41Server;

#[tokio::main]
async fn main() -> Result<()> {
    stderrlog::new().verbosity(4).init().unwrap();

    let server = NFSv41Server::new("127.0.0.1:9342")
        .context("Error creating server.")?;
    server.serve().await.context("The main server loop died.")
}
