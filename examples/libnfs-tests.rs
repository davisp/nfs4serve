use anyhow::{Context as _, Result};
use nfs41server::NFSv4Server;

#[tokio::main]
async fn main() -> Result<()> {
    stderrlog::new().verbosity(4).init().unwrap();

    let server = NFSv4Server::new("127.0.0.1:9342")
        .await
        .context("Error creating server.")?;
    server.serve().await.context("The main server loop died.")
}
