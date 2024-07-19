# MediaSort

MediaSort is a CLI tool that help you to organize your media files.
<br/>
See exemple below

## Features

- ⚡ Blazing Fast
- 📁 Organize your media files
- 📤 Webhook support

## Installation

Download the latest release from the [releases page](https://github.com/Angel-2180/MediaSort/releases)

## Usage/Examples

### Before sorting

`Input Directory`:

```
C:/User/Downloads/
|- Blazing Fast.S01E01.VOSTFR.1080p.x264.mp4
|- Blazing Fast.S01E02.VOSTFR.1080p.x264.mp4
|- Blazing Fast.S01E03.VOSTFR.1080p.x264.mp4
|- Blazing Fast.S69E01.VOSTFR.1080p.x264.mp4
|- Blazing Fast.S69E420.VOSTFR.1080p.x264.mp4
```

`Output Directory`:

```
D:/Medias/
```

### After sorting

`Command`:

```bash
MediaSort sort -i "C:/User/Downloads/" -o "D:/Medias/"
```

`Input Directory`:

```
C:/User/Downloads/
```

`Output Directory`:

```base
D:/Medias/
|- Series/
|  |- Blazing Fast/
|  |  |- S01/
|  |  |  |- Blazing Fast - E01.mp4
|  |  |  |- Blazing Fast - E02.mp4
|  |  |  |- Blazing Fast - E03.mp4
|  |  |- S69/
|  |  |  |- Blazing Fast - E01.mp4
|  |  |  |- Blazing Fast - E420.mp4
```

## Authors

- [@Angel-2180](https://github.com/Angel-2180)
- [@Hezaerd](https://github.com/Hezaerd)

## Contributing

Contributions are always welcome!

See [`CONTRIBUTING.md`](.) for ways to get started.

Please adhere to this project's [`code of conduct`](./CODE_OF_CONDUCT.md).
