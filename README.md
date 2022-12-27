# Texture Splicer

## Description
Maps the palette of one texture onto another
Works best with pixel art
Call through commandline with
```
palette_swap <TEXTURE IMAGE PATH> <PALETTE IMAGE PATH> <INTERPOLATE (true/false)>
```
Interpolation mode extends the palette of the palette image by interpolating between colors of the base palette
If interpolation mode is off, only exact colors from the base palette will be used

## Live Demo
See live demo at [naitronbomb.com](https://www.naitronbomb.com/mctexture/)

## Compiling
Requires [rust cargo](https://www.rust-lang.org/tools/install) to be installed on your system. Simply run the following command in the root directory of the source to build.
```
cargo build --release
```