// Copyright (C) 2019-2020 Philipp Wolfer <ph.wolfer@gmail.com>
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
//!
//! To get started see the documentation and examples of [`DiscId::read`], [`DiscId::read_features`],
//! [`DiscId::put`] and [`DiscId::parse`].
//!
//! Details about the use and calculation of disc IDs can be found at the [MusicBrainz
//! disc ID documentation](https://musicbrainz.org/doc/Disc_ID).
//!
//! The source code of this library is available on [GitHub](https://github.com/phw/rust-discid)
//! under the terms of the GNU Lesser General Public License version 3 or later.
//!
//! [`DiscId::read`]: ./struct.DiscId.html#method.read
//! [`DiscId::read_features`]: ./struct.DiscId.html#method.read_features
//! [`DiscId::put`]: ./struct.DiscId.html#method.put
//! [`DiscId::parse`]: ./struct.DiscId.html#method.parse

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use discid_sys::*;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt;
use std::num::ParseIntError;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::ptr;
use std::rc::Rc;

#[macro_use]
extern crate bitflags;

bitflags! {
    /// Constants representing the features supported by libdiscid.
    ///
    /// See [`DiscId::read_features`] for details.
    ///
    /// [`DiscId::read_features`]: ./struct.DiscId.html#method.read_features
    pub struct Features: u32 {
        /// Read the CD TOC.
        ///
        /// This is supported on all platforms and indicates that only the
        /// table of contents (TOC), from which the disc ID gets calculated,
        /// will be read.
        const READ = discid_feature::DISCID_FEATURE_READ.0;

        /// Read the MCN (aka barcode) information.
        ///
        /// Read the MCN field from the CD. Not all CDs provide this information.
        /// Without this feature [`DiscId::mcn`] will always return an empty string.
        ///
        /// [`DiscId::mcn`]: ./struct.DiscId.html#method.mcn
        const MCN  = discid_feature::DISCID_FEATURE_MCN.0;

        /// Supports reading the ISRCs per track.
        ///
        /// For each track read the ISRC encoded in the subchannel data. Not all CDs provide this
        /// information. Without this feature [`Track::isrc`] will always be an empty string.
        ///
        /// [`Track::isrc`]: ./struct.Track.html#structfield.isrc
        const ISRC = discid_feature::DISCID_FEATURE_ISRC.0;
    }
}

impl Features {
    fn as_bitfield(&self) -> discid_feature {
        discid_feature(self.bits())
    }
}

/// This is returned on errors reading the disc or setting the TOC.
#[derive(Debug)]
pub struct DiscError {
    reason: String,
}

impl DiscError {
    fn new(message: &str) -> Self {
        DiscError {
            reason: message.to_string(),
        }
    }
}

impl Error for DiscError {}

impl fmt::Display for DiscError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DiscError: {}", self.reason)
    }
}

impl From<ParseIntError> for DiscError {
    fn from(err: ParseIntError) -> Self {
        DiscError {
            reason: format!("{}", err),
        }
    }
}

#[derive(Debug)]
struct DiscIdHandle {
    real_handle: ptr::NonNull<discid_sys::DiscId>,
}

impl DiscIdHandle {
    fn new(handle: *mut discid_sys::DiscId) -> DiscIdHandle {
        unsafe {
            DiscIdHandle {
                real_handle: ptr::NonNull::new_unchecked(handle),
            }
        }
    }

    fn as_ptr(&self) -> *mut discid_sys::DiscId {
        self.real_handle.as_ptr()
    }
}

impl Drop for DiscIdHandle {
    fn drop(&mut self) {
        unsafe { discid_free(self.as_ptr()) }
    }
}

