
# Swatchify

## Summary

Bulk generates customizable filament swatches based on the [Filament Swatch](https://www.printables.com/model/27814-filament-swatch) by Makkuro from inventory data in a CSV file.

## Features

- reads a CSV file with the manufacturer, material, color and print temperature information
- generates the right parameter set to render filament swatches
- outputs a hierarchical filesystem tree with swatches grouped by material and manufacturer
- parallelizes the rendering process by running one OpenSCAD process per (logical) CPU core.

## Tech

- Written in [Rust](https://www.rust-lang.org/)
- Concurrent processing based on [Rayon](https://github.com/rayon-rs/rayon)

## Usage

Run `swatchify --help` and `swatchify generate --help` to see supported options.
Running `swatchify generate --inventory 'Filaments.csv' -d Swatches` will read filament details from the `Filaments.csv` file and render the filament swatches into the `Swatches` directory. Missing output directories are automatically created.

By default existing filament swatches are not re-created to save time, add `--force` as an option to recreate all filament swatches from scratch.

The CSV file is expected to have four columns: manufacturer, material, color and temperature. The order of the columns should not matter as long as the first line of the CSV file is a header line.

Example CSV file:

```csv
anufacturer,material,color,temperature
eSun,PLA,Orange,210
Fiberlogy,PLA,Vertigo,215
Hatchbox,PETG,Black,235
Hatchbox,PLA,White,215
```

## Acknowledgments

[Customizable Filament Swatch](https://www.printables.com/model/27814-filament-swatch) by [Makkuro](https://www.printables.com/social/13788-makkuro/about) shared under the [CC BY-SA 4.0](http://creativecommons.org/licenses/by-sa/4.0/) License.

## License

[MIT](./LICENSE)
