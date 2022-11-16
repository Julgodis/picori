<p align="center">
  <a href="https://github.com/Julgodis/picori/">
    <picture>
      <img src="assets/images/picori_logo_512.png" height="128">
    </picture>
    <h1 align="center">
      Picori
    </h1>
  </a>
</p>

<div align="center">

[![crates.io](https://img.shields.io/crates/v/picori)](https://crates.io/crates/picori)
[![docs.rs](https://docs.rs/picori/badge.svg)](https://docs.rs/picori/)
[![build](https://github.com/Julgodis/picori/actions/workflows/build_and_test.yml/badge.svg?branch=master)](https://github.com/Julgodis/picori/actions/workflows/build_and_test.yml)
[![coverage](/../coverage/coverage/badges/flat.svg)](https://julgodis.github.io/picori/coverage/)
[![license](https://img.shields.io/crates/l/picori)](https://github.com/Julgodis/picori/LICENSE)

Picori (ピッコル) is a library for decompilation, modding, and rom-hacking with focus on GameCube and Wii games. It support parsing and building common file formats, e.g., Dolphin executables (DOLs). 

[Features](#features) •
[Usage](#usage) •
[Examples](#examples) •
[Installation](#installation)

```diff
!!! The project is currently very early stages of development. !!!
!!! All features are not unimplemented and compatibility is not guaranteed. !!!
```

</div>

## Features

-   DOL (Dolphin executable)
-   REL (Relocatable module)
-   GCM (GameCube master disc)
-   CISO (Compact ISO)
-   RARC (Wii archive format)
-   Yaz0 compression
-   JIS X 0201 encoding
-   Shift JIS encoding

## Usage

Here is a simple example of how to use Picori to parse a DOL file and print the entry point.

```rust
use std::fs::File;
use picori::Result;
fn main() -> Result<()> {
    let mut file = File::open("main.dol")?;
    let dol = picori::Dol::from_binary(&mut file)?;
    println!("entry point: {:#08x}", dol.entry_point());
    Ok(())
}
```

## Examples

The `examples` directory contains a few examples of how to use
Picori.

* [`dol_dump`](examples/dol_dump.rs) - Dump information about a `.dol` file.
* [`rarc_dump`](examples/rarc_dump.rs) - Dump the content of a `.rarc` archive.
* [`rel_dump`](examples/rel_dump.rs) - Dump information about a `.rel` file.
* [`gcm_dump`](examples/gcm_dump.rs) - Dump information about a `.gcm`/`.iso` file.

## Installation

Picori is available on [crates.io](https://crates.io/crates/picori). Add the following to your `Cargo.toml`:

```toml
[dependencies]
picori = "0.1.0"
```

## Contributing

Contributions are welcome! If you would like to contribute, please open a pull
request on GitHub. Please make sure that your code is formatted with `rustfmt`,
and that it compiles without warnings or errors.

## License

Picori is licensed under an MIT license. See [LICENSE](LICENSE) for more information.
