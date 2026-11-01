<div align="center">

# NyashkChannel

SSL-style channel strip VST3 plugin: input, filters, expander / gate, compressor, 4-band EQ and output.

[![Build](https://github.com/xslimakx/NyashkChannel/actions/workflows/build.yml/badge.svg)](https://github.com/xslimakx/NyashkChannel/actions/workflows/build.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
![Format](https://img.shields.io/badge/format-VST3-8a2be2)

[**Download**](https://github.com/xslimakx/NyashkChannel/releases/latest)

</div>

<p align="center"><img src="docs/img/nyashkchannel.jpg" width="720"></p>

## Install
Use the Nyashk Launcher, or unzip the `.vst3` bundle from the latest release into your VST3 folder.

## Build
```
cargo xtask bundle -p nyashkchannel --release
```
The bundle appears in `target/bundled/`.

## License
MIT
