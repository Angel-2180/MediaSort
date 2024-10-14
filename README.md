# MediaSort - Organize your medias easily with a powerful CLI

**MediaSort** is a fast and efficient command-line tool *(CLI)* designed to help you neatly organize your media files, including TV shows and movies. With support for advanced search using TVMaze and TMDB, MediaSort allows you to automatically sort and rename media based on their title, season, and episode information. Whether you're managing a large media library or simply looking to keep your downloads folder tidy, MediaSort provides a simple yet powerful solution.

## Features

- ⚡ **Blazing Fast**: Processes and sorts large media libraries quickly.
- 📁 **Media Organization**: Automatically organizes media into structured folders based on their metadata.
- 📤 **Webhook Support**: Allows integration with external services via webhooks.
- 🔍 **TVMaze & TMDB Seach**: Automatically fetches and uses metadata from TVMaze and TMDB.
- 🗂️ **Profile Support**: Create custom profiles for different sorting preferences.
- ✨ **More Features Comming Soon !!!**

## Installation

Download the latest release from the [releases page](https://github.com/Angel-2180/MediaSort/releases). The release includes an installer for easier setup.

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
