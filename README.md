<div align="center">

<h1>Shellshot</h1>

[![Crates.io](https://img.shields.io/crates/v/shellshot)](https://crates.io/crates/shellshot)
[![Build Status](https://img.shields.io/github/actions/workflow/status/lhenry-dev/shellshot/ci.yml?branch=main)](https://github.com/lhenry-dev/shellshot/actions/workflows/ci.yml?branch=main)
[![Dependency Status](https://deps.rs/repo/github/lhenry-dev/shellshot/status.svg)](https://deps.rs/repo/github/lhenry-dev/shellshot)
[![Documentation](https://docs.rs/shellshot/badge.svg)](https://docs.rs/shellshot)
[![License](https://img.shields.io/crates/l/shellshot)](https://crates.io/crates/shellshot)
[![MSRV](https://img.shields.io/badge/MSRV-1.85.1-dea584.svg?logo=rust)](https://github.com/rust-lang/rust/releases/tag/1.85.1)
[![codecov](https://codecov.io/gh/lhenry-dev/shellshot/graph/badge.svg?token=UA9AAN26IO)](https://codecov.io/gh/lhenry-dev/shellshot)

---

**Transform your command-line output into clean, shareable images with a single command.**

</div>

`Shellshot` is a fast, cross-platform tool written in Rust that captures terminal sessions and transforms them into polished screenshots. Perfect for documentation, presentations, social media, or showcasing terminal workflows.

## Features

- **Beautiful Rendering**: High-quality image generation with customizable window decorations
- **ANSI Support**: Correctly renders ANSI colors, styles, and formatting.
- **Clipboard Integration**: Copy screenshots directly to your clipboard with one flag
- **Command Execution**: Execute commands and capture their output automatically
- **Customizable**: Adjust window decorations, colors, padding, and output filename.
- **Cross-Platform**: Works on Windows and Linux

## Installation

```bash
cargo install shellshot
```

## Usage Examples

### Usage Notes

- On Windows, some commands may require `--shell` to execute correctly (forces execution inside Bash on Windows).
- Either `--output <file>` or `--clipboard` must be specified, otherwise `shellshot` will fail.

### Basic Usage

On Linux, commands usually work directly:

```bash
# Linux
shellshot -o out.png echo "Hello from ShellShot!"
```

On Windows, some commands (like `echo`) are shell builtins, not executables.
You need to force execution inside a shell using [`--shell`](#shell--force-execution-inside-a-shell)

```bash
# Windows
shellshot --shell -o out.png echo "Hello from ShellShot!"
```

This will execute the command, capture its output, and generate an image file named `out.png` in the current directory.

![echo example](docs/echo_example.png)

```bash
shellshot -o out.png ping -c 5 localhost
```

![ping example](docs/ping_example.png)

### Command Options

#### `--shell` — Force execution inside a shell

The `--shell` flag forces Shellshot to execute the command **inside a shell** instead of running it directly.

**Why this is needed:**

- **Linux/macOS**: Forces execution inside `sh`. Most commands are either executables or shell builtins, so they usually run correctly without `--shell`. Use it if you want consistent shell behavior (e.g., for complex scripts or shell operators like pipes and redirects).
- **Windows**: Forces execution inside Bash. Many common commands like `echo` or `dir` are **shell builtins**, not standalone executables. Using `--shell` ensures these commands run correctly.

**Example:**

```bash
# Linux — works directly
shellshot -o out.png echo "Hello from ShellShot!"

# Windows — must use --shell because echo is a shell builtin
shellshot --shell -o out.png echo "Hello from ShellShot!"
```

#### `--no-decoration`

Remove window decorations (title bar and control buttons):

```bash
shellshot -o out.png --no-decoration node --version
```

#### `--decoration <style>` / `-d`

Specify the decoration style (default: `classic`):

```bash
# Linux
shellshot -o out.png --decoration classic ls --color=always
# Windows
shellshot --shell -o out.png --decoration classic dir
```

#### `--output` / `-o`

Specify a custom output filename:

```bash
shellshot --output out.png cargo build
shellshot -o screenshots/out.png cargo test
```

#### `--clipboard`

Copy the screenshot directly to your clipboard:

```bash
shellshot --clipboard git status
```

#### `--width` / `-W` et `--height` / `-H`

Specify the final image dimensions in **columns** (width) and **rows** (height), or use `'auto'` (default: auto):

```bash
# Linux
shellshot -o out.png --width 70 --height 50 echo "Hello, world!"
# Windows
shellshot --shell -o out.png --width 70 --height 50 echo "Hello, world!"
```

#### `--timeout` / `-t`

Set a timeout in seconds for command execution:

```bash
shellshot -o out.png --timeout 5 ping -c 10 localhost
```

### Examples

```bash
shellshot -o out.png cargo --version
shellshot --clipboard git log --oneline -5
shellshot -o out.png --no-decoration python --version

# Linux
shellshot -o out.png echo "Hello, Shellshot!"
shellshot -o out.png --decoration classic ls --color=always

# Windows
shellshot --shell -o out.png echo "Hello, Shellshot!"
shellshot --shell -o out.png --decoration classic dir
```
