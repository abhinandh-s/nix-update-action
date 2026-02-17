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
