//! The running macOS version, for features whose system APIs are newer than
//! Hex's macOS 13 floor: they turn themselves off on older systems instead of
//! crashing on a missing class or symbol.

use std::ffi::{CStr, c_char, c_int, c_void};
use std::sync::OnceLock;

unsafe extern "C" {
    fn sysctlbyname(
        name: *const c_char,
        old: *mut c_void,
        old_len: *mut usize,
        new: *const c_void,
        new_len: usize,
    ) -> c_int;
}

/// `(major, minor)`, or `(0, 0)` when macOS doesn't say.
fn version() -> (u32, u32) {
    static VERSION: OnceLock<(u32, u32)> = OnceLock::new();
    *VERSION.get_or_init(|| {
        let mut buffer = [0u8; 32];
        let mut len = buffer.len();
        // SAFETY: `buffer` holds `len` writable bytes; nothing is written back.
        let status = unsafe {
            sysctlbyname(
                c"kern.osproductversion".as_ptr(),
                buffer.as_mut_ptr().cast(),
                &mut len,
                std::ptr::null(),
                0,
            )
        };
        if status != 0 {
            return (0, 0);
        }
        CStr::from_bytes_until_nul(&buffer)
            .ok()
            .and_then(|text| text.to_str().ok())
            .map(parse)
            .unwrap_or((0, 0))
    })
}

fn parse(text: &str) -> (u32, u32) {
    let mut parts = text.split('.').map(|part| part.parse().unwrap_or(0));
    (parts.next().unwrap_or(0), parts.next().unwrap_or(0))
}

pub(crate) fn at_least(major: u32, minor: u32) -> bool {
    version() >= (major, minor)
}

/// Core Audio process taps (`CATapDescription`, `AudioHardwareCreateProcessTap`):
/// meetings' system audio without Screen Recording, and the browser pause.
pub(crate) fn process_taps() -> bool {
    at_least(14, 2)
}

/// Annotate: `SCRecordingOutput` (macOS 15) records, `SCScreenshotManager`
/// (macOS 14) takes its stills, and the helper mirrors an iPhone through
/// macOS 14's external camera devices.
pub(crate) fn annotate() -> bool {
    at_least(15, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_product_versions() {
        assert_eq!(parse("13.7.8"), (13, 7));
        assert_eq!(parse("15.0"), (15, 0));
        assert_eq!(parse("26"), (26, 0));
    }

    #[test]
    fn reads_the_running_system() {
        assert!(at_least(13, 0));
    }
}
