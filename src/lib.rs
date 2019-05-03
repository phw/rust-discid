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

//! `discid` allows you to calculate MusicBrainz or FreeDB disc IDs for audio CDs.
//!
//! `discid` provides bindings to the MusicBrainz [libdiscid](https://musicbrainz.org/doc/libdiscid)
//! library. In addition to calculating the disc IDs you can also get advanced data from the
//! audio CD such as MCN (media catalogue number) and per-track ISRCs.

#![deny(
    // missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use libc::{c_int, c_uint, size_t};
use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::c_char;
use std::ptr;

#[macro_use]
extern crate bitflags;

#[link(name = "discid")]
extern "C" {
    fn discid_new() -> *const size_t;
    fn discid_free(disc: *const size_t);
    // fn discid_read(disc: *const size_t, device: *const c_char) -> c_int;
    fn discid_read_sparse(disc: *const size_t, device: *const c_char, features: c_uint) -> c_int;
    fn discid_put(disc: *const size_t, first: c_int, last: c_int, offsets: *const c_int) -> c_int;
    fn discid_get_error_msg(disc: *const size_t) -> *const c_char;
    fn discid_get_id(disc: *const size_t) -> *const c_char;
    fn discid_get_freedb_id(disc: *const size_t) -> *const c_char;
    fn discid_get_toc_string(disc: *const size_t) -> *const c_char;
    fn discid_get_submission_url(disc: *const size_t) -> *const c_char;
    fn discid_get_first_track_num(disc: *const size_t) -> c_int;
    fn discid_get_last_track_num(disc: *const size_t) -> c_int;
    fn discid_get_sectors(disc: *const size_t) -> c_int;
    fn discid_get_track_offset(disc: *const size_t, track_num: c_int) -> c_int;
    fn discid_get_track_length(disc: *const size_t, track_num: c_int) -> c_int;
    fn discid_get_track_isrc(disc: *const size_t, track_num: c_int) -> *const c_char;
    fn discid_get_mcn(disc: *const size_t) -> *const c_char;
    fn discid_has_feature(feature: c_uint) -> c_int;
    fn discid_get_version_string() -> *const c_char;
    fn discid_get_default_device() -> *const c_char;
}

bitflags! {
    /// Constants representing the features supported by libdiscid.
    ///
    /// See `DiscId::read_features()` for details.
    pub struct Features: u32 {
        const READ = 1 << 0;
        const MCN  = 1 << 1;
        const ISRC = 1 << 2;
        const ALL  = Self::READ.bits | Self::MCN.bits | Self::ISRC.bits;
    }
}

/// `DiscId` holds information about a disc (TOC, MCN, ISRCs).
///
/// Use `DiscId::read`, `DiscId::read_features` or `DiscId::put` to initialize
/// an instance of `DiscId`.
pub struct DiscId {
    disc: *const size_t,
}

impl DiscId {
    fn new() -> DiscId {
        let disc = unsafe { discid_new() };
        DiscId { disc }
    }

    /// Read the disc in the given CD-ROM/DVD-ROM drive extracting only the TOC.
    ///
    /// This function reads the disc in the drive specified by the given device
    /// identifier. If the device is `None`, the default device, as returned by
    /// `DiscId::default_device()`, is used.
    ///
    /// This function will only read the TOC, hence only the disc ID itself will be
    /// available. Use `DiscId::read_features()` if you want to read also MCN and ISRCs.
    ///
    /// # Examples
    ///
    /// Read from default device
    /// ```ignore
    /// use discid::DiscId;
    ///
    /// let disc = DiscId::read(None).expect("Reading disc failed");
    /// println!("ID: {}", disc.id());
    /// ```
    ///
    /// Read from specific device
    /// ```ignore
    /// # use discid::DiscId;
    /// let disc = DiscId::read(Some("/dev/sr1")).expect("Reading disc failed");
    /// println!("ID: {}", disc.id());
    /// ```
    pub fn read(device: Option<&str>) -> Result<DiscId, String>
    {
        DiscId::read_features(device, Features::READ)
    }

