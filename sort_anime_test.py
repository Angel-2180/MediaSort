import logging
import re

# Configure logging
logging.basicConfig(level=logging.INFO)

def get_series_name(filename):
    # Replace all spaces with dots
    filename = filename.replace(' ', '.')

    # Remove content within square brackets at the beginning of the filename
    filename = re.sub(r'^\[.*?\]\s*', '', filename)

    # Remove content within parentheses
    filename = re.sub(r'\(.*?\)', '', filename)

    # Remove season and episode information
    filename = re.sub(r'(\.S\d+E\d+|\.\d{2,3}|-\d{2,3}-|-\s*\d{2,3}\s*-)', '', filename)

    # Remove quality information (e.g., 720p, 1080p)
    filename = re.sub(r'\b\d{3,4}p\b', '', filename)

    # Split the filename by season number (e.g., S01, S1, E01)
    split_filename = re.split(r'(\.S\d+|\.S\d+E\d+|\.E\d+|\.\d{2,3}\s)', filename, 1)
    if len(split_filename) > 1:
        series_name_part = split_filename[0]
    else:
        series_name_part = filename

    # Split the series name part by common delimiters and filter out unwanted parts
    parts = re.split(r'[.\s-]+', series_name_part)

    # Keywords to identify non-series parts
    ignore_keywords = [
        'VOSTFR', 'FRENCH', 'WEB', 'x264', 'AAC', '720p', '1080p', 'MULTI',
        'TsundereRaws', 'Tsundere', 'Raws', 'Wawacity', 'vostfree', 'tv', 'com',
        'mp4', 'mkv', 'www', 'CR', '(CR)', 'uno', 'boats', 'p', '0p', 'h264', 'h265', 'x265', 'x264', 'WEBRip',
        'WEB-DL', 'city',
        '1080p', '720p', '480p', '360p', '2160p', 'HEVC', 'x265', 'x264', 'AAC', 'AC3', '5.1', '2.0', '10bit', '8bit',
        'BluRay', 'BluRayRip',
        'H.264', 'H.265'
    ]

    # Combine the ignore keywords into a single regex pattern
    ignore_pattern = re.compile('|'.join(ignore_keywords), re.IGNORECASE)

    # Filter out parts that match any of the ignore keywords
    series_parts = [part for part in parts if not ignore_pattern.match(part)]

    # Combine the remaining parts to form the series name
    series_name = ' '.join(series_parts)

    # Cleanup: replace multiple spaces with a single space and strip leading/trailing spaces
    series_name = re.sub(r'\s+', ' ', series_name).strip()

    return series_name


def get_season_number(filename):
    # Use regex to extract season number, e.g., S01E03 -> 01 or S1E03 -> 1
    match = re.search(r'[Ss](\d{1,2})[Ee]?(\d{2})?', filename)
    if match:
        season_number = match.group(1).zfill(2)  # Pad single-digit seasons with a zero
        logging.info(f"Extracted season number: {season_number} from filename: {filename}")
        return season_number
    logging.warning(f"Could not extract season number from filename: {filename}")
    return '01'  # Default to season 01 if not found


def get_episode_number(filename):
    # Use regex to extract episode number, e.g., E03 -> 03
    episode_pattern = r'[Ee](\d{2})'
    match = re.search(episode_pattern, filename)
    if match:
        episode_number = match.group(1)
        logging.info(f"Extracted episode number: {episode_number} from filename: {filename}")
        return episode_number
    logging.warning(f"Could not extract episode number from filename: {filename}")
    return '01'  # Default to episode 01 if not found


def test_get_series_name():
    test_anime = [
        "A.Sign.of.Affection.S01E01.VOSTFR.1080p.WEB.x264-TsundereRaws-Wawacity.boats.mkv",
        "My Deer Friend Nokotan S01E01 VOSTFR 1080p WEB x264 AAC -Tsundere-Raws (CR).mp4",
        "Edens.Zero.S02E01.FRENCH.1080p.WEB.x264-TsundereRaws-Wawacity.uno.mkv",
        "Dragon Ball 001 Bulma et Son Goku.mkv",
        "[Mixouille] Bleach Kai - 01 - Les gardiens de nos âmes - 720p.MULTI.x264.mkv",
        "Komi-san.wa.Komyushou.Desu.05.VOSTFR.1080p.www.vostfree.tv.mp4",
        "Kaguya-sama.wa.Kokurasetai.Ultra.Romantic.06.VOSTFR.1080p.www.vostfree.tv.mp4",
        "Chou.Kadou.Girl.⅙.Amazing.Stranger.07.VOSTFR.720p.www.vostfree.com.mp4"
    ]

    for anime in test_anime:
        series_name = get_series_name(anime)
        season_number = get_season_number(anime)
        episode_number = get_episode_number(anime)
        print(f"Filename: {series_name}")
        print(f"Season number: {season_number}")
        print(f"Episode number: {episode_number}")
        print("-" * 20)


if __name__ == "__main__":
    test_get_series_name()
