//! Android-only pieces. libgit2 verifies HTTPS with its vendored OpenSSL, which knows no
//! certificate authorities there and is built without file access (`no-stdio`), so it can't read
//! a CA file either. At startup each of the system's CA certificates is read here and handed to
//! libgit2 from memory.

use std::fs;
use std::path::{Path, PathBuf};

/// Where Android keeps its CA certificates, one PEM per file: the updatable copy (Android 14+)
/// first, then the system image's.
#[cfg(target_os = "android")]
const CA_DIRS: [&str; 2] = ["/apex/com.android.conscrypt/cacerts", "/system/etc/security/cacerts"];

/// Makes libgit2 trust the system's certificate authorities; returns how many it took.
#[cfg(target_os = "android")]
pub fn trust_system_certs() -> std::io::Result<usize> {
    let dirs = CA_DIRS.map(Path::new);
    let files = cert_files(&dirs).ok_or_else(|| std::io::Error::other("no CA certificates on this device"))?;
    let added = files.iter().filter_map(|f| fs::read(f).ok()).filter(|pem| add_cert(pem)).count();
    Ok(added)
}

/// The certificate files of the first directory that has any.
fn cert_files(dirs: &[&Path]) -> Option<Vec<PathBuf>> {
    dirs.iter().find_map(|dir| {
        let files: Vec<_> = fs::read_dir(dir).ok()?.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect();
        (!files.is_empty()).then_some(files)
    })
}

/// Parses the first PEM certificate in `pem` and adds it to libgit2's trust store.
#[cfg(target_os = "android")]
fn add_cert(pem: &[u8]) -> bool {
    use std::os::raw::c_int;
    use std::ptr::null_mut;
    let Ok(len) = c_int::try_from(pem.len()) else {
        return false;
    };
    // SAFETY: the buffer outlives the BIO, which is freed before returning; the X509 is freed
    // after libgit2 has taken its own reference (X509_STORE_add_cert). Runs once at startup,
    // before any Git work.
    unsafe {
        let bio = openssl_sys::BIO_new_mem_buf(pem.as_ptr().cast(), len);
        if bio.is_null() {
            return false;
        }
        let cert = openssl_sys::PEM_read_bio_X509(bio, null_mut(), None, null_mut());
        openssl_sys::BIO_free_all(bio);
        if cert.is_null() {
            return false;
        }
        libgit2_sys::init();
        let opt = libgit2_sys::GIT_OPT_ADD_SSL_X509_CERT as c_int;
        let rc = libgit2_sys::git_libgit2_opts(opt, cert);
        openssl_sys::X509_free(cert);
        rc == 0
    }
}

#[cfg(test)]
mod tests;
