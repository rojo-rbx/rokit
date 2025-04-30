<!-- markdownlint-disable MD023 -->
<!-- markdownlint-disable MD026 -->
<!-- markdownlint-disable MD033 -->

# 🚀 Rokit

Next-generation toolchain manager for Roblox projects.

## Features

- ⚡ Tools download and install **_really fast_**.
- ♻️ Drop-in compatibility with projects that already use [Foreman][foreman] or [Aftman][aftman].
- 📦 Supports additional formats and compressed files, such as `.tar` and `.tar.gz`.
- 🤖 Additional commands for adding and updating tools, and even updating Rokit itself.
- 📝 Useful output messages that are easy for humans to read and understand.

## Installation

Follow the instructions for your platform below - when installed, Rokit will guide you through the rest.

### macOS & Linux

Run the automated installer script in your terminal:

```sh
curl -sSf https://raw.githubusercontent.com/rojo-rbx/rokit/main/scripts/install.sh | bash
```

### Windows

#### Option 1

Run the automated installer script in PowerShell:

```sh
Invoke-RestMethod https://raw.githubusercontent.com/rojo-rbx/rokit/main/scripts/install.ps1 | Invoke-Expression
```

#### Option 2

Download and run<sup>\*</sup> `rokit.exe` from the [latest release][latest-release] page - this will automatically install Rokit.

<sup>\* Make sure to run `rokit.exe` **directly**, by double-clicking it in the File Explorer, for automatic installation to be triggered. <br/> If you would like to install Rokit from a terminal or shell such as PowerShell / CMD, run `rokit.exe self-install` instead.
</sup>

### Other

<details> <summary> <b>Installing from source</b> </summary>

Rokit can be compiled and installed from source using [`cargo`][rustup]:

```sh
cargo install rokit --locked # Installs the Rokit binary
rokit self-install # Initializes necessary directories and data files for Rokit to work
```

This _may_ work on systems that Rokit is not officially compatible with, but note that no support is provided for non-official targets. <br/>
Check the [latest GitHub release][latest-release] to see which targets Rokit is compatible with.

</details>

## Usage

Running `rokit --help` in your terminal will give you a full overview of all available commands. <br/>
Running `rokit command-name --help` will give you full details about a _specific_ command.

<details> <summary> <b>Brief overview of available commands</b> </summary>

- `rokit init` - Initializes a new project in the current directory.
- `rokit add` - Adds and installs a tool.
- `rokit list` - Lists all currently installed tools.
- `rokit install` - Installs all project-specific tools.
- `rokit update` - Updates a specific tool, or all project-specific tools, to the latest version.
- `rokit authenticate` - Authenticates with GitHub or other artifact providers.
- `rokit self-update` - Updates Rokit itself to the latest version.
- `rokit self-install` - Installs Rokit itself and updates tool executable links.

</details>

## Q & A

<details> <summary> <b>Why use Rokit over Foreman or Aftman?</b> </summary>

### For a new Roblox developer

Rokit is the _fastest_ and _friendliest_ way to get set up with tooling for a new Roblox project. <br/>
Here are some key differences between Rokit and Foreman / Aftman:

- Usable in _any_ project, even projects using `foreman.toml` / `aftman.toml`, while Foreman / Aftman are incompatible with each other by design.
- Massively improved installation times and more consistent cross-platform behavior.
- No manual editing of manifest or link files.

### For everyone else

While Foreman and Aftman have been working great for a long time, they have an uncertain future as toolchain managers for the community. <br/>
Most of their existing problems boil down to issues with maintainership:

- [Foreman][foreman] is maintained by Roblox itself, and has angled itself towards internal uses at Roblox.
- [Aftman][aftman] is maintained by a third party that is no longer interested in Roblox.

Rokit aims to solve this by taking a community-first approach and being built with community contributions in mind. <br/>
Rokit also acknowledges that developers will not migrate from any of the existing toolchain managers _without good reason_, it needs to be **_substantially better_** and require **_virtually no effort_** to migrate.

</details>

<details> <summary> <b>How do you pronounce Rokit?</b> </summary>

### However you want.

- "Rocket" for speed
- "Ro-kit" for Roblox-y flair
- "Rock-it" if you're feeling groovy

</details>

[rustup]: https://rustup.rs
[foreman]: https://github.com/Roblox/foreman
[aftman]: https://github.com/LPGhatguy/aftman
[latest-release]: https://github.com/rojo-rbx/rokit/releases/latest