/// `DiscId` holds information about a disc (TOC, MCN, ISRCs).
///
/// Use [`DiscId::read`], [`DiscId::read_features`], [`DiscId::put`] or [`DiscId::parse`]
/// to initialize an instance of `DiscId`.
///
/// [`DiscId::read`]: #method.read
/// [`DiscId::read_features`]: #method.read_features
/// [`DiscId::put`]: #method.put
/// [`DiscId::parse`]: #method.parse
pub struct DiscId {
    handle: Rc<DiscIdHandle>,
}

impl DiscId {
    fn new() -> Result<DiscId, DiscError> {
        let handle = unsafe { discid_new() };
        if handle.is_null() {
            Err(DiscError::new(
                "discid_new() failed, could not allocate memory",
            ))
        } else {
            Ok(DiscId {
                handle: Rc::new(DiscIdHandle::new(handle)),
            })
        }
    }

    /// Read the disc in the given CD-ROM/DVD-ROM drive extracting only the TOC.
    ///
    /// This function reads the disc in the drive specified by the given device
    /// identifier. If the device is `None`, the default device, as returned by
    /// [`DiscId::default_device`], is used.
    ///
    /// This function will only read the TOC, hence only the disc ID itself will be
    /// available. Use [`DiscId::read_features`] if you want to read also MCN and ISRCs.
    ///
    /// # Examples
    ///
    /// Read from default device
    /// ```no_run
    /// use discid::DiscId;
    ///
    /// let disc = DiscId::read(None).expect("Reading disc failed");
    /// println!("ID: {}", disc.id());
    /// ```
    ///
    /// Read from specific device
    /// ```no_run
    /// # use discid::DiscId;
    /// let disc = DiscId::read(Some("/dev/sr1")).expect("Reading disc failed");
    /// println!("ID: {}", disc.id());
    /// ```
    ///
    /// [`DiscId::read_features`]: #method.read_features
    /// [`DiscId::default_device`]: #method.default_device
    pub fn read(device: Option<&str>) -> Result<DiscId, DiscError> {
        DiscId::read_features(device, Features::READ)
    }

    /// Read the disc in the given CD-ROM/DVD-ROM drive with additional features.
    ///
    /// This function is similar to [`DiscId::read`] but allows to read information about MCN
    /// and per-track ISRCs in addition to the normal TOC data.
    ///
    /// The parameter `features` accepts a bitwise combination of values defined in [`Features`].
    /// [`Features::READ`] is always implied, so it is not necessary to specify it.
    ///
    /// Reading MCN and ISRCs is not available on all platforms. You can use the
    /// [`has_feature`] method to check if a specific feature is available. Passing unsupported
    /// features here will just be ignored.
    ///
    /// Note that reading MCN and ISRC data is significantly slower than just reading the TOC, so
    /// only request the features you actually need.
    ///
    /// # Examples
    ///
    /// Read both ISRC and MCN from default device
    /// ```no_run
    /// use discid::{DiscId, Features};
    ///
    /// let features = Features::MCN | Features::ISRC;
    /// let disc = DiscId::read_features(None, features).expect("Reading disc failed");
    /// println!("ID : {}", disc.id());
    /// println!("MCN: {}", disc.mcn());
    /// for track in disc.tracks() {
    ///     println!("#{} ISRC: {}", track.number, track.isrc);
    /// }
    /// ```
    ///
    /// [`DiscId::read`]: #method.read
    /// [`has_feature`]: #method.has_feature
    /// [`Features`]: ./struct.Features.html
    /// [`Features::READ`]: ./struct.Features.html#associatedconstant.READ
    pub fn read_features(device: Option<&str>, features: Features) -> Result<DiscId, DiscError> {
        let disc = DiscId::new()?;
        let c_device: *const c_char = match device {
            Some(d) => CString::new(d).expect("CString::new failed").into_raw(),
            None => ptr::null(),
        };
        let status =
            unsafe { discid_read_sparse(disc.handle.as_ptr(), c_device, features.as_bitfield()) };
        if status == 0 {
            Err(disc.error())
        } else {
            Ok(disc)
        }
    }

