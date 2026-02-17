use clap::Parser;
use reqwest::Client;
use serde::Deserialize;
use std::fs::File;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Repositories in "owner/repo" format
    #[arg(short, long, value_delimiter = '\n', num_args = 1..)]
    repositories: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let client = Client::new();

    // Opening the file once to write the start of the Nix set
    let mut file = File::create("sources.nix")?;
    writeln!(file, "# This file was automatically generated. Do not edit manually.")?;
    writeln!(file, "{{")?;

    for repo_str in args.repositories {
        let parts: Vec<&str> = repo_str.split('/').collect();
        if parts.len() != 2 {
            eprintln!("::error:: Invalid repo format: {}", repo_str);
            continue;
        }

        let (owner, repo) = (parts[0], parts[1]);
        println!("::group::Processing {}/{}", owner, repo);

        match fetch_latest_release(&client, owner, repo).await {
            Ok(release) => {
                if let Err(e) = append_repo_to_nix(&mut file, repo, &release) {
                    eprintln!("::error:: Failed to write {}: {}", repo, e);
                }
            }
            Err(e) => eprintln!("::error:: Failed to fetch {}: {}", repo, e),
        }
        println!("::endgroup::");
    }

    writeln!(file, "}}")?; // Close the main Nix set
    Ok(())
}

async fn fetch_latest_release(client: &Client, owner: &str, repo: &str) -> anyhow::Result<ReleaseResponse> {
    let url = format!("https://api.github.com/repos/{}/{}/releases/latest", owner, repo);
    let response = client
        .get(url)
        .header("User-Agent", "nix-update-action")
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
    #[serde(default)] // GitHub API doesn't provide this by default!
    pub digest: String, 
    pub browser_download_url: String,
}

fn append_repo_to_nix(file: &mut File, repo_name: &str, release: &ReleaseResponse) -> anyhow::Result<()> {
    writeln!(file, "  {:?} = {{", repo_name)?;
    writeln!(file, "    version = {:?};", release.tag_name)?;
    writeln!(file, "    assets = {{")?;

    for asset in &release.assets {
        let platform = match asset.name.as_str() {
            n if n.contains("x86_64-unknown-linux-musl") => Some("x86_64-linux"),
            n if n.contains("aarch64-unknown-linux-musl") => Some("aarch64-linux"),
            n if n.contains("x86_64-apple-darwin") => Some("x86_64-darwin"),
            n if n.contains("aarch64-apple-darwin") => Some("aarch64-darwin"),
            _ => None,
        };

        if let Some(p) = platform {
            writeln!(file, "      {:?} = {{", p)?;
            writeln!(file, "        url = {:?};", asset.browser_download_url)?;
            writeln!(file, "        hash = {:?};", asset.digest)?;
            writeln!(file, "      }};")?;
        }
    }
    writeln!(file, "    }};")?;
    writeln!(file, "  }};")?;
    Ok(())
}
