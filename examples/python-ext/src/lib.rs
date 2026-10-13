//! A Python extension built by maturin for an older glibc with cargo-libc.

use pyo3::prelude::*;
use std::ffi::{c_char, CStr};

extern "C" {
    fn gnu_get_libc_version() -> *const c_char;
}

/// The glibc release the extension runs on.
#[pyfunction]
fn runtime_glibc() -> String {
    // SAFETY: glibc returns a static NUL-terminated string.
    unsafe { CStr::from_ptr(gnu_get_libc_version()) }
        .to_string_lossy()
        .into_owned()
}

/// Sums 1..=n on four threads, then writes and reads back a file: threads,
/// thread-local storage, unwinding tables and file metadata from the extension.
#[pyfunction]
fn work(n: u64, path: std::path::PathBuf) -> PyResult<u64> {
    let handles: Vec<_> = (0..4u64)
        .map(|part| {
            std::thread::spawn(move || (1..=n).filter(|i| i % 4 == part).sum::<u64>())
        })
        .collect();
    let total = handles.into_iter().map(|h| h.join().unwrap()).sum::<u64>();
    std::fs::write(&path, total.to_string())?;
    let len = std::fs::metadata(&path)?.len();
    let back: u64 = std::fs::read_to_string(&path)?.parse().unwrap();
    assert_eq!(len as usize, back.to_string().len());
    Ok(back)
}

#[pymodule]
fn glibc_python_ext(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(runtime_glibc, m)?)?;
    m.add_function(wrap_pyfunction!(work, m)?)?;
    Ok(())
}
