//! Cloudflare Access sign-in state for the app.
//!
//! Flow: register the per-user `com.ljcatherine.Nexus:` URL scheme (pointing at
//! this exe) → discover metadata → open the system browser at the Access
//! authorize URL → the Nexus Worker's `/oauth2redirect` 302s to the scheme →
//! Windows starts a second Nexus process with the URL, which writes it to
//! `%LOCALAPPDATA%\NexusDesktop\oauth-callback.txt` and exits → this module
//! checks the state, exchanges the code and stores the session (DPAPI).

use crate::app::Toast;
use crate::settings::local_dir;
use nexus_access::{store, Metadata, Pkce, Session};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub fn callback_path() -> PathBuf {
    local_dir().join("oauth-callback.txt")
}

/// If launched by the URL scheme, hand the URL to the running instance.
/// Returns true when the process should exit.
pub fn handle_launch_args() -> bool {
    let prefix = format!("{}:", nexus_access::CALLBACK_SCHEME.to_ascii_lowercase());
    for a in std::env::args().skip(1) {
        if a.to_ascii_lowercase().starts_with(&prefix) && a.len() < 12_000 {
            let _ = nexus_workspaces::write_atomic(&callback_path(), a.as_bytes());
            return true;
        }
    }
    false
}

/// Per-user URL scheme registration (HKCU\Software\Classes). No admin rights.
#[cfg(windows)]
pub fn register_scheme() -> Result<(), String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let base = format!("Software\\Classes\\{}", nexus_access::CALLBACK_SCHEME);
    let (k, _) = hkcu.create_subkey(&base).map_err(|e| e.to_string())?;
    k.set_value("", &"URL:Nexus Desktop sign-in").map_err(|e| e.to_string())?;
    k.set_value("URL Protocol", &"").map_err(|e| e.to_string())?;
    let (cmd, _) = hkcu.create_subkey(format!("{base}\\shell\\open\\command")).map_err(|e| e.to_string())?;
    cmd.set_value("", &format!("\"{}\" \"%1\"", exe.display())).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(windows))]
pub fn register_scheme() -> Result<(), String> {
    Err("URL scheme registration is only implemented on Windows".into())
}

