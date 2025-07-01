# CastModelViewer (The Rust Edition)

CastModelViewer is a WIP, Rust-based tool for viewing and previewing `.cast` model files. It is a full rewrite of the original [CastModelViewer](https://github.com/echo000/CastModelViewer/tree/old-csharp) using [@dtzxporter/porter-lib](https://github.com/dtzxporter/porter-lib) for cross-platform model parsing and rendering.

---

## Features

- Loads and previews Cast model files (`.cast`)
- Cross-platform: Windows, Linux, macOS

---

## Getting Started

### Usage

- **Open a model:** You can load `.cast` files by either:
  - Dragging and dropping the file into the CastModelViewer window
  - Clicking the "Load File" button in the tool and selecting your file
- **Open the Preview:**
  - Click on the file in the asset view and press `P` to open the model preview

### Prerequisites

- [Rust](https://rust-lang.org/) (latest stable recommended)
- A supported platform (Windows, Linux, or macOS)

### Building

Clone the repository:
```sh
git clone https://github.com/echo000/CastModelViewer.git
cd CastModelViewer
```

Build and run:
```sh
cargo run --release
```

---

## Requirements

- Rust (latest stable)
- Porter-lib (automatically managed via `Cargo.toml`)

---

## Download

You can build from source using Cargo, or download the old prebuilt releases from the [Releases Page](https://github.com/echo000/CastModelViewer/releases).

---

##  Disclaimers

> **Disclaimer:**  
> CastModelViewer is provided "as-is" with no warranty. Use at your own risk.

---

## Credits

- **Scobalula**  
  - [SEModelViewer](https://github.com/Scobalula/SEModelViewer/) – Inspiration for Cast model viewing
- **DTZxPorter**  
  - [porter-lib](https://github.com/dtzxporter/porter-lib)
  - [Cast](https://github.com/dtzxporter/cast)

---

## Contributing

Contributions, bug reports, and suggestions are welcome!
Feel free to open an issue or pull request.

---
