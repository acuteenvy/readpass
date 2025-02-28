# readpass

[![CI](https://img.shields.io/github/actions/workflow/status/acuteenvy/readpass/ci.yml?label=CI&logo=github&style=flat-square)](https://github.com/acuteenvy/readpass/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/readpass?logo=rust&style=flat-square&color=orange)](https://crates.io/crates/readpass)
[![MSRV](https://img.shields.io/crates/msrv/readpass?logo=rust&style=flat-square&color=teal)](https://crates.io/crates/readpass)
[![crates.io downloads](https://img.shields.io/crates/d/readpass?logo=rust&style=flat-square)](https://crates.io/crates/readpass)
[![license](https://img.shields.io/github/license/acuteenvy/readpass?style=flat-square&color=purple)](/LICENSE-APACHE)

A tiny library for reading passwords without displaying them on the terminal.
It's similar to the C function `getpass()` or Python's `getpass` module.

## Usage

Read a password without displaying it on the terminal:

```rust
let passwd = readpass::from_tty()?;
```

If you want to display a prompt, print it to stdout or stderr before reading:

```rust
use std::io::{self, Write};

write!(io::stderr(), "Please enter a password: ")?;
let passwd = readpass::from_tty()?;
```

Strings returned by `readpass` are wrapped in [`Zeroizing`](https://docs.rs/zeroize/latest/zeroize/struct.Zeroizing.html)
to ensure the password is zeroized from memory after it's `Drop`ped.

Docs: <https://docs.rs/readpass>.

## License

The source code is released under the Apache 2.0 license.
<br><br>
This is a fork of [rpassword](https://github.com/conradkleinespel/rpassword) by Conrad Kleinespel.
The original library appears unmaintained, and rolls its own version of [zeroize](https://github.com/RustCrypto/utils/tree/master/zeroize)
in [rtoolbox](https://docs.rs/rtoolbox/0.0.2/rtoolbox/safe_string/struct.SafeString.html).
This crate aims to change that.