    /// Provides the TOC of a known CD.
    ///
    /// This function may be used if the TOC has been read earlier and you want to calculate
    /// the disc ID afterwards, without accessing the disc drive.
    ///
    /// `first` is the track number of the first track (1-99).
    /// The `offsets` parameter points to an array which contains the track offsets for each track.
    /// The first element, `offsets[0]`, is the lead-out track. It must contain the total number of
    /// sectors on the disc. `offsets` must not be longer than 100 elements (lead-out + 99 tracks).
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
    pub fn put(first: i32, offsets: &[i32]) -> Result<DiscId, DiscError> {
        let disc = DiscId::new()?;
        let last = first + (offsets.len() as c_int) - 2;
        let offset_ptr: *mut c_int;
        let mut full_offsets: [c_int; 100];

        if first > 1 && last <= 99 {
            // libdiscid always expects an array of 100 integers, no matter the track count.
            // If the first track is larger 1 empty tracks must be filled with 0.
            full_offsets = [0; 100];
            full_offsets[0] = offsets[0]; // Total sectors on disc
            full_offsets[first as usize..(last + 1) as usize].copy_from_slice(&offsets[1..]);
            offset_ptr = full_offsets.as_ptr() as *mut c_int;
        } else {
            // In case the track count starts at 1 we do not have to copy the array.
            // libdiscid will not read beyond the boundary of `last + 1`, which in our case
            // equals `offsets.length`.
            offset_ptr = offsets.as_ptr() as *mut c_int;
        }

        let status = unsafe { discid_put(disc.handle.as_ptr(), first, last, offset_ptr) };
        if status == 0 {
            Err(disc.error())
        } else {
            Ok(disc)
        }
    }

    /// Parses a TOC string and returns a [`DiscId`] instance for it.
    ///
    /// The TOC string provided here must have the same format as returned by [`toc_string`].
    ///
    /// This function can be used if you already have a TOC string like e.g.
    /// `1 11 242457 150 44942 61305 72755 96360 130485 147315 164275 190702 205412 220437`.
    ///
    /// # Examples:
    ///
    /// ```
    /// use discid::DiscId;
    ///
    /// let toc = "1 11 242457 150 44942 61305 72755 96360 130485 147315 164275 190702 205412 220437";
    /// let disc = DiscId::parse(toc).expect("DiscId::put() failed");
    /// assert_eq!("lSOVc5h6IXSuzcamJS1Gp4_tRuA-", disc.id());
    /// assert_eq!(toc, disc.toc_string());
    /// ```
    ///
    /// [`DiscId`]: ./struct.DiscId.html
    /// [`toc_string`]: #method.toc_string
    pub fn parse(toc: &str) -> Result<DiscId, DiscError> {
        let mut i: usize = 0;
        let mut first_track: c_int = 1;
        let mut last_track: c_int = 1;
        let mut offsets: [c_int; 100] = [0; 100];
        for part in toc.split(' ') {
            let parsed_int = part.parse::<c_int>()?;
            if i == 0 {
                first_track = parsed_int;
            } else if i == 1 {
                last_track = parsed_int;
            } else if i > 1 {
                if i > (last_track as usize + 2) || i > 99 + 2 {
                    return Err(DiscError::new(
                        "TOC string contains too many offsets (max. 100)",
                    ));
                }

                offsets[i - 2] = parsed_int;
            }

            i += 1;
        }

        if i < 3 {
            return Err(DiscError::new(&format!("Invalid TOC string {:?}", toc)));
        }

        let offset_count = (i - 3) as c_int;
        let track_count = last_track - first_track + 1;
        if track_count != offset_count {
            return Err(DiscError::new(&format!(
                "Number of offsets {} does not match track count {}",
                offset_count, last_track
            )));
        }

        DiscId::put(first_track, &offsets[0..(i - 2)])
    }

    /// Check if a certain feature is implemented on the current platform.
    ///
    /// This only works for single features, not bit masks with multiple features.
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
        let result = unsafe { discid_has_feature(feature.as_bitfield()) };
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

