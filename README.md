# readpass

`readpass` makes it easy to read passwords from Rust code in console applications on Unix-like OSes and Windows.
It's similar to Linux's C function `getpass()` or Python's `getpass` module.

## Usage

Read a password without displaying it on the terminal:

```rust
let passwd = readpass::from_tty()?;
```

If you want to display a prompt, print it to stdout or stderr before reading:

```rust
use std::io::{self, Write};

writeln!(io::stderr(), "Please enter a password: ")?;
let passwd = readpass::from_tty()?;
```

Docs: [https://docs.rs/readpass](https://docs.rs/readpass).

## License

The source code is released under the Apache 2.0 license.
This is a fork of [rpassword](https://github.com/conradkleinespel/rpassword) by Conrad Kleinespiel.
The original code rolls its own version of [zeroize](https://github.com/RustCrypto/utils/tree/master/zeroize),
in the [rtoolbox](https://docs.rs/rtoolbox/0.0.2/rtoolbox/safe_string/struct.SafeString.html) crate.
This crate aims to change that.
