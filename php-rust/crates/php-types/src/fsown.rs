//! `chown`/`chgrp`/`lchown`/`lchgrp` sulla libc di sistema (WP-16), come
//! ext/standard/filestat.c: un nome si risolve via getpwnam/getgrnam, l'id
//! numerico passa diretto; -1 = "non cambiare" (semantica POSIX di chown).

use std::ffi::CString;

/// uid per un nome utente, `None` se sconosciuto (o nome con NUL).
pub fn resolve_uid(name: &[u8]) -> Option<u32> {
    let c = CString::new(name).ok()?;
    // SAFETY: getpwnam su CString valida; il puntatore è letto subito.
    let pw = unsafe { libc::getpwnam(c.as_ptr()) };
    if pw.is_null() { None } else { Some(unsafe { (*pw).pw_uid }) }
}

/// gid per un nome gruppo, `None` se sconosciuto.
pub fn resolve_gid(name: &[u8]) -> Option<u32> {
    let c = CString::new(name).ok()?;
    // SAFETY: getgrnam su CString valida; il puntatore è letto subito.
    let gr = unsafe { libc::getgrnam(c.as_ptr()) };
    if gr.is_null() { None } else { Some(unsafe { (*gr).gr_gid }) }
}

/// chown(2)/lchown(2): `uid`/`gid` a -1 lasciano invariato quel lato.
pub fn change_owner(path: &[u8], uid: i64, gid: i64, follow: bool) -> Result<(), std::io::Error> {
    let c = CString::new(path)
        .map_err(|_| std::io::Error::from_raw_os_error(libc::ENOENT))?;
    // SAFETY: path CString valida; -1 as uid_t/gid_t = "don't change" POSIX.
    let rc = unsafe {
        if follow {
            libc::chown(c.as_ptr(), uid as libc::uid_t, gid as libc::gid_t)
        } else {
            libc::lchown(c.as_ptr(), uid as libc::uid_t, gid as libc::gid_t)
        }
    };
    if rc == 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) }
}