    fn error(&self) -> DiscError {
        let str_ptr = unsafe { discid_get_error_msg(self.handle.as_ptr()) };
        DiscError::new(&to_str(str_ptr))
    }

    /// The MusicBrainz disc ID.
    pub fn id(&self) -> String {
        let str_ptr = unsafe { discid_get_id(self.handle.as_ptr()) };
        to_str(str_ptr)
    }

    /// The FreeDB disc ID.
    pub fn freedb_id(&self) -> String {
        let str_ptr = unsafe { discid_get_freedb_id(self.handle.as_ptr()) };
        to_str(str_ptr)
    }

    /// Return a string representing CD Table Of Contents (TOC).
    ///
    /// The TOC string is a list of integers separated by a single space character. The integers
    /// represent (in order):
    ///
    /// - First track number (normally one)
    /// - Last track number
    /// - Lead-out track offset
    /// - Up to 99 frame offsets
    ///
    /// See also [`DiscId::parse`] and the documentation for [Disc ID Calculation](https://musicbrainz.org/doc/Disc_ID_Calculation).
    ///
    /// [`DiscId::parse`]: #method.parse
    pub fn toc_string(&self) -> String {
        let str_ptr = unsafe { discid_get_toc_string(self.handle.as_ptr()) };
        to_str(str_ptr)
    }

    /// An URL for submitting the DiscID to MusicBrainz.
    pub fn submission_url(&self) -> String {
        let str_ptr = unsafe { discid_get_submission_url(self.handle.as_ptr()) };
        to_str(str_ptr)
    }

    /// The number of the first track on this disc.
    pub fn first_track_num(&self) -> i32 {
        unsafe { discid_get_first_track_num(self.handle.as_ptr()) }
    }

    /// The number of the last track on this disc.
    pub fn last_track_num(&self) -> i32 {
        unsafe { discid_get_last_track_num(self.handle.as_ptr()) }
    }

    /// The length of the disc in sectors.
    pub fn sectors(&self) -> i32 {
        unsafe { discid_get_sectors(self.handle.as_ptr()) }
    }

    /// The media catalogue number on the disc, if present.
    pub fn mcn(&self) -> String {
        let str_ptr = unsafe { discid_get_mcn(self.handle.as_ptr()) };
        to_str(str_ptr)
    }

    /// Returns an iterator to access information about each track on the disc.
    ///
    /// Returns an instance of [`Track`] for each track.
    ///
    /// # Examples
    ///
    /// ```
    /// use discid::DiscId;
    ///
    /// let offsets = [
    ///    242457, 150, 44942, 61305, 72755, 96360, 130485, 147315, 164275, 190702, 205412, 220437,
    /// ];
    /// let disc = DiscId::put(1, &offsets).expect("DiscId::put() failed");
    /// for track in disc.tracks() {
    ///     println!("Track #{}", track.number);
    ///     println!("    ISRC    : {}", track.isrc);
    ///     println!("    Offset  : {}", track.offset);
    ///     println!("    Sectors : {}", track.sectors);
    /// }
    /// ```
    ///
    /// [`Track`]: ./struct.Track.html
    pub fn tracks(&self) -> TrackIter {
        TrackIter::new(Rc::clone(&self.handle))
    }

    /// Returns a [`Track`] instance for the nth track.
    ///
    /// # Panics
    /// Panics if `number` is outside the range given by [`first_track_num`]
    /// and [`last_track_num`].
    ///
    /// # Examples
    ///
    /// ```
    /// use discid::DiscId;
    ///
    /// let offsets = [
    ///    242457, 150, 44942, 61305, 72755, 96360, 130485, 147315, 164275, 190702, 205412, 220437,
    /// ];
    /// let disc = DiscId::put(1, &offsets).expect("DiscId::put() failed");
    /// let track = disc.nth_track(7);
    /// assert_eq!(7, track.number);
    /// assert_eq!(147315, track.offset);
    /// assert_eq!(16960, track.sectors);
    /// ```
    ///
    /// [`Track`]: ./struct.Track.html
    /// [`first_track_num`]: #method.first_track_num
    /// [`last_track_num`]: #method.last_track_num
    pub fn nth_track(&self, number: i32) -> Track {
        let first = self.first_track_num();
        let last = self.last_track_num();
        if number < first || number > last {
            panic!(
                "track number out of bounds: given {}, expected between {} and {}",
                number, first, last
            );
        }
        get_track(Rc::clone(&self.handle), number)
    }
}

