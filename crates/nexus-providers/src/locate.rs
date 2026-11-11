//! Locate official CLI binaries.

use std::path::{Path, PathBuf};

fn on_path(names: &[&str]) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        for n in names {
            let p = dir.join(n);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

fn home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from)
}

/// `codex` CLI: PATH, then the official Windows installer location.
pub fn codex() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("NEXUS_CODEX_PATH").map(PathBuf::from).filter(|p| p.is_file()) {
        return Some(p);
    }
    on_path(&["codex.exe", "codex.cmd", "codex"]).or_else(|| {
        let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from)?;
        let p = local.join("Programs").join("OpenAI").join("Codex").join("bin").join("codex.exe");
        p.is_file().then_some(p)
    })
}

/// Standalone Claude Code CLI: env override, PATH, `~/.local/bin`, npm global,
/// and finally the copy bundled with the Claude desktop app (newest version).
pub fn claude() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("NEXUS_CLAUDE_PATH").map(PathBuf::from).filter(|p| p.is_file()) {
        return Some(p);
    }
    if let Some(p) = on_path(&["claude.exe", "claude.cmd", "claude"]) {
        return Some(p);
    }
    let h = home()?;
    for p in [h.join(".local").join("bin").join("claude.exe"), h.join("AppData").join("Roaming").join("npm").join("claude.cmd")] {
        if p.is_file() {
            return Some(p);
        }
    }
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from)?;
    newest_bundled(&local.join("Claude-Data"))
}

fn newest_bundled(root: &Path) -> Option<PathBuf> {
    let mut found: Vec<(Vec<u32>, PathBuf)> = Vec::new();
    for app in std::fs::read_dir(root).ok()?.flatten() {
        let cc = app.path().join("claude-code");
        let Ok(versions) = std::fs::read_dir(&cc) else { continue };
        for v in versions.flatten() {
            let ver: Vec<u32> = v.file_name().to_string_lossy().split('.').filter_map(|x| x.parse().ok()).collect();
            let Ok(builds) = std::fs::read_dir(v.path()) else { continue };
            for b in builds.flatten() {
                let exe = b.path().join("claude.exe");
                if exe.is_file() {
                    found.push((ver.clone(), exe));
                }
            }
        }
    }
    found.sort();
    found.pop().map(|(_, p)| p)
}

/// Command builder that never opens a console window.
pub fn command(program: &Path) -> std::process::Command {
    let mut c = if program.extension().is_some_and(|e| e.eq_ignore_ascii_case("cmd")) {
        let mut c = std::process::Command::new("cmd");
        c.arg("/C").arg(program);
        c
    } else {
        std::process::Command::new(program)
    };
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000);
    }
    scrub_agent_env(&mut c);
    c
}

/// Environment variables removed from every provider (agent) process so it
/// does not inherit an SSH agent socket or a Git SSH override from Nexus's
/// environment. This does not hide the Windows OpenSSH agent service, which
/// uses a fixed named pipe rather than an environment variable.
pub const SCRUBBED_ENV: [&str; 5] = ["SSH_AUTH_SOCK", "SSH_AGENT_PID", "GIT_SSH", "GIT_SSH_COMMAND", "GIT_ASKPASS"];

pub fn scrub_agent_env(c: &mut std::process::Command) {
    for k in SCRUBBED_ENV {
        c.env_remove(k);
    }
}

#[cfg(test)]
mod scrub_tests {
    #[test]
    fn provider_commands_drop_ssh_agent_variables() {
        let c = super::command(std::path::Path::new("codex.exe"));
        for k in super::SCRUBBED_ENV {
            assert!(c.get_envs().any(|(n, v)| n == k && v.is_none()), "{k} not removed");
        }
    }
}
