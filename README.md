# Rust bindings for MusicBrainz libdiscid
[![crates.io](https://img.shields.io/crates/v/discid.svg)](https://crates.io/crates/discid)
[![Documentation](https://docs.rs/discid/badge.svg)](https://docs.rs/discid)
[![License](https://img.shields.io/crates/l/discid.svg)](https://crates.io/crates/discid)

## About
discid provides Rust bindings for the MusicBrainz DiscID library [libdiscid](http://musicbrainz.org/doc/libdiscid).
It allows calculating DiscIDs (MusicBrainz and freedb) for Audio CDs. Additionally
the library can extract the MCN/UPC/EAN and the ISRCs from disc.

## Requirements
* libdiscid >= 0.6.0

## Usage

### Read only the TOC

```rust,no_run
use discid::DiscId;

fn main() {
  // Specifying the device is optional. If set to `None` a platform
  // specific default will be used.
  let device = Some("/dev/cdrom");
  let disc = DiscId::read(device).expect("Reading disc failed");
  println!("ID: {}", disc.id());
}
```

### Read the TOC and ISRCs

```rust,no_run
use discid::{DiscId, Features};

fn main() {
  let disc = DiscId::read_features(None, Features::ISRC).expect("Reading disc failed");
  println!("Disc ID: {}", disc.id());

  for track in disc.tracks() {
    println!("Track #{} ISRC: {}", track.number, track.isrc);
  }
}
```

See the [API documentation](https://docs.rs/discid) for details.
Additional examples are available in the
[examples/](https://github.com/phw/rust-discid/tree/master/examples) directory.

## Contribute
The source code for discid is available on
[GitHub](https://github.com/phw/rust-discid).

Please report any issues on the
[issue tracker](https://github.com/phw/rust-discid/issues).

## License
discid Copyright (c) 2019-2020 by Philipp Wolfer <ph.wolfer@gmail.com>

discid is free software: you can redistribute it and/or modify
it under the terms of the GNU Lesser General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Lesser General Public License for more details.

See LICENSE for details.
