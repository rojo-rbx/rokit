<!-- markdownlint-disable MD023 -->
<!-- markdownlint-disable MD033 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## `1.0.0` - November 13th, 2024

Given that Rokit is already used in production by many Roblox developers, and many months have passed with no new major issues, it is time to release version `1.0.0`.

This comes with a couple guarantees:

- We will not be making any breaking changes to Rokit in the foreseeable future, and if we do, we will release a new major version.
- Rokit's tool storage & tool artifact selection algorithms are stable, and safe to rely on in production environments.
- New features and subcommands may still be added, as long as they do not break existing functionality.

As always, run `rokit self-update` to update to the latest version of Rokit! 🚀

### Changed

- Changed tool downloading to prefer compressed artifacts over uncompressed ones for quicker downloads ([#76])

### Fixed

- Fixed Rokit not being recognized as in PATH for `system-info` and `self-install` commands, unnecessarily prompting the user to restart ([#74])
- Fixed tools such as `lefthook`, with compatible and incompatible artifacts within the same release, not installing correctly ([#76])

[#74]: https://github.com/rojo-rbx/rokit/pull/74
[#76]: https://github.com/rojo-rbx/rokit/pull/76

## `0.2.5` - August 28th, 2024

### Added

- Added support for tool artifacts compressed using gzip (`.gz`) ([#73])

[#73]: https://github.com/rojo-rbx/rokit/pull/73

### Changed

- Changed instructions in `self-install` command on Windows to tell the user to restart their _computer_ instead of their _terminal_ ([#71])

[#71]: https://github.com/rojo-rbx/rokit/pull/71

### Fixed

- Fixed a regression in the `self-update` command that prevented tool links from being updated

## `0.2.4` - August 18th, 2024

### Added

- Added option to display Rokit's changelog in the terminal when running `rokit self-update` ([#56])
- Added a `--check` flag to `rokit update` to check for updates without modifying any tools ([#62])

### Changed

- Removed unnecessary dependencies in the automatic installer script ([#67])
- Changed the `User-Agent` header that Rokit uses for download tools to be more descriptive, hopefully resolving `403 Forbidden` errors ([#68])

[#56]: https://github.com/rojo-rbx/rokit/pull/56
[#62]: https://github.com/rojo-rbx/rokit/pull/62
[#67]: https://github.com/rojo-rbx/rokit/pull/67
[#68]: https://github.com/rojo-rbx/rokit/pull/68

## `0.2.3` - August 9th, 2024

### Fixed

- Fixed cross-device link error when running `self-install` on Linux ([#63])
- Fixed issues with standard I/O streams on Unix systems ([#64])

[#63]: https://github.com/rojo-rbx/rokit/pull/63
[#64]: https://github.com/rojo-rbx/rokit/pull/64

## `0.2.2` - August 3rd, 2024

### Added

- Added a `--skip-parse` flag to `rokit authenticate` to skip token formatting checks

### Fixed

- Fixed `rokit authenticate` not correctly verifying certain GitHub tokens ([#60])

[#60]: https://github.com/rojo-rbx/rokit/pull/60

## `0.2.1` - July 17th, 2024

### Added

- Added a new self-install mechanism to automatically install Rokit on Windows, if Rokit is launched by clicking it in the File Explorer or similar locations ([#52])

### Fixed

- Fixed `rokit system-info` displaying Rokit as not in PATH even when it was ([#50])
- Fixed process group behavior on Windows that was preventing programs spawned by Rokit from exiting properly ([#51])

[#50]: https://github.com/rojo-rbx/rokit/pull/50
[#51]: https://github.com/rojo-rbx/rokit/pull/51
[#52]: https://github.com/rojo-rbx/rokit/pull/52

## `0.2.0` - July 16th, 2024

### Added

- Added a new `--verbose` flag to CLI commands in Rokit for easier debugging when something goes wrong ([#46])

### Changed

- Removed warnings with additional information when searching for tool fallbacks, in favor of using the new `--verbose` flag ([#46])

### Fixed

- Fixed Rokit erroring on first startup due to some directories not yet being created ([#42])
- Fixed `selene` and other tools not being installable because their releases contain multiple archives / binaries ([#45])

[#42]: https://github.com/rojo-rbx/rokit/pull/42
[#45]: https://github.com/rojo-rbx/rokit/pull/45
[#46]: https://github.com/rojo-rbx/rokit/pull/46

## `0.1.7` - July 15th, 2024

### Fixed

- Fixed artifact names with versions in them, such as `lune-0.8.6-linux-x86_64.zip`, no longer installing correctly in Rokit `0.1.6` ([#40])

[#40]: https://github.com/rojo-rbx/rokit/pull/40

## `0.1.6` - July 15th, 2024

### Fixed

- Fixed artifacts with names ending in `win64.zip` or similar not being detected as compatible on Windows ([#39])

[#39]: https://github.com/rojo-rbx/rokit/pull/39

## `0.1.5` - July 14th, 2024

### Fixed

- Fixed tool specifications failing to parse in `foreman.toml` when using inline tables ([#36])
- Fixed tools not specifying architectures (such as `wally-macos.zip`) failing to install ([#38])

[#36]: https://github.com/rojo-rbx/rokit/pull/36
[#38]: https://github.com/rojo-rbx/rokit/pull/38

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
