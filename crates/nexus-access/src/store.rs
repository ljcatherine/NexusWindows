//! DPAPI-protected session storage (`%LOCALAPPDATA%\NexusDesktop\access-session.bin`).
//! Windows Credential Manager limits secrets to ~2.5 KB, too small for OAuth
//! tokens, so the session JSON is encrypted with `CryptProtectData` for the
//! current user and only that user on this PC can decrypt it.

use crate::Session;
use std::path::{Path, PathBuf};

pub fn default_path() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("NexusDesktop").join("access-session.bin")
}

#[cfg(windows)]
mod dpapi {
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB};

    const ENTROPY: &[u8] = b"NexusDesktop.CloudflareAccess.v1";

    fn blob(b: &[u8]) -> CRYPT_INTEGER_BLOB {
        CRYPT_INTEGER_BLOB { cbData: b.len() as u32, pbData: b.as_ptr() as *mut u8 }
    }

    unsafe fn take(out: CRYPT_INTEGER_BLOB) -> Vec<u8> {
        let v = unsafe { std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec() };
        unsafe { LocalFree(out.pbData as _) };
        v
    }

    pub fn protect(data: &[u8]) -> Option<Vec<u8>> {
        let input = blob(data);
        let entropy = blob(ENTROPY);
        let mut out = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
        let ok = unsafe { CryptProtectData(&input, std::ptr::null(), &entropy, std::ptr::null(), std::ptr::null(), CRYPTPROTECT_UI_FORBIDDEN, &mut out) };
        if ok == 0 { None } else { Some(unsafe { take(out) }) }
    }

    pub fn unprotect(data: &[u8]) -> Option<Vec<u8>> {
        let input = blob(data);
        let entropy = blob(ENTROPY);
        let mut out = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
        let ok = unsafe { CryptUnprotectData(&input, std::ptr::null_mut(), &entropy, std::ptr::null(), std::ptr::null(), CRYPTPROTECT_UI_FORBIDDEN, &mut out) };
        if ok == 0 { None } else { Some(unsafe { take(out) }) }
    }
}

#[cfg(not(windows))]
mod dpapi {
    pub fn protect(_: &[u8]) -> Option<Vec<u8>> {
        None
    }
    pub fn unprotect(_: &[u8]) -> Option<Vec<u8>> {
        None
    }
}

pub fn save(path: &Path, s: &Session) -> Result<(), String> {
    let json = serde_json::to_vec(s).map_err(|e| e.to_string())?;
    let enc = dpapi::protect(&json).ok_or("DPAPI encryption failed")?;
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, enc).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

pub fn load(path: &Path) -> Option<Session> {
    let enc = std::fs::read(path).ok()?;
    let json = dpapi::unprotect(&enc)?;
    serde_json::from_slice(&json).ok()
}

pub fn clear(path: &Path) {
    let _ = std::fs::remove_file(path);
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn dpapi_roundtrip_and_ciphertext_hides_token() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("s.bin");
        let s = Session { access_token: "secret-access-token-123".into(), refresh_token: Some("rt".into()), expires_at: 42 };
        save(&p, &s).unwrap();
        let raw = std::fs::read(&p).unwrap();
        assert!(!raw.windows(6).any(|w| w == b"secret"), "token must not appear in plaintext");
        assert_eq!(load(&p).unwrap(), s);
        clear(&p);
        assert!(load(&p).is_none());
    }
}
