# nowascii

Show the album cover for the music currently playing as colorful, square ASCII-style art in your terminal. `nowascii` reads track metadata from MPRIS, the standard media-player interface used by Linux desktop players.

The image is center-cropped to a square and rendered with colored ASCII density characters (`@%#*+=-:.`). Output uses about half as many rows as columns to compensate for the taller shape of common terminal cells.

## Requirements

- Linux desktop session with a D-Bus session bus.
- A player exposing MPRIS metadata and an album-art URL (for example, Spotify, Firefox, VLC, or Rhythmbox; support depends on the player and how it was installed).
- A terminal with Unicode and 24-bit ANSI color support.
- Network access only when the player provides a remote `http(s)` cover URL.

## Install from a release (x86_64)

Download the `nowascii-linux-x86_64` executable from the [latest GitHub Release](https://github.com/pneftekin/nowascii/releases/latest).

### Arch Linux

```sh
sudo install -Dm755 nowascii-linux-x86_64 /usr/local/bin/nowascii
```

Or download and install directly:

```sh
curl -fL "https://github.com/pneftekin/nowascii/releases/latest/download/nowascii-linux-x86_64" -o /tmp/nowascii
sudo install -Dm755 /tmp/nowascii /usr/local/bin/nowascii
```

### Debian / Ubuntu / Linux Mint

Install `curl` if needed, then use the same download and `install` commands above. `/usr/local/bin` is on the default shell `PATH` on standard installations.

### Fedora

Install `curl` if needed, then use the same download and `install` commands above. No compiler or Python environment is needed for the release binary.

### openSUSE

Install `curl` if needed, then use the same download and `install` commands above.

The release binary is for **x86_64 (64-bit Intel/AMD)** Linux. ARM users can build from source using the instructions below. Release builds target a broadly compatible glibc Linux environment; very old systems may need a source build on that system.

To update, repeat the download and install commands. To uninstall:

```sh
sudo rm /usr/local/bin/nowascii
```

## Build from source

Install Rust and Cargo using your distribution package manager or [rustup](https://rustup.rs/), then run:

```sh
git clone https://github.com/pneftekin/nowascii.git
cd nowascii
cargo install --path .
```

Cargo installs the executable to `~/.cargo/bin`. Make sure that directory is on your `PATH`. This method works on Arch, Debian/Ubuntu, Fedora, openSUSE, and other Linux distributions with a Rust toolchain. It downloads the Rust dependencies on first build.

### Try it on Arch Linux

From the `nowascii` project directory:

```sh
sudo pacman -S --needed rust
cargo run --release -- --width 40
```

Start a track in your usual music player first. This builds the standalone release executable in `target/release/nowascii` and runs it; no virtual environment is involved. To install it for your user after it works:

```sh
cargo install --path .
nowascii --width 40
```

Or install the already-built binary system-wide:

```sh
sudo install -Dm755 target/release/nowascii /usr/local/bin/nowascii
nowascii --width 40
```

To build a standalone binary in the project directory:

```sh
cargo build --release
./target/release/nowascii
```

## Usage

Start playing a track in an MPRIS-compatible player, then run:

```sh
nowascii
nowascii --width 48
nowascii -w 64
```

Default width is 32 terminal cells; allowed widths are 4–200. The rendered height is half the width in rows, which makes the art approximately square in a typical terminal. Only the colored art is printed to standard output.

```text
nowascii [--width N]
```

Use `nowascii --help` for the options. If multiple players are open, the first player reporting `Playing` is used. If none is playing, the first player with readable metadata is used.

## Troubleshooting

- **No MPRIS players found:** Start playback in a media player that supports MPRIS and run `nowascii` from the same logged-in desktop session.
- **No album art:** Some players do not expose `mpris:artUrl` for a track. Check the player's MPRIS support and whether that track has cover art.
- **No color:** Use a terminal with 24-bit ANSI color enabled.
- **Command not found:** Confirm `/usr/local/bin` (release install) or `~/.cargo/bin` (Cargo install) is on your `PATH`.

## License

MIT. See [LICENSE](LICENSE).