impl fmt::Debug for DiscId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DiscId {}", self.toc_string())
    }
}

/// Holds information about a single track
#[derive(Debug)]
pub struct Track {
    /// Track number (1-99) of the track.
    pub number: i32,

    /// Start offset in sectors.
    pub offset: i32,

    /// Track length in sectors.
    pub sectors: i32,

    /// ISRC for this track (might be empty).
    ///
    /// This will only bet set if [`DiscId::read_features`] is called with [`Features::ISRC`].
    ///
    /// [`DiscId::read_features`]: ./struct.DiscId.html#method.read_features
    /// [`Features::ISRC`]: ./struct.Features.html#associatedconstant.ISRC
    pub isrc: String,
}

/// Allows iterating over all tracks of a read disc.
///
/// Returns an instance of [`Track`] for each track.
///
/// [`Track`]: ./struct.Track.html
#[derive(Debug)]
pub struct TrackIter {
    handle: Rc<DiscIdHandle>,
    curr: i32,
    last_track: i32,
}

impl TrackIter {
    fn new(handle: Rc<DiscIdHandle>) -> TrackIter {
        let handle_ptr = handle.as_ptr();
        let first_track = unsafe { discid_get_first_track_num(handle_ptr) };
        let last_track = unsafe { discid_get_last_track_num(handle_ptr) };
        TrackIter {
            handle,
            curr: first_track,
            last_track,
        }
    }
}

impl Iterator for TrackIter {
    type Item = Track;

    fn next(&mut self) -> Option<Track> {
        let track_num = self.curr;
        self.curr += 1;
        if track_num <= self.last_track {
            Some(get_track(Rc::clone(&self.handle), track_num))
        } else {
            None
        }
    }
}

fn get_track(handle: Rc<DiscIdHandle>, number: i32) -> Track {
    let handle_ptr = handle.as_ptr();
    let isrc_ptr = unsafe { discid_get_track_isrc(handle_ptr, number) };
    Track {
        number,
        offset: unsafe { discid_get_track_offset(handle_ptr, number) },
        sectors: unsafe { discid_get_track_length(handle_ptr, number) },
        isrc: to_str(isrc_ptr),
    }
}

fn to_str(c_buf: *const c_char) -> String {
    let c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
    let str_slice = c_str.to_string_lossy();
    str_slice.into_owned()
}

#[cfg(test)]
mod tests {
    use super::{DiscError, DiscId, Features, Track};

    #[test]
    #[ignore]
    fn discid_read_default_device() {
        // This test requires an actual CD inserted into the default CD drive.
        let disc = DiscId::read(None).expect("DiscId::read failed");
        assert_eq!(28, disc.id().len());
    }

    #[test]
    #[should_panic(expected = "cannot open")]
    fn discid_read_invalid_device() {
        DiscId::read(Some("notadevice")).expect("DiscId::read failed");
    }

    #[test]
    #[ignore]
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    fn discid_read_features_all() {
        // This test requires an actual CD inserted into the default CD drive.
        // The CD must provide MCN and ISRC information.
        let disc =
            DiscId::read_features(None, Features::all()).expect("DiscId::read_features failed");
        assert_eq!(28, disc.id().len());
        assert!(!disc.mcn().is_empty());
        assert_eq!(12, disc.nth_track(1).isrc.len());
    }

