# Rust bindings for MusicBrainz libdiscid
[![Build Status](https://travis-ci.org/phw/discid-rs.svg?branch=master)](https://travis-ci.org/phw/discid-rs)

## About
discid-rs provides Rust bindings for the MusicBrainz DiscID library [libdiscid](http://musicbrainz.org/doc/libdiscid).
It allows calculating DiscIDs (MusicBrainz and freedb) for Audio CDs. Additionally
the library can extract the MCN/UPC/EAN and the ISRCs from disc.

This library is currently in early development and the API may still change.

## Requirements
* libdiscid >= 0.6.0

## Usage

### Read only the TOC

```rust
use discid::DiscId;

// Specifying the device is optional. If set to `None` a platform
// specific default will be used.
let device = Some("/dev/cdrom");
let disc = DiscId::read(device).expect("Reading disc failed");
println!("ID : {}", disc.get_id());
```

## Contribute
The source code for discid-rs is available on
[GitHub](https://github.com/phw/discid-rs).

Please report any issues on the
[issue tracker](https://github.com/phw/discid-rs/issues).

## License
discid-rs Copyright (c) 2019 by Philipp Wolfer <ph.wolfer@gmail.com>

discid-rs is free software: you can redistribute it and/or modify
it under the terms of the GNU Lesser General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Lesser General Public License for more details.

See LICENSE for details.
