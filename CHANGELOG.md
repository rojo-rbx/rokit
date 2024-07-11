<!-- markdownlint-disable MD023 -->
<!-- markdownlint-disable MD033 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## `0.1.4` - July 11th, 2024

### Fixed

- `rokit self-update` will no longer encounter an OS error on Windows systems ([#33])

[#33]: https://github.com/rojo-rbx/rokit/pull/33

## `0.1.3` - June 18th, 2024

### Changed

- Overhauled the `rokit list` subcommand to be more useful and distinct from `rokit system-info`
- Improved the formatting of Rokit manifests created using `rokit init`

### Fixed

- Fixed the "restart terminal" hint being shown after a `rokit self-install`, even if restarting isn't necessary for Rokit to function

## `0.1.2` - June 14th, 2024

### Fixed

- Fixed `tarmac` not installing correctly on non-arm systems due to its name containing `arm` ([#26])
- Fixed OS permission errors during `rokit install` for tools that are currently running ([#27])
- Fixed `tarmac` not installing correctly on non-mac systems due to its name containing `mac` ([#28])

[#26]: https://github.com/rojo-rbx/rokit/pull/26
[#27]: https://github.com/rojo-rbx/rokit/pull/27
[#28]: https://github.com/rojo-rbx/rokit/pull/28

## `0.1.1` - June 9th, 2024

### Fixed

- Fixed nested processes (processes spawned by processes spawned by rokit) hanging on macOS

## `0.1.0` - June 9th, 2024

### Added

- Added an automatic retry mechanism for network requests, making installation more robust ([#24])

### Changed

- Rokit now falls through to system-wide installations of tools when a tool is not managed using a Rokit manifest ([#25])

### Fixed

- Fixed zombie processes being left around on Windows ([#23])

[#23]: https://github.com/rojo-rbx/rokit/pull/23
[#24]: https://github.com/rojo-rbx/rokit/pull/24
[#25]: https://github.com/rojo-rbx/rokit/pull/25

## `0.0.7` - May 8th, 2024

### Added

- Added warnings when tool aliases and specs fail to parse

## `0.0.6` - May 2nd, 2024

### Fixed

- Fixed issues with UNC paths on Windows

## `0.0.5` - May 1st, 2024

### Fixed

- Fixed error messages being written to stdout instead of stderr ([#14])

[#14]: https://github.com/rojo-rbx/rokit/pull/14

## `0.0.4` - April 24th, 2024

### Fixed

- Fixed panics for manifests with a missing tools section ([#15])
- Fixed panics when running `rokit install` or `rokit add` ([#16])
- Fixed tool links missing executable extensions on Windows ([#18])

[#15]: https://github.com/rojo-rbx/rokit/pull/15
[#16]: https://github.com/rojo-rbx/rokit/pull/16
[#18]: https://github.com/rojo-rbx/rokit/pull/18

## `0.0.3` - April 23rd, 2024

### Fixed

- Fixed tools installed by Rokit not being usable on Windows ([#10])

[#10]: https://github.com/rojo-rbx/rokit/pull/10

## `0.0.2` - April 2nd, 2024

### Breaking Changes

Tools are now stored in a case-insensitive manner to prevent unnecessary downloading and linking of duplicate tool specifications. This means that tools in manifests that are not all lowercase may no longer work on case-sensitive filesystems. To fix this, remove the `~/.rokit/tool-storage` directory, and Rokit will re-download and install tools next time you run `rokit install`.

### Fixed

- Fixed tool aliases being case-sensitive
- Fixed Rokit process being killed during self-update
- Fixed common tool name shortcuts in the CLI being case-sensitive
- Fixed tool storage and tool trust cache being case-sensitive

## `0.0.1` - March 31st, 2024

### Added

- Added new `authenticate` subcommand to add auth tokens for GitHub and future artifact providers.

### Fixed

- Fixed `self-install` failing to add Rokit to `$PATH` if some shell configuration files don't exist. ([#2])

[#2]: https://github.com/rojo-rbx/rokit/pull/2

## `0.0.0` - March 29th, 2024

Initial testing release
