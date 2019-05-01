// Copyright (C) 2019 Philipp Wolfer <ph.wolfer@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use libc::{c_int, c_uint, size_t};
use std::ptr;

#[link(name = "discid")]
extern {
    fn discid_new() -> *const size_t;
    fn discid_free(disc: *const size_t);
    // fn discid_read(disc: *const size_t, device: *const c_char) -> c_int;
    fn discid_read_sparse(disc: *const size_t, device: *const c_char, features: c_uint) -> c_int;
    fn discid_put(disc: *const size_t, first: c_int, last: c_int, offsets: *const c_int) -> c_int;
    fn discid_get_error_msg(disc: *const size_t) -> *const c_char;
    fn discid_get_id (disc: *const size_t) -> *const c_char;
    fn discid_get_mcn (disc: *const size_t) -> *const c_char;
    fn discid_get_version_string() -> *const c_char;
    fn discid_get_default_device() -> *const c_char;
}

pub const FEATURE_READ: u32 = 1 << 0;
pub const FEATURE_MCN:  u32 = 1 << 1;
pub const FEATURE_ISRC: u32 = 1 << 2;

pub struct DiscId {
    disc: *const size_t,
}

impl DiscId {
    fn new() -> DiscId {
        let disc = unsafe { discid_new() };
        DiscId { disc}
    }

    pub fn read<S>(device: Option<S>) -> Result<DiscId, String>
        where S: Into<String> {
        DiscId::read_features(device, FEATURE_READ)
    }

    pub fn read_features<S>(device: Option<S>, features: u32) -> Result<DiscId, String>
        where S: Into<String> {
        let disc = DiscId::new();
        let c_device: *const c_char = match device {
            Some(d) => CString::new(d.into()).expect("CString::new failed").into_raw(),
            None    => ptr::null(),
        };
        let status = unsafe { discid_read_sparse(disc.disc, c_device, features) };
        if status == 0 {
            Err(disc.get_error_msg())
        } else {
            Ok(disc)
        }
    }

    pub fn put(first: i32, offsets: &[i32]) -> Result<DiscId, String> {
        let disc = DiscId::new();
        let last = (offsets.len() - 1) as i32;
        let status = unsafe { discid_put(disc.disc, first, last, offsets.as_ptr()) };
        if status == 0 {
            Err(disc.get_error_msg())
        } else {
            Ok(disc)
        }
    }

    pub fn get_version_string() -> String {
        let str_ptr = unsafe { discid_get_version_string() };
        to_str(str_ptr)
    }

    pub fn get_default_device() -> String {
        let version_ptr = unsafe { discid_get_default_device() };
        to_str(version_ptr)
    }

    pub fn get_error_msg(&self) -> String {
        let str_ptr = unsafe { discid_get_error_msg(self.disc) };
        to_str(str_ptr)
    }

    pub fn get_id(&self) -> String {
        let str_ptr = unsafe { discid_get_id(self.disc) };
        to_str(str_ptr)
    }

    pub fn get_mcn(&self) -> String {
        let str_ptr = unsafe { discid_get_mcn(self.disc) };
        to_str(str_ptr)
    }
}

impl Drop for DiscId {
    fn drop(&mut self) {
        unsafe { discid_free(self.disc) }
    }
}

fn to_str(c_buf: *const c_char) -> String {
    let c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
    let str_slice: &str = c_str.to_str().unwrap();
    str_slice.to_owned()
}

#[cfg(test)]
mod tests {
    use super::DiscId;

    #[test]
    fn test_put() {
        let first = 1;
        let offsets = [206535, 150, 18901, 39738, 59557, 79152, 100126,
                       124833, 147278, 166336, 182560];
        let disc = DiscId::put(first, &offsets).expect("DiscId::put failed");
        assert_eq!("Wn8eRBtfLDfM0qjYPdxrz.Zjs_U-", disc.get_id());
    }

    #[test]
    fn test_get_default_device() {
        let device = DiscId::get_default_device();
        assert!(!device.is_empty());
    }

    #[test]
    fn test_get_version_string() {
        let version = DiscId::get_version_string();
        assert!(version.starts_with("libdiscid"));
    }
}
