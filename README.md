## Workflow Usage

```yml
name: Update Nix Sources

on:
  push:
  schedule:
    - cron: '0 0 * * *' # Runs every day at 00:00 UTC
  workflow_dispatch:    # Allows you to run it manually from the "Actions" tab

jobs:
  update:
    runs-on: ubuntu-latest
    permissions:
      contents: write # Needed to push the changes back to the repo
    
    steps:
      - name: Checkout Repository
        uses: actions/checkout@v4

      - name: Run Nix Source Updater
        # Points to your Action repo and the version tag
        uses: abhinandh-s/nix-update-action@master #v1.0.0 
        with:
          owner: 'drager'
          repo: 'wasm-pack'

      - name: Commit and Push Changes
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          
          # Only commit if sources.nix actually changed
          if git add sources.nix && git commit -m "chore: update sources.nix to latest release"; then
            git push
          else
            echo "No changes detected in sources.nix"
          fi
```

### Generated sources.nix

```nix
{
  version = "v0.14.0";
  assets = {
    "aarch64-darwin" = {
      url = "https://github.com/drager/wasm-pack/releases/download/v0.14.0/wasm-pack-v0.14.0-aarch64-apple-darwin.tar.gz";
      hash = "sha256:9d0e70c6b229de18f0abfe910f2963e8f09ebae218250e9b09a1c3fdd955bef9";
    };
    "aarch64-linux" = {
      url = "https://github.com/drager/wasm-pack/releases/download/v0.14.0/wasm-pack-v0.14.0-aarch64-unknown-linux-musl.tar.gz";
      hash = "sha256:5941c7b05060440ff37ee50fe9009a408e63fa5ba607a3b0736f5a887ec5f2ca";
    };
    "x86_64-darwin" = {
      url = "https://github.com/drager/wasm-pack/releases/download/v0.14.0/wasm-pack-v0.14.0-x86_64-apple-darwin.tar.gz";
      hash = "sha256:46b66072ee9912b53f83841aecb04479a60e0705f7bb8b6681b377a07a512a23";
    };
    "x86_64-linux" = {
      url = "https://github.com/drager/wasm-pack/releases/download/v0.14.0/wasm-pack-v0.14.0-x86_64-unknown-linux-musl.tar.gz";
      hash = "sha256:278a8d668085821f4d1a637bd864f1713f872b0ae3a118c77562a308c0abfe8d";
    };
  };
  licenseKeys = [ "asl20" "mit" ];
}
```

## Nix Usage

```nix
{
  lib,
  stdenv,
  fetchurl,
}:
let
  sources = import ./sources.nix;
  system = stdenv.hostPlatform.system;

  # Access 'assets' attribute created by the Rust script
  asset = sources.assets.${system} or (throw "Unsupported system: ${system}");
in
stdenv.mkDerivation {
  pname = "wasm-pack";
  version = sources.version; # Uses the version from generated `sources.nix`

  src = fetchurl {
    url = asset.url;   # Use the full URL from sources.nix
    hash = asset.hash; # Uses the "sha256:..." hex format
  };

  sourceRoot = ".";

  installPhase = ''
    runHook preInstall

    mkdir -p $out/bin
    # Search for the binary in the unpacked source
    find . -maxdepth 2 -name "wasm-pack" -type f -exec cp {} $out/bin/ \;
    chmod +x $out/bin/wasm-pack

    runHook postInstall
  '';

    meta = with lib; {
      description = "Your favorite rust -> wasm workflow tool";
      homepage = "https://github.com/rustwasm/wasm-pack";
      license = licenses.mit;
      platforms = builtins.attrNames sources.assets;
      mainProgram = "wasm-pack";
    };
}
```
