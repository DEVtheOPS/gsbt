# Changelog

## [0.3.0](https://github.com/DEVtheOPS/gsbt/compare/v0.2.1...v0.3.0) (2026-08-14)


### Features

* add backup stats reporting ([a8cb6a3](https://github.com/DEVtheOPS/gsbt/commit/a8cb6a3f56cde74905fe0b7b006fa86ab12fbcd0))
* add command stubs for backup, prune, list, restore ([f429d20](https://github.com/DEVtheOPS/gsbt/commit/f429d20cdde27eaf43bffbad5f26be8d38a9b2b4))
* add config loading and discovery ([b9cad33](https://github.com/DEVtheOPS/gsbt/commit/b9cad33f6e615d0c80bff0a1a1cf946db9139ab0))
* add config schema generation and init command ([33f6cb0](https://github.com/DEVtheOPS/gsbt/commit/33f6cb0d9cfaf11d22e81db87351b31ddc670ec4))
* add core dependencies ([2e39e08](https://github.com/DEVtheOPS/gsbt/commit/2e39e0842304006fd4a53b388277cd5f15fad2ca))
* add environment variable substitution ([e439ee1](https://github.com/DEVtheOPS/gsbt/commit/e439ee17488d2ef1a9ae4e4232935bc0feba8675))
* add file pattern matching for include/exclude ([bb7327e](https://github.com/DEVtheOPS/gsbt/commit/bb7327eb844ae334174ee0a9699e6367498c7023))
* add nitrado connector and connector factory ([5737ee5](https://github.com/DEVtheOPS/gsbt/commit/5737ee5a89b1431a03f9f26f28d71f457998a408))
* add progress reporting and rich mode for backup ([ec66d5b](https://github.com/DEVtheOPS/gsbt/commit/ec66d5b95b3ea210f9f968bc4129eb0f8924bdf6))
* add standardized logging package with rich output ([dfe2516](https://github.com/DEVtheOPS/gsbt/commit/dfe25162fee679b12fae08961c96e9ca6fdb5ee9))
* add standardized progress reporting package ([25c5324](https://github.com/DEVtheOPS/gsbt/commit/25c53244db86640d4518648f3a64306f9ea57604))
* add version command ([7a3dce4](https://github.com/DEVtheOPS/gsbt/commit/7a3dce4b4b81594eda0036f2034c7752e2d85034))
* define config structs with yaml parsing ([d5b12da](https://github.com/DEVtheOPS/gsbt/commit/d5b12da11afa57535f386b46380c3805a12a6cde))
* define connector interface ([1bfd33c](https://github.com/DEVtheOPS/gsbt/commit/1bfd33c406cc5bec54afa4d7a28b986d8ac9bdb2))
* implement FTP connector ([628aaf0](https://github.com/DEVtheOPS/gsbt/commit/628aaf0adb58ada51ae9e26664945845dddb5e0f))
* implement SFTP connector ([ff40381](https://github.com/DEVtheOPS/gsbt/commit/ff40381845031464a70ebbda5953121657e2c09e))
* initialize go module and main entry point ([e8b6df8](https://github.com/DEVtheOPS/gsbt/commit/e8b6df8c046a560e33c1d0e2774e5b11bc092eb5))
* integrate hedzr progressbar rich output ([ce57c08](https://github.com/DEVtheOPS/gsbt/commit/ce57c08a38f3b1a5aca69f46c593118fda557f2e))
* setup cobra root command with global flags ([cec57b8](https://github.com/DEVtheOPS/gsbt/commit/cec57b846be46e110598ffad433964a81c277fec))
* wire backup command with ftp flow and archiving ([3a25dc7](https://github.com/DEVtheOPS/gsbt/commit/3a25dc7ba52853fe2c54eec04c0715bc77748fa3))


### Bug Fixes

* **deps:** group dependabot updates to reduce PR flood ([1f04a4c](https://github.com/DEVtheOPS/gsbt/commit/1f04a4c65c7f81ffddadf1dfe1c6e2fe06f71207))
* enforce mutually exclusive verbose and quiet flags ([ee87924](https://github.com/DEVtheOPS/gsbt/commit/ee87924f88a35e0c16232afe45a3028f09fa1aec))
* keep rich progress single bar and update per-server ([cfa7834](https://github.com/DEVtheOPS/gsbt/commit/cfa7834dbd34aac8ad6934c26726ef7b099e271c))
* render markup in logger prefixes ([171d2da](https://github.com/DEVtheOPS/gsbt/commit/171d2da26fd2fa333765e652f276c66e99b3e2b7))
* use single rich bar per server ([8383a13](https://github.com/DEVtheOPS/gsbt/commit/8383a13f4d07b2c377ade7215d00181fb6d86907))

## [0.2.1](https://github.com/DEVtheOPS/gsbt/compare/v0.2.0...v0.2.1) (2026-05-26)


### Bug Fixes

* **deps:** group dependabot updates to reduce PR flood ([1f04a4c](https://github.com/DEVtheOPS/gsbt/commit/1f04a4c65c7f81ffddadf1dfe1c6e2fe06f71207))

## Changelog

All notable changes to this project will be documented in this file.

The format is based on Conventional Commits and maintained by Release Please.
