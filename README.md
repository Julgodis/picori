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

Picori is a library to support modding and decompilation tools for GameCube and Wii games. It includes support to serialize and deserialize many development and game formats, common compression algorithms, string encodings, and the ability to demangle C++ symbols.

[Features](#features) •
[Examples](#examples) •
[Installation](#installation)

```diff
!!! It is currently in a very early stage of development. !!!
```

</div>


## Features

-   DOL (Dolphin Executable)
-   REL (Relocatable Executable)
-   GCM (GameCube Master Disc)
-   CISO (Compact ISO)
-   Yaz0
-   JIS X 0201
-   Shift JIS (1997 and 2004)

## Examples

**TODO: Add examples**

## Installation

Picori is available on [crates.io](https://crates.io/crates/picori). Add the following to your `Cargo.toml`:

```toml
[dependencies]
picori = "0.1"
```

## Contributing

Contributions are welcome! If you would like to contribute, please open a pull
request on GitHub. Please make sure that your code is formatted with `rustfmt`,
and that it compiles without warnings or errors.

## License

Picori is licensed under an MIT license. See [LICENSE](LICENSE) for more information.
