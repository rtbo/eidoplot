# eidoplot

A simple and minimal data plotting library for Rust.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

Eidoplot separates figure design from data and rendering surfaces.

### Key Features

- **Supported plot types:**
  - XY line plots
  - Scatter plots
  - Histograms and bars

- **Modular architecture:**
  - Declarative Intermediate Representation (IR) for figure design
  - Separate rendering surfaces (SVG, pixels)
  - Data is separated from figure design.
  - Support for multiple data sources (CSV, Polars)
  - Figure units are decorrelated from pixel size for easy scaling

- **Theme**
  - A theme can be applied to change the look of a figure with a single line of code.

- **Automatic Layout**
  - All the layout is done consistently and automatically. You can add multiple axes,
  multiple plots etc. All will be laid-out in a consistent way, leaving enough space
  for axis ticks labels, legends etc. without a single line of code on the user side.

- **GUI integration and real-time rendering**
 - The package `iced-oplot` provides a [iced](https::/github.com/iced-rs/iced.git) `Figure` widget.
Thanks to separation of data from design, redraws of the same figure with different data
is very efficient and compatible with real-time rendering, up to hundreds of redraws per second.

- **Declarative DSL:**
  - `.eplt` language for concise figure description.
This DSL is fairly incomplete, but all examples in the repo are working.


## Installation

Add eidoplot to your `Cargo.toml`:

```sh
cargo add eidoplot
cargo add eidoplot-pxl # for rasterized rendering (e.g. pixels, PNG)
cargo add eidoplot-svg # for SVG rendering
```

### Available Features

- `data-csv` (enabled by default): CSV file support
- `data-polars`: Polars DataFrames support
- `dsl-diag`: Diagnostics for `.eplt` DSL
- `utils` (enabled by default): Utility functions (linspace, logspace, etc.)
- `noto-sans` (enabled by default): Bundles the Noto-Sans font in the executable.

## Architecture

### Intermediate Representation (IR)

The `ir` module contains a declarative representation of figures, independent of rendering. This allows:
- Separation of design logic from rendering logic
- A stable intermediate format
- In the future, an easy mapping to other programming languages

### Rendering Surfaces

Rendering surfaces implement the `render::Surface` trait and are in separate crates:
- **`eidoplot-svg`**: SVG format rendering
- **`eidoplot-pxl`**: Bitmap rendering (PNG, etc.)

### Drawing Module

The `drawing` module bridges the IR and rendering surfaces. It translates abstract IR concepts into drawing primitives (lines, text, etc.).

## Examples

The project includes several examples in the `examples/` folder:

- `sine`: Simple sine wave
- `bode_rlc`: Bode diagram of RLC circuit
- `gauss`: Normal distribution with histogram
- `iris`: Iris dataset with scatter plot
- `bars`: Bar charts
- `bitcoin`: Time series data
- `subplots`: Figures with multiple subplots
- `multiple_axes`: Plots with multiple axes

To run an example:

```bash
cargo run --example sine -- svg png
cargo run --example bode_rlc -- svg png macchiato
cargo run --example gauss -- svg png
cargo run --example iris --features data-csv -- svg png
```

## Workspace Crates

- **`eidoplot`**: Main library
- **`eidoplot-base`**: Base types (colors, geometry)
- **`eidoplot-dsl`**: Parser for `.eplt` DSL
- **`eidoplot-svg`**: SVG rendering backend
- **`eidoplot-pxl`**: Pixel rendering backend (suitable for PNG exports)
- **`iced-oplot`**: Figure widget for iced
- **`eidoplot-text`**: Text and font management
- **`eidoplot-tests`**: Integration tests

## Polars Integration

With the `data-polars` feature, you can use Polars DataFrames as `eidoplot::data::Source` directly.


## License

This project is distributed under the MIT License. See the [LICENSE](LICENSE) file for details.

Copyright (c) 2025 Rémi Thebault

## Contributing

Contributions are welcome! Feel free to open issues or pull requests on the [GitHub repository](https://github.com/rtbo/eidoplot).