fn open_browser(url: &str) -> Result<(), String> {
    std::process::Command::new("rundll32").args(["url.dll,FileProtocolHandler", url]).spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// Shared handle used by background SSH tasks to get a fresh token.
#[derive(Clone)]
pub struct AccessHandle {
    session: Arc<Mutex<Option<Session>>>,
    meta: Arc<Mutex<Option<Metadata>>>,
    /// False when the session must stay in memory only.
    persist: Arc<AtomicBool>,
}

impl AccessHandle {
    pub fn signed_in(&self) -> bool {
        self.session.lock().unwrap().is_some()
    }

    /// Blocking: returns a token valid for at least two minutes, refreshing if needed.
    pub fn fresh_token(&self) -> Result<String, String> {
        let cur = self.session.lock().unwrap().clone().ok_or("Sign in to Cloudflare Access in Settings first")?;
        if cur.is_fresh(120) {
            return Ok(cur.access_token);
        }
        let meta = match self.meta.lock().unwrap().clone() {
            Some(m) => m,
            None => nexus_access::discover().map_err(|e| e.to_string())?,
        };
        *self.meta.lock().unwrap() = Some(meta.clone());
        match nexus_access::refresh(&meta, &cur) {
            Ok(s) => {
                if self.persist.load(Ordering::SeqCst) {
                    let _ = store::save(&store::default_path(), &s);
                }
                let tok = s.access_token.clone();
                *self.session.lock().unwrap() = Some(s);
                Ok(tok)
            }
            Err(e) => Err(format!("Cloudflare Access session expired ({e}); sign in again in Settings")),
        }
    }
}

enum Msg {
    Meta(Result<Metadata, String>),
    Session(Result<Session, String>),
}

pub struct Access {
    pub handle: AccessHandle,
    pending: Option<(Pkce, Metadata, Instant)>,
    pub busy: bool,
    pub last_error: Option<String>,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
    wake: Arc<dyn Fn() + Send + Sync>,
}

impl Access {
    pub fn new(wake: Arc<dyn Fn() + Send + Sync>, memory_only: bool) -> Self {
        let (tx, rx) = channel();
        let session = if memory_only {
            store::clear(&store::default_path());
            None
        } else {
            store::load(&store::default_path())
        };
        let handle = AccessHandle { session: Arc::new(Mutex::new(session)), meta: Arc::default(), persist: Arc::new(AtomicBool::new(!memory_only)) };
        Self { handle, pending: None, busy: false, last_error: None, tx, rx, wake }
    }

    pub fn memory_only(&self) -> bool {
        !self.handle.persist.load(Ordering::SeqCst)
    }

    /// Switch between disk (DPAPI) and memory-only storage. Turning memory-only
    /// on deletes the file; the current sign-in stays valid until Nexus closes.
    pub fn set_memory_only(&self, on: bool) -> Result<(), String> {
        self.handle.persist.store(!on, Ordering::SeqCst);
        let path = store::default_path();
        if on {
            store::clear(&path);
            if path.exists() {
                return Err(format!("Could not delete {}", path.display()));
            }
            Ok(())
        } else if let Some(s) = self.handle.session.lock().unwrap().as_ref() {
            store::save(&path, s)
        } else {
            Ok(())
        }
    }

    pub fn signed_in(&self) -> bool {
        self.handle.session.lock().unwrap().is_some()
    }

    pub fn status(&self) -> String {
        match self.handle.session.lock().unwrap().as_ref() {
            Some(s) if s.is_fresh(0) => format!("Signed in · token valid for {} min", s.expires_in() / 60),
            Some(s) if s.refresh_token.is_some() => "Signed in · token will refresh on next use".into(),
            Some(_) => "Session expired · sign in again".into(),
            None if self.pending.is_some() => "Waiting for browser sign-in…".into(),
            None => "Not signed in".into(),
        }
    }

    pub fn waiting(&self) -> bool {
        self.pending.is_some()
    }

    pub fn start_sign_in(&mut self) -> Result<(), String> {
        register_scheme()?;
        let _ = std::fs::remove_file(callback_path());
        self.busy = true;
        self.last_error = None;
        let tx = self.tx.clone();
        let wake = self.wake.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Meta(nexus_access::discover().map_err(|e| e.to_string())));
            wake();
        });
        Ok(())
    }

    pub fn cancel(&mut self) {
        self.pending = None;
        self.busy = false;
    }

    pub fn sign_out(&mut self) {
        let s = self.handle.session.lock().unwrap().take();
        store::clear(&store::default_path());
        if let Some(s) = s {
            let meta = self.handle.meta.lock().unwrap().clone();
            std::thread::spawn(move || {
                if let Ok(m) = meta.map(Ok).unwrap_or_else(nexus_access::discover) {
                    let _ = nexus_access::revoke(&m, &s);
                }
            });
        }
    }

    pub fn poll(&mut self, toasts: &mut Vec<Toast>, ok: egui::Color32, err: egui::Color32) {
        while let Ok(m) = self.rx.try_recv() {
            match m {
                Msg::Meta(Ok(meta)) => {
                    *self.handle.meta.lock().unwrap() = Some(meta.clone());
                    let pkce = Pkce::new();
                    let url = nexus_access::authorization_url(&meta, &pkce);
                    match open_browser(&url) {
                        Ok(()) => self.pending = Some((pkce, meta, Instant::now())),
                        Err(e) => self.last_error = Some(format!("Could not open the browser: {e}")),
                    }
                    self.busy = false;
                }
                Msg::Meta(Err(e)) | Msg::Session(Err(e)) => {
                    self.busy = false;
                    self.pending = None;
                    toasts.push(Toast { text: format!("Cloudflare sign-in failed: {e}"), color: err, at: Instant::now() });
                    self.last_error = Some(e);
                }
                Msg::Session(Ok(s)) => {
                    self.busy = false;
                    let saved = if self.memory_only() { Ok(()) } else { store::save(&store::default_path(), &s) };
                    match saved {
                        Ok(()) => toasts.push(Toast { text: "Signed in to Cloudflare Access".into(), color: ok, at: Instant::now() }),
                        Err(e) => toasts.push(Toast { text: format!("Signed in, but saving the session failed: {e}"), color: err, at: Instant::now() }),
                    }
                    *self.handle.session.lock().unwrap() = Some(s);
                }
            }
        }
        // Callback from the browser via the URL scheme.
        if let Some((pkce, meta, started)) = &self.pending {
            if started.elapsed().as_secs() > 600 {
                self.pending = None;
                self.last_error = Some("Sign-in timed out".into());
                return;
            }
            if let Ok(url) = std::fs::read_to_string(callback_path()) {
                let _ = std::fs::remove_file(callback_path());
                let (pkce, meta) = (pkce.clone(), meta.clone());
                self.pending = None;
                match nexus_access::parse_callback(&url, &pkce.state) {
                    Ok(code) => {
                        self.busy = true;
                        let tx = self.tx.clone();
                        let wake = self.wake.clone();
                        std::thread::spawn(move || {
                            let _ = tx.send(Msg::Session(nexus_access::exchange(&meta, &code, &pkce).map_err(|e| e.to_string())));
                            wake();
                        });
                    }
                    Err(e) => {
                        self.last_error = Some(e.to_string());
                        toasts.push(Toast { text: format!("Cloudflare sign-in failed: {e}"), color: err, at: Instant::now() });
                    }
                }
            }
        }
    }
}
