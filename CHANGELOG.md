# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.3] - 2025-02-12
### Changes
- Update `hashbrown` to 0.15 #157

## [0.9.2] - 2024-06-05
### Added
- `Font.name()` via AcrylicShrimp #141

## [0.9.1] - 2024-06-05
### Fixed
- Incorrect attribute configuration for not_std web builds

## [0.9.0] - 2024-05-13
### Added
- Flag to conditionally use std hashmap over `hashbrown`
### Changes
- Update `ttf-parser` to 0.21
- Update `rayon` to 1.10

## [0.8.0] - 2023-11-25
### Added
- `Font.has_glyph()` for convenience.
### Changes
- The 0.7.4 breaks semver by adding a field to FontSettings. 0.7.4 is yanked and republished as 0.8.0

## [0.7.4] - 2023-11-25
### Added
- Support Loading Ligature & Substitution Glyphs.
### Changes
- Update `ttf-parser` to 0.20
- Update `hashbrown` to 0.14
### Fixed
- Fixed a potential unaligned read on x86.

## [0.7.3] - 2023-04-16
### Added
- Expose layout settings on `Layout`
- Add a line height option to LayoutSettings
### Changes
- Relicense to MIT OR Apache-2.0 OR Zlib
- Update `hashbrown` to 0.13
- Refactored development related artifacts into the `dev` folder. This reduces pollution in the main crate.
- Removed some needless unsafe, documented other uses.
### Fixed
- More doc typos.

## [0.7.2] - 2022-03-03
### Added
- Added `byte_offset` to GlyphPosition
### Changes
- Breaking - Renamed `line_start`/`line_end` to `glyph_start`/`glyph_end`
### Fixed
- More doc typos.
- `line_start`/`glyph_start` skipping spacing characters.

## [0.7.1] - 2022-02-25
### Changes
- `ttf-parser` updated to 0.15
### Fixed
- `LinePosition` doc typo.
- Benign compiler error in debug mode in layout.

## [0.7.0] - 2022-02-25
### Added
- A changelog.
- `simd` flag, enabled by default. Leverages simd functions. This was implicitly always enabled prior.
- `parallel` flag, disabled by default. Uses `std` + `rayon` to thread font loading.
- `Font.chars()` gets all valid unicode codepoints that have mappings to glyph geometry in the font.
- `LinePosition` holds various metadata on positioned lines computed during layout.
### Changed
- `Layout.lines()` returns a `Option<Vec<LinePosition>>` now instead of a line count.