    /// Read the disc in the given CD-ROM/DVD-ROM drive with additional features.
    ///
    /// This function is similar to `DiscId::read()` but allows to read information about MCN
    /// and per-track ISRCs in addition to the normal TOC data.
    ///
    /// The parameter `features` accepts a bitwise combination of values defined in `Features`.
    /// `Features::READ` is always implied, so it is not necessary to specify it.
    ///
    /// Reading MCN and ISRCs is not available on all platforms. You can use
    /// `DiscId::has_feature()` to check if a specific feature is available. Passing unsupported
    /// features here will just be ignored.
    ///
    /// Note that reading MCN and ISRC data is significantly slower than just reading the TOC, so
    /// only request the features you actually need.
    ///
    /// # Examples
    ///
    /// Read both ISRC and MCN from default device
    /// ```ignore
    /// use discid::{DiscId, Features};
    ///
    /// let features = Features::MCN | Features::ISRC;
    /// let disc = DiscId::read_features(None, features).expect("Reading disc failed");
    /// println!("ID: {}", disc.id());
    /// ```
    pub fn read_features(device: Option<&str>, features: Features) -> Result<DiscId, String>
    {
        let disc = DiscId::new();
        let c_device: *const c_char = match device {
            Some(d) => CString::new(d)
                .expect("CString::new failed")
                .into_raw(),
            None => ptr::null(),
        };
        let status = unsafe { discid_read_sparse(disc.disc, c_device, features.bits()) };
        if status == 0 {
            Err(disc.error_msg())
        } else {
            Ok(disc)
        }
    }

    /// Provides the TOC of a known CD.
    ///
    /// This function may be used if the TOC has been read earlier and you want to calculate
    /// the disc ID afterwards, without accessing the disc drive.
    ///
    /// # Examples:
    ///
    /// ```
    /// use discid::DiscId;
    ///
    /// let first_track = 1;
    /// // The offsets contain the total number of sectors as first element, followed by
    /// // the start sectors of each track.
    /// let offsets = [
    ///    242457, 150, 44942, 61305, 72755, 96360, 130485, 147315, 164275, 190702, 205412, 220437,
    /// ];
    /// let disc = DiscId::put(first_track, &offsets).expect("DiscId::put() failed");
    /// assert_eq!("lSOVc5h6IXSuzcamJS1Gp4_tRuA-", disc.id());
    /// ```
    pub fn put(first: i32, offsets: &[i32]) -> Result<DiscId, String> {
        let disc = DiscId::new();
        let last = (offsets.len() - 1) as c_int;
        let status = unsafe { discid_put(disc.disc, first, last, offsets.as_ptr()) };
        if status == 0 {
            Err(disc.error_msg())
        } else {
            Ok(disc)
        }
    }


    /// Check if a certain feature is implemented on the current platform.
    ///
    /// See the [libdiscid feature matrix](https://musicbrainz.org/doc/libdiscid#Feature_Matrix)
    /// for a list of supported features per platform.
    ///
    /// # Examples
    /// ```
    /// use discid::{DiscId, Features};
    ///
    /// let can_read = DiscId::has_feature(Features::READ);
    /// assert!(can_read);
    /// ```
    pub fn has_feature(feature: Features) -> bool {
        let result = unsafe { discid_has_feature(feature.bits()) };
        result == 1
    }

    /// Return version information about libdiscid.
    ///
    /// The returned string will be e.g. `"libdiscid 0.6.2"`.
    ///
    /// # Examples
    /// ```
    /// use discid::DiscId;
    ///
    /// println!("{}", DiscId::version_string());
    /// ```
    pub fn version_string() -> String {
        let str_ptr = unsafe { discid_get_version_string() };
        to_str(str_ptr)
    }

    /// Return the name of the default disc drive for this operating system.
    ///
    /// The default device is system dependent, e.g. `/dev/cdrom` on Linux and `D:` on Windows.
    ///
    /// # Examples
    /// ```
    /// use discid::DiscId;
    ///
    /// println!("{}", DiscId::default_device());
    /// ```
    pub fn default_device() -> String {
        let version_ptr = unsafe { discid_get_default_device() };
        to_str(version_ptr)
    }

    fn error_msg(&self) -> String {
        let str_ptr = unsafe { discid_get_error_msg(self.disc) };
        to_str(str_ptr)
    }

    /// The MusicBrainz disc ID.
    pub fn id(&self) -> String {
        let str_ptr = unsafe { discid_get_id(self.disc) };
        to_str(str_ptr)
    }

    /// The FreeDB disc ID.
    pub fn freedb_id(&self) -> String {
        let str_ptr = unsafe { discid_get_freedb_id(self.disc) };
        to_str(str_ptr)
    }

    /// Return a string representing CD Table Of Contents (TOC).
    pub fn toc_string(&self) -> String {
        let str_ptr = unsafe { discid_get_toc_string(self.disc) };
        to_str(str_ptr)
    }

