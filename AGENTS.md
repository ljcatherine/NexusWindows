# Nexus Desktop engineering rules

Nexus Desktop is a native Rust Windows companion to the Nexus iOS app. It has
its own build, versioning and releases.

## Security rules (highest priority)
- Hosts with the **Debian** role are read-only by default. Enforcement lives in
  `crates/nexus-security`: every remote operation must pass through
  `PolicyEngine`; never add a code path that bypasses it. Gateway routes that
  mutate Debian-hosted state are denied by default.
- The owner can unlock Debian writes for 30 minutes (`PolicyEngine::unlock_debian`),
  over direct SSH with a separate passphrase key Nexus never stores. It never
  applies to AI agents in any mode, never to the gateway, and delete/service
  control still need single-use escalations.
- Debian terminals use the fixed observational command catalog only: no
  arbitrary shell, no user/model-supplied arguments interpolated into commands.
- Remote Debian workspaces open read-only; edits go to a local shadow workspace.
- Tests for policy use mocks or disposable environments, never a live server.

## Engineering
- Rust stable (see `rust-toolchain.toml`), workspace in `crates/`.
- Keep transport, domain and UI separate. UI crates must not perform remote I/O
  directly; they go through service crates that enforce policy.
- No fabricated telemetry, provider status or test results. A provider is only
  "connected" after authentication and a real supported operation are verified.
- Use the official Codex (`codex app-server`) and Claude Code (`claude -p`
  stream-json) integrations. Never scrape, extract cookies or impersonate clients.
- Never disable SSH host-key or TLS verification.
- Credentials go in Windows Credential Manager, never in source.
- Deployment-specific values (Cloudflare Access issuer and client ID, gateway
  host) are build-time environment variables; see `docs/BUILD-AND-RELEASE.md`.

## Releases
- Every Windows release is published as a GitHub Release (MSI, portable zip,
  exe and `SHA256SUMS.txt`) by `scripts/release.ps1`, using the version in the
  workspace `Cargo.toml`. No CI builds releases.