    #[test]
    #[should_panic(expected = "cannot open")]
    fn discid_read_features_invalid_device() {
        DiscId::read_features(Some("notadevice"), Features::READ)
            .expect("DiscId::read_features failed");
    }

    #[test]
    fn discid_put() {
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
        for track in disc.tracks() {
            let offset = offsets[track.number as usize];
            let next = if track.number < last_track {
                track.number + 1
            } else {
                0
            };
            let length = offsets[next as usize] - offset;
            assert_eq!(
                offset, track.offset,
                "track {} expected offset {}",
                track.number, offset
            );
            assert_eq!(
                length, track.sectors,
                "track {} expected sectors {}",
                track.number, length
            );
        }
    }

    #[test]
    fn discid_put_first_track_not_one() {
        let first = 3;
        let offsets = [
            206535, 150, 18901, 39738, 59557, 79152, 100126, 124833, 147278, 166336, 182560,
        ];
        let disc = DiscId::put(first, &offsets).expect("DiscId::put failed");
        assert_eq!("ByBKvJM1hBL7XtvsPyYtIjlX0Bw-", disc.id());
        assert_eq!(3, disc.first_track_num());
        assert_eq!(12, disc.last_track_num());
        assert_eq!(206535, disc.sectors());
    }

    #[test]
    #[should_panic(expected = "Illegal track limits")]
    fn discid_put_too_many_offsets() {
        let first = 1;
        let offsets: [i32; 101] = [0; 101];
        DiscId::put(first, &offsets).expect("DiscId::put failed");
    }

    #[test]
    #[should_panic(expected = "Illegal track limits")]
    fn discid_put_too_many_tracks() {
        let first = 11;
        let offsets: [i32; 101] = [0; 101];
        DiscId::put(first, &offsets).expect("DiscId::put failed");
    }

    #[test]
    fn discid_parse() {
        let toc =
            "1 11 242457 150 44942 61305 72755 96360 130485 147315 164275 190702 205412 220437";
        let disc = DiscId::parse(toc).expect("DiscId::parse failed");
        assert_eq!("lSOVc5h6IXSuzcamJS1Gp4_tRuA-", disc.id());
        assert_eq!(toc, disc.toc_string());
    }

    #[test]
    fn discid_parse_minimal() {
        let toc = "1 1 44942 150";
        let disc = DiscId::parse(toc).expect("DiscId::parse failed");
        assert_eq!("ANJa4DGYN_ktpzOwvVPtcjwP7mE-", disc.id());
        assert_eq!(toc, disc.toc_string());
    }

    #[test]
    fn discid_parse_first_track_not_one() {
        let toc = "3 12 242457 150 18901 39738 59557 79152 100126 124833 147278 166336 182560";
        let disc = DiscId::parse(toc).expect("DiscId::parse failed");
        assert_eq!("fC1yNbC5bVjbvphqlAY9JyYoWEY-", disc.id());
        assert_eq!(toc, disc.toc_string());
    }

    #[test]
    #[should_panic(expected = "invalid digit found in string")]
    fn discid_parse_invalid_nan() {
        let toc = "1 2 242457 150 a";
        DiscId::parse(toc).expect("DiscId::parse failed");
    }

    #[test]
    #[should_panic(expected = "TOC string contains too many offsets")]
    fn discid_parseinvalid_too_many_offsets() {
        let toc = "1 2 242457 150 200 300";
        DiscId::parse(&toc).expect("DiscId::parse failed");
    }

    #[test]
    #[should_panic(expected = "TOC string contains too many offsets")]
    fn discid_parseinvalid_too_many_offsets_total() {
        let mut indexes = vec!["0"; 103];
        indexes[0] = "1";
        indexes[1] = "100";
        let toc = indexes.join(" ");
        DiscId::parse(&toc).expect("DiscId::parse failed");
    }

