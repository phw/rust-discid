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

#[macro_use]
extern crate bitflags;

#[link(name = "discid")]
extern {
    fn discid_new() -> *const size_t;
    fn discid_free(disc: *const size_t);
    // fn discid_read(disc: *const size_t, device: *const c_char) -> c_int;
    fn discid_read_sparse(disc: *const size_t, device: *const c_char, features: c_uint) -> c_int;
    fn discid_put(disc: *const size_t, first: c_int, last: c_int, offsets: *const c_int) -> c_int;
    fn discid_get_error_msg(disc: *const size_t) -> *const c_char;
    fn discid_get_id (disc: *const size_t) -> *const c_char;
    fn discid_get_freedb_id (disc: *const size_t) -> *const c_char;
    fn discid_get_toc_string (disc: *const size_t) -> *const c_char;
    fn discid_get_submission_url (disc: *const size_t) -> *const c_char;
    fn discid_get_first_track_num (disc: *const size_t) -> c_int;
    fn discid_get_last_track_num (disc: *const size_t) -> c_int;
    fn discid_get_sectors (disc: *const size_t) -> c_int;
    fn discid_get_track_offset (disc: *const size_t, track_num: c_int) -> c_int;
    fn discid_get_track_length (disc: *const size_t, track_num: c_int) -> c_int;
    fn discid_get_track_isrc (disc: *const size_t, track_num: c_int) -> *const c_char;
    fn discid_get_mcn (disc: *const size_t) -> *const c_char;
    fn discid_has_feature(feature: c_uint) -> c_int;
    fn discid_get_version_string() -> *const c_char;
    fn discid_get_default_device() -> *const c_char;
}

bitflags! {
    pub struct Features: u32 {
        const READ = 1 << 0;
        const MCN  = 1 << 1;
        const ISRC = 1 << 2;
        const ALL  = Self::READ.bits | Self::MCN.bits | Self::ISRC.bits;
    }
}

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
        DiscId::read_features(device, Features::READ)
    }

    pub fn read_features<S>(device: Option<S>, features: Features) -> Result<DiscId, String>
        where S: Into<String> {
        let disc = DiscId::new();
        let c_device: *const c_char = match device {
            Some(d) => CString::new(d.into()).expect("CString::new failed").into_raw(),
            None    => ptr::null(),
        };
        let status = unsafe { discid_read_sparse(disc.disc, c_device, features.bits()) };
        if status == 0 {
            Err(disc.get_error_msg())
        } else {
            Ok(disc)
        }
    }

    pub fn put(first: i32, offsets: &[i32]) -> Result<DiscId, String> {
        let disc = DiscId::new();
        let last = (offsets.len() - 1) as c_int;
        let status = unsafe { discid_put(disc.disc, first, last, offsets.as_ptr()) };
        if status == 0 {
            Err(disc.get_error_msg())
        } else {
            Ok(disc)
        }
    }

    pub fn has_feature(feature: Features) -> bool {
        let result = unsafe { discid_has_feature(feature.bits()) };
        result == 1
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

    pub fn get_freedb_id(&self) -> String {
        let str_ptr = unsafe { discid_get_freedb_id(self.disc) };
        to_str(str_ptr)
    }

    pub fn get_toc_string(&self) -> String {
        let str_ptr = unsafe { discid_get_toc_string(self.disc) };
        to_str(str_ptr)
    }

    pub fn get_submission_url(&self) -> String {
        let str_ptr = unsafe { discid_get_submission_url(self.disc) };
        to_str(str_ptr)
    }

    pub fn get_first_track_num(&self) -> i32 {
        unsafe { discid_get_first_track_num(self.disc) }
    }

    pub fn get_last_track_num(&self) -> i32 {
        unsafe { discid_get_last_track_num(self.disc) }
    }

    pub fn get_sectors(&self) -> i32 {
        unsafe { discid_get_sectors(self.disc) }
    }

    pub fn get_track_offset(&self, track_num: i32) -> i32 {
        unsafe { discid_get_track_offset(self.disc, track_num) }
    }

    pub fn get_track_length(&self, track_num: i32) -> i32 {
        unsafe { discid_get_track_length(self.disc, track_num) }
    }

    pub fn get_mcn(&self) -> String {
        let str_ptr = unsafe { discid_get_mcn(self.disc) };
        to_str(str_ptr)
    }

    pub fn get_track_isrc(&self, track_num: i32) -> String {
        let str_ptr = unsafe { discid_get_track_isrc(self.disc, track_num) };
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
    use super::{DiscId, Features};

    #[test]
    fn test_put() {
        let first = 1;
        let offsets = [206535, 150, 18901, 39738, 59557, 79152, 100126,
                       124833, 147278, 166336, 182560];
        let disc = DiscId::put(first, &offsets).expect("DiscId::put failed");
        let last_track = disc.get_last_track_num();
        assert_eq!("Wn8eRBtfLDfM0qjYPdxrz.Zjs_U-", disc.get_id());
        assert_eq!("830abf0a", disc.get_freedb_id());
        assert_eq!(1, disc.get_first_track_num());
        assert_eq!(10, last_track);
        assert_eq!(206535, disc.get_sectors());
        assert_eq!(
            "1 10 206535 150 18901 39738 59557 79152 100126 124833 147278 166336 182560",
            disc.get_toc_string());
        assert_eq!(
            "http://musicbrainz.org/cdtoc/attach?id=Wn8eRBtfLDfM0qjYPdxrz.Zjs_U-&tracks=10&toc=1+10+206535+150+18901+39738+59557+79152+100126+124833+147278+166336+182560",
            disc.get_submission_url());
        for i in first..last_track+1 {
            let offset = offsets[i as usize];
            let next = if i < last_track { i + 1 } else { 0 };
            let length = offsets[next as usize] - offset;
            assert_eq!(offset, disc.get_track_offset(i), "track {} expected offset {}", i, offset);
            assert_eq!(length, disc.get_track_length(i), "track {} expected length {}", i, length);
        }
    }

    #[test]
    fn test_get_default_device() {
        let device = DiscId::get_default_device();
        assert!(!device.is_empty());
    }

    #[test]
    fn test_has_feature() {
        assert_eq!(true, DiscId::has_feature(Features::READ));
    }

    #[test]
    fn test_get_version_string() {
        let version = DiscId::get_version_string();
        assert!(version.starts_with("libdiscid"));
    }
}
