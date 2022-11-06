<p align="center">
  <a href="https://">
    <picture>
      <img src="assets/picori_logo_512.png" height="128">
    </picture>
    <h1 align="center">picori</h1>
  </a>
</p>

[![crates.io](https://img.shields.io/crates/v/test1)](https://crates.io/crates/test1)
[![docs.rs](https://docs.rs/test1/badge.svg)](https://docs.rs/druid/)
[![license](https://img.shields.io/crates/l/druid)](https://github.com/linebender/druid/blob/master/LICENSE)
[![coverage](/../coverage/coverage/badges/flat.svg)](https://julgodis.github.io/picori/coverage/)

Picori is a library for building modding tools and decompliation tools for
GameCube and Wii games. It includes support to serialize and deserialize many
Nintendo specific development and game formats, common compression algorithms, string
encodings and the ability to demangle C++ symbols.

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
    ELF is not a specific format used by either GameCube or Wii,
    but no known compiler outputs DOL files direct (and for good reasons),
    instead they produce ELF files. Support for ELF (specific to GameCube and
    Wii) are useful.

## Compression

Picori supports the following compression algorithms:

-   Yaz0
-   Yay0

## C++ Demangler

Picori also includes a C++ demangler for MWCC (Metrowerks CodeWarrior
Compiler) that was probably include and shipped with the SDK and used for
GameCube development.

## Examples

TODO: Add examples

License: MIT
