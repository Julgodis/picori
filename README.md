<p align="center">
  <a href="https://">
    <picture>
      <img src="assets/images/picori_logo_512.png" height="128">
    </picture>
    <h1 align="center">picori</h1>
  </a>
</p>

[![crates.io](https://img.shields.io/crates/v/picori)](https://crates.io/crates/picori)
[![docs.rs](https://docs.rs/picori/badge.svg)](https://docs.rs/picori/)
[![build](https://github.com/Julgodis/picori/actions/workflows/build_and_test.yml/badge.svg?branch=master)](https://github.com/Julgodis/picori/actions/workflows/build_and_test.yml)
[![coverage](/../coverage/coverage/badges/flat.svg)](https://julgodis.github.io/picori/coverage/)
[![license](https://img.shields.io/crates/l/picori)](https://github.com/Julgodis/picori/LICENSE)

Picori is a library for building modding tools and decompilation tools for GameCube and Wii games. It includes support to serialize and deserialize many Nintendo-specific development and game formats, common compression algorithms, string encodings, and the ability to demangle C++ symbols.

## Formats

Picori supports the following formats:

-   DOL - Dolphin Executable
-   REL - Relocatable Executable
-   GCM - GameCube Master Disc
-   RARC - Nintendo RARC
-   CISO - Compact ISO (WIB)
-   ELF - Executable and Linkable Format[^note-elf]

In the future adding support for more formats is planned.

[^note-elf]:
    ELF is not a specific format used by either GameCube or Wii, but no known compiler outputs DOL files direct (and for good reasons), instead they produce ELF files. Support for ELF (specific to GameCube and Wii) will be very useful.

## Compression

Picori supports the following compression algorithms:

-   Yaz0
-   Yay0

## C++ Demangler

Picori also includes a C++ demangler for MWCC (Metrowerks CodeWarrior Compiler) that was probably included and shipped with the SDK and used for GameCube development.

## Examples

TODO: Add examples

License: MIT
