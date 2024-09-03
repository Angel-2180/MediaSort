# MediaSort

MediaSort is a CLI tool that help you to organize your media files.
<br/>
See exemple below

## Features

- ⚡ Blazing Fast
- 📁 Organize your media files
- 📤 Webhook support
- 🔍 Search TV Maze and TMDB for media names
- 🗂️ Profiles implementation
- ✨ And More to come !!!

## Installation

Download the latest release from the [releases page](https://github.com/Angel-2180/MediaSort/releases)

An installer is included with the release to simplify the installation process.

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

### Profiles

`Create Profile`:

```bash
MediaSort profile create --name "Angel" --input "C:\User\Downloads\\" --output "D:\Medias\\"
```

`Edit Profile`:

```bash
MediaSort profile edit --name Angel --key flags --value dry-run=true
```

MediaSort supports the following flags and their defaults value:

- `--search`: true -> for database searching
- `--verbose`: false
- `--webhook`: "default"
- `--threads`: max_cpu thread divided by 2
- `--dry-run`: false
- `--recursive`: false
- `--tv-template`: "Series" -> for folder naming
- `--movie-template`: "Films" -> for folder naming

`Delete Profile`:

```bash
MediaSort profile delete --name Angel
```

## Authors

- [@Angel-2180](https://github.com/Angel-2180)
- [@Hezaerd](https://github.com/Hezaerd)

## Contributing

Contributions are always welcome!

See [`CONTRIBUTING.md`](.) for ways to get started.

Please adhere to this project's [`code of conduct`](./CODE_OF_CONDUCT.md).
