# Church Presenter

Church Presenter is a lightweight, high-performance presentation software built with **Rust** and **GTK4**. It allows church media teams to present Scripture verses and song lyrics on local displays as well as broadcast them as transparent or themed overlays via **NDI (Network Device Interface)** directly into production software like OBS Studio, vMix, or Wirecast.

---

## Key Features

- 📖 **Scripture Database Integration**: Quick SQL search and lookup of Scripture verses (configured with `KJV.sqlite`).
- 🎶 **Song Stanza & Lyric Presenter**: Manage songs, lyrics, alignments, shadows, scaling, and custom backgrounds.
- 🎨 **Themed Slides & Dynamic Layouts**: Features standard color/image themes (`classic-red`, `royal-blue`, `forest-green`, `dark-slate`, and `black`) with custom branding, logo mode, and blackout/clearout states.
- 📡 **NDI Broadcast**: Instantly output live slides as an uncompressed, alpha-channel transparent video stream over the local network (supported on Windows and Linux).
- ✨ **Fade Transitions**: Smooth animations when transitioning between live slides.
- 🛠️ **Cross-Platform Compatibility**: Fully compatible with Linux, Windows, and macOS (with NDI broadcast compiled conditionally on macOS).

---

## Prerequisites & System Dependencies

To compile and run Church Presenter, you need the **Rust** toolchain installed, along with platform-specific GTK4 libraries.

### 🐧 Linux (Ubuntu/Debian)

Install the GTK4 development packages:

```bash
sudo apt-get update
sudo apt-get install -y libgtk-4-dev build-essential pkg-config
```

### 🏁 Windows (using MSYS2)

The recommended setup for compiling GTK4 apps on Windows is via **MSYS2**:

1. Download and install [MSYS2](https://www.msys2.org/).
2. Open the **MSYS2 UCRT64** terminal and install the GTK4 toolchain:
   ```bash
   pacman -S mingw-w64-x86_64-gtk4 mingw-w64-x86_64-pkg-config mingw-w64-x86_64-gcc
   ```
3. Set your system `PATH` to include your MSYS2 MinGW binary folder (usually `C:\msys64\mingw64\bin`).

### 🍎 macOS

Install GTK4 using **Homebrew**:

```bash
brew install gtk4 pkg-config
```
> **Note:** NDI Broadcast output is currently disabled on macOS because the upstream `ndi` crate does not provide macOS library binaries. The main presentation UI compiles and runs natively.

---

## Getting Started

### 1. Clone and Navigate

```bash
git clone https://github.com/thruqe/Church-Presenter.git
cd Church-Presenter
```

### 2. Run the App

For development mode with debug logs:

```bash
cargo run
```

### 3. Build for Release

To compile an optimized, stripped production binary:

```bash
cargo build --release
```

Compiled binaries will be located under `target/release/`.

---

## Database Configuration

The application expects the SQLite Bible database `KJV.sqlite` to be present in the project root directory. It contains Scripture tables used by the lookup interface.

---

## License

This project is licensed under the MIT License.
