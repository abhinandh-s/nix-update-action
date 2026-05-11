use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

mod macros;

static HEADER_WARNING: &str = "# This file was automatically generated. Do not edit manually.";

// For extracting `system` from url.
//
// (releases/download/v0.14.0/wasm-pack-v0.14.0-aarch64-apple-darwin.tar.gz)
//                                              ^^^^^^^^^^^^^^^^^^^^
//                                              -> "aarch64-darwin"
define_system![
    "x86_64-unknown-linux-musl" => "x86_64-linux",
    "aarch64-unknown-linux-musl" => "aarch64-linux",
    "x86_64-apple-darwin" => "x86_64-darwin",
    "aarch64-apple-darwin" => "aarch64-darwin",
];

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    owner: String,
    #[arg(short, long)]
    repo: String,

    #[arg(short, long)]
    file: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let client = Client::new();

    let mut json_file = File::create(args.file)?;
    writeln!(json_file, "{HEADER_WARNING}")?;

    let (owner, repo) = (args.owner, args.repo);

    println!("::group::Processing {}/{}", owner, repo);

    match fetch_latest_release(&client, &owner, &repo).await {
        Ok(release) => {
            let asset_map: HashMap<String, ReleaseAsset> = release
                .assets
                .clone()
                .into_iter()
                .filter(|a| !a.name.contains(".exe") && !a.name.contains("windows"))
                .map(|a| {
                    let platform = system_matcher(a.name.as_str());
                    (platform.to_string(), a)
                })
                .collect();

            let output = NixJsonOutput {
                version: release.tag_name.clone(),
                assets: asset_map,
            };

            let json = serde_json::to_string_pretty(&output)?;

            //  println!("{}", json);
            write!(json_file, "{}", json)?;
            //     if let Err(e) = append_repo_to_nix(&mut file, &repo, &release) {
            //         eprintln!("::error:: Failed to write {}: {}", repo, e);
            //     }
        }
        Err(e) => eprintln!("::error:: Failed to fetch {}: {}", repo, e),
    }
    println!("::endgroup::");

    // writeln!(file, "}}")?; // Close the main Nix set
    Ok(())
}

async fn fetch_latest_release(
    client: &Client,
    owner: &str,
    repo: &str,
) -> anyhow::Result<ReleaseResponse> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );
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

// ```nix
// {
//   version = "{TAG}";
//   assets = {
//     "{SYSTEM}" = {
//       url = "https://github.com/{OWNER}/{REPO}/releases/download/{TAG}/{ASSET}";
//       hash = "sha256:{HASH}";
//     };
//     "{SYSTEM}" = {
//       url = "https://github.com/{OWNER}/{REPO}/releases/download/{TAG}/{ASSET}";
//       hash = "sha256:{HASH}";
//     };
//     "{SYSTEM}" = {
//       url = "https://github.com/{OWNER}/{REPO}/releases/download/{TAG}/{ASSET}";
//       hash = "sha256:{HASH}";
//     };
//   };
// }
// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct ReleaseResponse {
    #[serde(rename(serialize = "version", deserialize = "tag_name"))]
    pub tag_name: String,
    pub assets: Vec<ReleaseAsset>,
}

//```nix
// "{SYSTEM}" = {
//   url = "https://github.com/{OWNER}/{REPO}/releases/download/{TAG}/{ASSET}";
//   hash = "sha256:{HASH}";
// };
//```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReleaseAsset {
    #[serde(skip_serializing)] // We don't want "name" inside the object
    pub name: String, // system info
    #[serde(rename(serialize = "hash", deserialize = "digest"))]
    pub digest: String,
    #[serde(rename(serialize = "url", deserialize = "browser_download_url"))]
    pub browser_download_url: String, // url
}

#[derive(Serialize)]
pub struct NixJsonOutput {
    pub version: String,
    pub assets: HashMap<String, ReleaseAsset>,
}

fn append_repo_to_nix(
    file: &mut File,
    repo_name: &str,
    release: &ReleaseResponse,
) -> anyhow::Result<()> {
    writeln!(file, "  {:?} = {{", repo_name)?;
    writeln!(file, "    version = {:?};", release.tag_name)?;
    writeln!(file, "    assets = {{")?;

    for asset in &release.assets {
        if asset.name.contains(".exe") || asset.name.contains("windows") {
            continue;
        }
        let platform = system_matcher(asset.name.as_str());
        writeln!(file, "      {:?} = {{", platform)?;
        writeln!(file, "        url = {:?};", asset.browser_download_url)?;
        writeln!(file, "        hash = {:?};", asset.digest)?;
        writeln!(file, "      }};")?;
    }
    writeln!(file, "    }};")?;
    writeln!(file, "  }};")?;
    Ok(())
}
