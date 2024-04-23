<!-- markdownlint-disable MD023 -->
<!-- markdownlint-disable MD033 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## `0.0.3` - April 23rd, 2024

### Fixed

- Fixed tools installed by Rokit not being usable on Windows ([#10])

[#10]: https://github.com/filiptibell/rokit/pull/10

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

[#2]: https://github.com/filiptibell/rokit/pull/2

## `0.0.0` - March 29th, 2024

Initial testing release
