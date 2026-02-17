use clap::Parser;
use reqwest::Client;
use serde::Deserialize;
use std::fs::File;
use std::io::Write;

/// Simple program to update Nix flake sources from GitHub releases
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// GitHub owner of the repository
    #[arg(short, long)]
    owner: String,

    /// Name of the repository
    #[arg(short, long)]
    repo: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _did_change = 0;
    let args = Args::parse();

    // This creates a collapsible section in GitHub Actions logs
    println!("::group::Updating sources for {}/{}", args.owner, args.repo);

    // Now use args.owner and args.repo in your existing logic
    let release = fetch_latest_release(&args.owner, &args.repo).await?;
    generate_sources_nix(&release)?;

    println!("Successfully generated sources.nix");
    println!("::endgroup::");
    Ok(())
}

async fn fetch_latest_release(owner: &str, repo: &str) -> anyhow::Result<ReleaseResponse> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );
    let client = Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "rust-release-notifier")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;

    let release: ReleaseResponse = response.json().await?;
    Ok(release)
}

#[derive(Deserialize, Debug)]
pub struct ReleaseResponse {
    pub tag_name: String,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Deserialize, Debug)]
pub struct ReleaseAsset {
    pub name: String,
    pub digest: String,
    pub browser_download_url: String,
}

fn generate_sources_nix(release: &ReleaseResponse) -> anyhow::Result<()> {
    let mut file = File::create("sources.nix")?;
    writeln!(file, "# This file was automatically generated. Do not edit manually.")?;
    writeln!(file, "{{")?;
    writeln!(file, "  version = {:?};", release.tag_name)?;
    writeln!(file, "  assets = {{")?;

    for asset in &release.assets {
        let platform = match asset.name.as_str() {
            n if n.contains("x86_64-unknown-linux-musl") => Some("x86_64-linux"),
            n if n.contains("aarch64-unknown-linux-musl") => Some("aarch64-linux"),
            n if n.contains("x86_64-apple-darwin") => Some("x86_64-darwin"),
            n if n.contains("aarch64-apple-darwin") => Some("aarch64-darwin"),
            _ => None,
        };

        if let Some(p) = platform {
            writeln!(file, "    {:?} = {{", p)?;
            writeln!(file, "      url = {:?};", asset.browser_download_url)?;
            writeln!(file, "      hash = {:?};", asset.digest)?;
            writeln!(file, "    }};")?;
        }
    }

    writeln!(file, "  }};")?;
    writeln!(file, "}}")?;
    Ok(())
}