    #[test]
    #[should_panic(expected = "Number of offsets 1 does not match track count 2")]
    fn discid_parse_invalid_missing_offsets() {
        let toc = "1 2 242457 150";
        DiscId::parse(toc).expect("DiscId::parse failed");
    }

    #[test]
    #[should_panic(expected = "Invalid TOC string")]
    fn discid_parse_invalid_not_enough_elements() {
        let toc = "1";
        DiscId::parse(toc).expect("DiscId::parse failed");
    }

    #[test]
    #[should_panic(expected = "cannot parse integer from empty string")]
    fn discid_parse_invalid_empty() {
        let toc = "";
        DiscId::parse(toc).expect("DiscId::parse failed");
    }

    #[test]
    fn discid_nth_track() {
        let first = 1;
        let offsets = [
            206535, 150, 18901, 39738, 59557, 79152, 100126, 124833, 147278, 166336, 182560,
        ];
        let disc = DiscId::put(first, &offsets).expect("DiscId::put failed");
        let track = disc.nth_track(4);
        let expected_offset = offsets[4];
        let expected_sectors = offsets[5] - offsets[4];
        assert_eq!(4, track.number);
        assert_eq!("", track.isrc); // Always empty for DiscId::put
        assert_eq!(
            expected_offset, track.offset,
            "track {} expected offset {}",
            track.number, expected_offset
        );
        assert_eq!(
            expected_sectors, track.sectors,
            "track {} expected sectors {}",
            track.number, expected_sectors
        );
    }

    #[test]
    #[should_panic(expected = "track number out of bounds: given 11, expected between 1 and 10")]
    fn discid_nth_track_out_of_bounds() {
        let first = 1;
        let offsets = [
            206535, 150, 18901, 39738, 59557, 79152, 100126, 124833, 147278, 166336, 182560,
        ];
        let disc = DiscId::put(first, &offsets).expect("DiscId::put failed");
        disc.nth_track(11);
    }

    #[test]
    fn discid_default_device() {
        let device = DiscId::default_device();
        assert!(!device.is_empty());
    }

    #[test]
    fn discid_has_feature() {
        assert_eq!(true, DiscId::has_feature(Features::READ));
    }

    #[test]
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    fn discid_has_feature_all() {
        assert_eq!(true, DiscId::has_feature(Features::READ));
        assert_eq!(true, DiscId::has_feature(Features::MCN));
        assert_eq!(true, DiscId::has_feature(Features::ISRC));
    }

    #[test]
    fn discid_version_string() {
        let version = DiscId::version_string();
        assert!(version.starts_with("libdiscid"));
    }

    #[test]
    fn discid_debug() {
        let first = 1;
        let offsets = [2000, 150, 1000];
        let disc = DiscId::put(first, &offsets).expect("DiscId::put failed");
        assert_eq!("DiscId 1 2 2000 150 1000", format!("{:?}", disc));
    }

    #[test]
    fn features() {
        assert_eq!(3, (Features::READ | Features::MCN).bits());
        assert_eq!(
            Features::all(),
            Features::READ | Features::MCN | Features::ISRC
        );
    }

    #[test]
    fn track_debug() {
        let track = Track {
            number: 3,
            offset: 57402,
            sectors: 32960,
            isrc: "DED831801578".to_string(),
        };

        assert_eq!(
            "Track { number: 3, offset: 57402, sectors: 32960, isrc: \"DED831801578\" }",
            format!("{:?}", track)
        );
    }

    #[test]
    fn disc_error_new() {
        let message = "The message";
        let error = DiscError::new(message);

        assert_eq!(message, error.reason);
    }

    #[test]
    fn disc_error_fmt() {
        let error = DiscError::new("The message");
        assert_eq!("DiscError: The message", format!("{}", error));
    }

    #[test]
    fn disc_error_debug() {
        let error = DiscError::new("The message");
        assert_eq!(
            "DiscError { reason: \"The message\" }",
            format!("{:?}", error)
        );
    }
}
