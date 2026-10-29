use clap::Parser;
use std::{net::SocketAddr, path::PathBuf};
use wikit_server::{Catalog, router};

/// Serve local Wikit or MDX dictionaries over HTTP.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Dictionary path; repeat for multiple dictionaries.
    #[arg(short, long, required = true)]
    dictionary: Vec<PathBuf>,
    /// Listen address. Use 0.0.0.0 only when remote access is intended.
    #[arg(long, default_value = "127.0.0.1:8888")]
    listen: SocketAddr,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let catalog = Catalog::load(&args.dictionary)?;
    let listener = tokio::net::TcpListener::bind(args.listen).await?;
    eprintln!(
        "Wikit server listening on http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, router(catalog))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
