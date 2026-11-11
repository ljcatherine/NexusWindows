# Build and release

## Toolchain
- Rust stable, MSVC host (`rust-toolchain.toml` tracks `stable`; 1.99.1 at time of writing).
- Visual Studio 2022 Build Tools + Windows 10/11 SDK (linker and resource compiler).
- No NASM/CMake needed: SSH uses the `ring` crypto backend; syntax highlighting uses the
  pure-Rust regex engine.
- WiX Toolset 7 (`dotnet tool install --global wix`) for the MSI. The owner approved
  accepting its OSMF EULA on 2026-11-08 (`wix eula accept wix7`, once per machine).
  Never change the component GUIDs in `installer/nexus.wxs`, or upgrades break.

## Commands
```powershell
cargo test --workspace
cargo build --release -p nexus-app     # target\release\NexusDesktop.exe (icon + version resource embedded)
powershell -ExecutionPolicy Bypass -File scripts\release.ps1 -DryRun
powershell -ExecutionPolicy Bypass -File scripts\release.ps1 -NoMsi
```

## Versioning and releases
- Workspace version in `Cargo.toml`. Each release bumps the patch (`1.0.0 → 1.0.1`);
  major/minor change only with owner approval.
- `scripts/release.ps1` runs tests, builds, packages `NexusDesktop-<v>-x64.exe`, a portable
  zip, optional MSI, `SHA256SUMS.txt`, `build-metadata.json`, tags `v<v>` and publishes a
  GitHub Release on `ljcatherine/NexusWindows`. No GitHub Actions.
- Releases are unsigned; titles say so.

## Your gateway (Cloudflare Access relay)
The public source ships placeholder values. To reach your own Nexus Worker,
set these at build time, for example in `[env]` of `%USERPROFILE%\.cargo\config.toml`
(kept outside the repo):

```toml
[env]
NEXUS_ACCESS_ISSUER = "https://<team>.cloudflareaccess.com"
NEXUS_ACCESS_CLIENT_ID = "<Access application client ID>"
NEXUS_GATEWAY_URL = "https://<worker>.workers.dev"
NEXUS_GATEWAY_HOST = "<worker>.workers.dev"
```

Without them the app builds and runs, but Cloudflare relay sign-in will not work.

## Visual QA
`NEXUS_SCREENSHOT=out.png NEXUS_SCREEN=ide NEXUS_OPEN_DIR=<folder> NEXUS_OPEN_FILE=<rel> NEXUS_TERMINAL=1 NEXUS_FRAMES=400 NexusDesktop.exe`
renders, saves a PNG of its own framebuffer and exits.

## Data locations
- `%APPDATA%\NexusDesktop\settings.json`, `sessions.db`, `audit.jsonl`, `known_hosts.json`
- `%LOCALAPPDATA%\NexusDesktop\recovery\`, `shadows\`, `last-crash.txt`
- Secrets: Windows Credential Manager, service `NexusDesktop`.

## nexus-ro (Linux server helper)
- Build: `rustup target add x86_64-unknown-linux-musl`, then
  `cargo build -p nexus-ro --release --target x86_64-unknown-linux-musl`. The binary is
  static and linked with `rust-lld` (`.cargo/config.toml`), so no Linux toolchain is needed.
- Test in a disposable Linux (WSL), never on Debian:
  `wsl -d Ubuntu-24.04 -- sh scripts/test-nexus-ro.sh <copy of the binary in /tmp>`.
- The release script attaches `nexus-ro-<v>-x86_64-linux-musl` and its SHA-256.
- For real SSH acceptance, run `sh scripts/test-nexus-ro-sshd.sh <helper>`
  as root inside a disposable Ubuntu container with `openssh-server` and
  `openssh-sftp-server` installed. Mount the source and helper read-only, use
  `--network none`, and publish no ports. The script generates temporary keys
  and pins the fixture host key. It checks catalog reads and verifies that
  the restricted key cannot start SFTP or download a file through it.
