# Nexus Desktop status

> Only claims backed by evidence are marked verified.

## 1.0.0 — first public release

Nexus Desktop is an AI IDE and home-lab control room for Windows. This release
contains everything built during the 0.1.x private series:

- **IDE:** workspace explorer with ignore-file support, multi-tab editor with
  syntax highlighting, multi-cursor, find/replace, minimap, split view and Git
  gutter; quick open, command palette, workspace search, source control,
  build/test tasks with a Problems list; PowerShell, Command Prompt, WSL and Git
  Bash terminals (ConPTY).
- **AI:** Codex (`codex app-server`) and Claude Code (`claude -p` stream-json)
  in Ask, Plan, Agent and Bypass modes, plus OpenAI-compatible APIs. Visible
  context sharing, secret detection, image and PDF attachments, command
  approvals, hunk-by-hunk review of agent edits, inline edits and resumable
  sessions with task memory.
- **Infrastructure:** dashboard with telemetry and low-resource alerts, devices
  with pinned SSH host keys, services, logs, processes and network probes, SSH
  and SFTP terminals, NixOS dry-build/switch, Debian shadow workspaces, and a
  Cloudflare Access relay.
- **Security:** one Rust policy engine for every remote action; Debian hosts are
  read-only by default and enforced server-side by the `nexus-ro` forced-command
  helper; optional app lock with a 4-digit code from the Nexus iPhone app
  (`docs/OTP.md`).

**Verified:** `cargo test --workspace` passes on Windows for this release.
**Not verified in this release:** live Cloudflare relay and phone pairing need
your own gateway and iPhone app; see `docs/BUILD-AND-RELEASE.md` to configure them.