    /// An URL for submitting the DiscID to MusicBrainz.
    pub fn submission_url(&self) -> String {
        let str_ptr = unsafe { discid_get_submission_url(self.disc) };
        to_str(str_ptr)
    }

    /// The number of the first track on this disc.
    pub fn first_track_num(&self) -> i32 {
        unsafe { discid_get_first_track_num(self.disc) }
    }

    /// The number of the last track on this disc.
    pub fn last_track_num(&self) -> i32 {
        unsafe { discid_get_last_track_num(self.disc) }
    }

    /// The length of the disc in sectors.
    pub fn sectors(&self) -> i32 {
        unsafe { discid_get_sectors(self.disc) }
    }

    /// Returns the offset in sectors for a certain track number (starting at 1).
    pub fn track_offset(&self, track_num: i32) -> i32 {
        unsafe { discid_get_track_offset(self.disc, track_num) }
    }

    /// Returns the length (in sectors) for a certain track number (starting at 1).
    pub fn track_length(&self, track_num: i32) -> i32 {
        unsafe { discid_get_track_length(self.disc, track_num) }
    }

    /// The media catalogue number on the disc, if present.
    pub fn mcn(&self) -> String {
        let str_ptr = unsafe { discid_get_mcn(self.disc) };
        to_str(str_ptr)
    }

    /// Returns the ISRC for a certain track number (starting at 1).
    pub fn track_isrc(&self, track_num: i32) -> String {
        let str_ptr = unsafe { discid_get_track_isrc(self.disc, track_num) };
        to_str(str_ptr)
    }
}

impl Drop for DiscId {
    fn drop(&mut self) {
        unsafe { discid_free(self.disc) }
    }
}

impl fmt::Debug for DiscId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DiscId {}", self.toc_string())
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
    #[should_panic(expected = "cannot open")]
    fn test_read_invalid_device() {
        DiscId::read(Some("notadevice")).expect("DiscId::read failed");
    }

    #[test]
    #[should_panic(expected = "cannot open")]
    fn test_read_features_invalid_device() {
        DiscId::read_features(Some("notadevice"), Features::READ).expect("DiscId::read failed");
    }

    #[test]
    fn test_put() {
        let first = 1;
        let offsets = [
            206535, 150, 18901, 39738, 59557, 79152, 100126, 124833, 147278, 166336, 182560,
        ];
        let disc = DiscId::put(first, &offsets).expect("DiscId::put failed");
        let last_track = disc.last_track_num();
        assert_eq!("Wn8eRBtfLDfM0qjYPdxrz.Zjs_U-", disc.id());
        assert_eq!("830abf0a", disc.freedb_id());
        assert_eq!(1, disc.first_track_num());
        assert_eq!(10, last_track);
        assert_eq!(206535, disc.sectors());
        assert_eq!(
            "1 10 206535 150 18901 39738 59557 79152 100126 124833 147278 166336 182560",
            disc.toc_string()
        );
        assert_eq!(
            "http://musicbrainz.org/cdtoc/attach?id=Wn8eRBtfLDfM0qjYPdxrz.Zjs_U-&tracks=10&toc=1+10+206535+150+18901+39738+59557+79152+100126+124833+147278+166336+182560",
            disc.submission_url());
        for i in first..last_track + 1 {
            let offset = offsets[i as usize];
            let next = if i < last_track { i + 1 } else { 0 };
            let length = offsets[next as usize] - offset;
            assert_eq!(
                offset,
                disc.track_offset(i),
                "track {} expected offset {}",
                i,
                offset
            );
            assert_eq!(
                length,
                disc.track_length(i),
                "track {} expected length {}",
                i,
                length
            );
        }
    }

    #[test]
    fn test_default_device() {
        let device = DiscId::default_device();
        assert!(!device.is_empty());
    }

    #[test]
    fn test_has_feature() {
        assert_eq!(true, DiscId::has_feature(Features::READ));
    }

    #[test]
    fn test_version_string() {
        let version = DiscId::version_string();
        assert!(version.starts_with("libdiscid"));
    }

    #[test]
    fn test_debug() {
        let first = 1;
        let offsets = [2000, 150, 1000];
        let disc = DiscId::put(first, &offsets).expect("DiscId::put failed");
        assert_eq!("DiscId 1 2 2000 150 1000", format!("{:?}", disc));
    }
}
