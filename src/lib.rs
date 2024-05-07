use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::c_char;
use std::ptr;

mod devices;
mod owcapi;

pub use devices::{Device, DeviceInfo};

#[derive(Debug)]
pub struct Error {
    code: isize,
}

impl From<std::ffi::NulError> for Error {
    fn from(_err: std::ffi::NulError) -> Self {
        Self::new(0)
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(_err: std::num::TryFromIntError) -> Self {
        Self::new(1)
    }
}

impl Error {
    #[inline]
    pub fn new(code: isize) -> Self {
        Self { code }
    }
    #[inline]
    pub fn code(&self) -> isize {
        self.code
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OWFS error: {}", self.code)
    }
}

impl std::error::Error for Error {}

/// # Safety
///
/// Consider with libowcapi safety
pub unsafe fn init(path: &str) -> Result<(), Error> {
    let c_path = CString::new(path)?;
    let res = owcapi::OW_init(c_path.as_ptr());
    if res == 0 {
        Ok(())
    } else {
        Err(Error::new(res))
    }
}

/// # Safety
///
/// Consider with libowcapi safety
pub unsafe fn get(path: &str) -> Result<String, Error> {
    let c_path = CString::new(path)?;
    let mut buf: *mut c_char = ptr::null_mut();
    let buf_ptr: *const *mut c_char = &mut buf;
    let mut buf_length: usize = 0;
    let res = owcapi::OW_get(c_path.as_ptr(), buf_ptr, &mut buf_length);
    if res >= 0 {
        let data = CStr::from_ptr(buf);
        let result = data.to_string_lossy().to_string();
        libc::free(buf.cast::<libc::c_void>());
        Ok(result)
    } else {
        libc::free(buf.cast::<libc::c_void>());
        Err(Error::new(res))
    }
}

/// # Safety
///
/// Consider with libowcapi safety
pub unsafe fn get_bytes(path: &str) -> Result<Vec<u8>, Error> {
    let c_path = CString::new(path)?;
    let mut buf: *mut c_char = ptr::null_mut();
    let buf_ptr: *const *mut c_char = &mut buf;
    let mut buf_length: usize = 0;
    let res = owcapi::OW_get(c_path.as_ptr(), buf_ptr, &mut buf_length);
    if res >= 0 {
        let result = std::slice::from_raw_parts(buf.cast::<u8>(), buf_length).to_vec();
        libc::free(buf.cast::<libc::c_void>());
        Ok(result)
    } else {
        libc::free(buf.cast::<libc::c_void>());
        Err(Error::new(res))
    }
}

/// # Safety
///
/// Consider with libowcapi safety
pub unsafe fn set(path: &str, value: &str) -> Result<(), Error> {
    let c_path = CString::new(path)?;
    let c_val = CString::new(value)?;
    let len_i: isize = c_val.as_bytes_with_nul().len().try_into()?;
    #[allow(clippy::cast_sign_loss)]
    let res = owcapi::OW_put(c_path.as_ptr(), c_val.as_ptr(), len_i as usize);
    if res == len_i {
        Ok(())
    } else {
        Err(Error::new(-2))
    }
}

/// # Safety
///
/// Consider with libowcapi safety
pub unsafe fn set_bytes(path: &str, value: &[u8]) -> Result<(), Error> {
    let c_path = CString::new(path)?;
    let len_i: isize = value.len().try_into()?;
    #[allow(clippy::cast_sign_loss)]
    let res = owcapi::OW_put(
        c_path.as_ptr(),
        value.as_ptr().cast::<c_char>(),
        len_i as usize,
    );
    if res == len_i {
        Ok(())
    } else {
        Err(Error::new(-2))
    }
}

#[derive(Default)]
pub struct ScanOptions<'a> {
    types: Option<HashSet<&'a str>>,
    attrs_any: Option<HashSet<&'a str>>,
    attrs_all: Option<HashSet<&'a str>>,
}

impl<'a> ScanOptions<'a> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    #[inline]
    pub fn types(mut self, types: &'a [&'a str]) -> Self {
        self.types = Some(types.iter().copied().collect());
        self
    }
    #[inline]
    pub fn attrs_all(mut self, attrs_all: &'a [&'a str]) -> Self {
        self.attrs_all = Some(attrs_all.iter().copied().collect());
        self
    }
    #[inline]
    pub fn attrs_any(mut self, attrs_any: &'a [&'a str]) -> Self {
        self.attrs_any = Some(attrs_any.iter().copied().collect());
        self
    }
    unsafe fn matches(&self, dev: &Device) -> bool {
        if let Some(ref types) = self.types {
            if let Ok(tp) = dev.get("type") {
                if !types.contains(&tp.as_str()) {
                    return false;
                }
            } else {
                return false;
            }
        }
        let dev_attrs: Option<HashSet<&str>> =
            if self.attrs_all.is_some() || self.attrs_any.is_some() {
                Some(dev.attrs().iter().copied().collect())
            } else {
                None
            };
        if let Some(ref attrs_all) = self.attrs_all {
            let d_attrs = dev_attrs.as_ref().unwrap();
            for a in attrs_all {
                if !d_attrs.contains(a) {
                    return false;
                }
            }
        }
        if let Some(ref attrs_any) = self.attrs_any {
            let d_attrs = dev_attrs.as_ref().unwrap();
            let mut found = false;
            for a in attrs_any {
                if d_attrs.contains(a) {
                    found = true;
                }
            }
            if !found {
                return false;
            }
        }
        true
    }
}

/// # Safety
///
/// Consider with libowcapi safety
pub unsafe fn scan(options: ScanOptions) -> Result<Vec<Device>, Error> {
    let data = get("/uncached/")?;
    let mut result = Vec::new();
    for el in data.split(',') {
        if let Some(ch) = el.chars().next() {
            if ch.is_ascii_digit() || ('A'..='F').contains(&ch) {
                if let Some(el_path) = el.strip_suffix('/') {
                    let mut dev = Device::new(el_path);
                    if dev.load().is_ok() && options.matches(&dev) {
                        result.push(dev);
                    }
                }
            }
        }
    }
    Ok(result)
}

#[inline]
/// # Safety
///
/// Consider with libowcapi safety
pub unsafe fn finish() {
    owcapi::OW_finish();
}
