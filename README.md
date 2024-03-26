<!-- markdownlint-disable MD023 -->
<!-- markdownlint-disable MD026 -->
<!-- markdownlint-disable MD033 -->

# 🚀 Rokit

Next-generation toolchain manager for Roblox projects.

## Features

- ⚡ Tools download and install **_really fast_**.
- ♻️ Compatible with projects that already use [Foreman][foreman] or [Aftman][aftman].
- 📝 Useful output messages that are easy for humans to read and understand.

## Installation

Run the installation command for your platform, found below, in a terminal. <br/>
Rokit will guide you through the rest.

### macOS & Linux

<details> <summary> <b>Bash / Zsh</b> </summary>

```sh
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/filiptibell/rokit/main/scripts/install.sh | sh
```

</details>

### Windows

<details> <summary> <b>PowerShell</b> </summary>

```ps1
iex "& { $(irm https://raw.githubusercontent.com/filiptibell/rokit/main/scripts/install.ps1) } RunJob"
```

</details>

<details> <summary> <b>CMD (Legacy)</b> </summary>

```bat
# TODO: Install script using cmd.exe
```

</details>

### Upgrading

Run `rokit self-update` in your terminal.

## Q & A

<details> <summary> <b>Why use Rokit over Foreman or Aftman?</b> </summary>

### For a new Roblox developer

Rokit is the _fastest_ and _friendliest_ way to get set up with tooling for a new Roblox project. <br/>
Installation is completely automated and you do not need to manually edit any files to get your tools working.

### For everyone else

Foreman and Aftman have an uncertain future as toolchain managers for the community. <br/>
Most of the existing problems boil down to issues with maintainership:

- Foreman is maintained by Roblox itself.
- Aftman is maintained by a third party that is no longer interested in Roblox.

Rokit aims to solve this by taking a community-first approach and being built with community contributions in mind. <br/>
Rokit also acknowledges that developers will not migrate from any of the existing toolchain managers _without good reason_, it needs to be **_substantially better_**.

</details>

<details> <summary> <b>How do you pronounce Rokit?</b> </summary>

### However you want.

- "Rocket" for speed
- "Ro-kit" for Roblox-y flair
- "Rock-it" if you're feeling groovy

</details>

[foreman]: https://github.com/Roblox/foreman
[aftman]: https://github.com/LGPhatguy/aftman
