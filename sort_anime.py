import os
import shutil
import logging
import re
import requests
from dotenv import load_dotenv

# Set up logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
load_dotenv()
download_folder = "E:/Angel_/telechargment"
nas_root_folder = "//10.0.0.27/Anime"
webhook_url = os.getenv("DISCORD_WEBHOOK_URL")



class Episode:
    def __init__(self, filename):
        self.filename = filename
        self.filename_clean = self.clean_filename(filename)
        self.name: str = "unknown"
        self.season: int = 0
        self.episode: int = 0
        self.extension = filename.split(".")[-1]


        self.fetch_infos()

        print(self.__str__())

    def fetch_infos(self):
        self.name = self.extract_series_name()
        self.season = self.extract_season()
        self.episode = self.extract_episode()

    def __str__(self):
        return f"*{self.name}* - **S{self.season:02d}E{self.episode:02d}**"

    def clean_filename(self, file_to_clean) -> str:
        # remove urls
        file_to_clean = re.sub(r'(www\..*?\..{2,3})', '', file_to_clean)

        # remove common separators
        file_to_clean = re.sub(r'[-_.+]', ' ', file_to_clean)

        # remove content within parentheses
        file_to_clean = re.sub(r'\(.*?\)', '', file_to_clean)

        # remove content within brackets
        file_to_clean = re.sub(r'\[.*?\]', '', file_to_clean)

        # Remove common video file extensions
        file_to_clean = re.sub(r'(mkv|mp4|avi|wmv|flv|mov|webm)', '', file_to_clean)

        # Remove common video quality indicators (e.g 1080p, 720p)
        file_to_clean = re.sub(r'\b\d{3,4}p\b', '', file_to_clean)

        # remove common codec indicators
        file_to_clean = re.sub(r'(x264|x265|HEVC|MULTI|AAC)', '', file_to_clean)

        # remove language
        file_to_clean = re.sub(r'(FRENCH|VOSTFR|VOSTA|VF|VO)', '', file_to_clean)

        # clean urls
        file_to_clean = re.sub(r'(www|com|vostfree|boats|uno|Wawacity|WEB|TsundereRaws|Tsundere|Raws|fit)', '', file_to_clean)

        # remove consecutive spaces
        file_to_clean = ' '.join(file_to_clean.split())

        # remove first and last spaces
        file_to_clean = file_to_clean.strip()

        return file_to_clean

    def extract_series_name(self) -> str:
        # possible patterns: S01E01, S01, S1
        name_patterns = [
            r'(.+?)(S\d{1,2}E\d{1,2}|S\d{1,2})',
            r'(.+?)(\d{1,3})'
        ]

        for pattern in name_patterns:
            fetched_name = re.search(pattern, self.filename_clean)

            if fetched_name:
                return fetched_name.group(1).strip()

        return ""

    def extract_episode(self) -> int:
        # episode format exemple: E01 or E1 or 01 or 1
        episode_patterns = [
            r'S\d{1,2}E(\d{1,2})',
            r'\b(\d{1,4})\b'
        ]

        for patter in episode_patterns:
            match = re.search(patter, self.filename_clean)

            if match:
                episode = match.group(1)
                return int(episode)

        return 0

    def extract_season(self) -> int:
        # season format exemple: S01 or S1
        season_pattern = r'S(\d{1,2})E\d{1,2}'
        match = re.search(season_pattern, self.filename_clean)

        if match:
            episode = match.group(1)
            return int(episode)
        return 1


def sort():
    logging.info(f"Detected modification in {download_folder}")
    for filename in os.listdir(download_folder):
        if not filename.endswith(".part"):  # skip incomplete downloads
            episode = Episode(filename)
            move_file(episode)

def with_opencv(filename):
    # import module
    import cv2
    import datetime

    # create video capture object
    data = cv2.VideoCapture(filename)

    # count the number of frames
    frames = data.get(cv2.CAP_PROP_FRAME_COUNT)
    fps = data.get(cv2.CAP_PROP_FPS)

    # calculate duration of the video
    seconds = round(frames / fps)
    video_time = datetime.timedelta(seconds=seconds)


    return seconds

def find_or_create_series_folder(series_name, season_number):
    try:
        series_path = os.path.join(nas_root_folder, "Series", series_name)
        # Check for different season folder naming conventions
        season_folders = [f"S{str(season_number).zfill(2)}", f"S{int(season_number)}"]
        for season_folder in season_folders:
            season_path = os.path.join(series_path, season_folder)
            if os.path.exists(season_path):
                logging.info(f"Found existing season folder: {season_path}")
                return season_path
        # If no matching season folder is found, create one
        new_season_path = os.path.join(series_path, f"S{str(season_number).zfill(2)}")
        os.makedirs(new_season_path, exist_ok=True)
        logging.info(f"Created new season folder: {new_season_path}")
        return new_season_path
    except Exception as e:
        logging.error(f"Error finding or creating series folder for {series_name}: {e}")
        return nas_root_folder  # Fallback to root if there's an error

def move_file(episode: Episode):
    try:
        src = os.path.join(download_folder, episode.filename)
        if os.path.isfile(src):
            #if lenght of video is greater than 50min it's a movie
            if with_opencv(src) > 3000:
                logging.info(f"Detected movie: {episode.filename_clean}")
                dest_dir = os.path.join(nas_root_folder, "Films")
                dest = os.path.join(dest_dir, episode.filename)
                shutil.move(src, dest)
                logging.info(f"Moved: {episode.filename_clean} to {dest}")

                # Rename file to use cleaned filename and episode number
                new_filename = f"{episode.name}.{episode.extension}"
                new_dest = os.path.join(dest_dir, new_filename)
                os.rename(dest, new_dest)
                logging.info(f"Renamed: {dest} to {new_dest if new_dest else dest}")
            else:
                logging.info(f"Detected series: {episode.filename_clean}")
                dest_dir = find_or_create_series_folder(episode.name, episode.season)
                dest = os.path.join(dest_dir, episode.filename)
                shutil.move(src, dest)

                logging.info(f"Moved: {episode.filename_clean} to {dest}")

                # Rename file to use cleaned filename and episode number
                new_filename = f"{episode.name} - S{str(episode.season).zfill(2)}E{str(episode.episode).zfill(2)}.{episode.extension}"
                new_dest = os.path.join(dest_dir, new_filename)
                os.rename(dest, new_dest)
                logging.info(f"Renamed: {dest} to {new_dest if new_dest else dest}")

            # Send a Discord webhook notification
            payload = {
                "content": f"||@everyone|| Added: {episode} to the library!",
                "username": "Anime Bot"
            }
            response = requests.post(webhook_url, json=payload)
            try:
                response.raise_for_status()
            except requests.exceptions.HTTPError as e:
                logging.error(f"Failed to send Discord webhook: {e}")

        else:
            logging.warning(f"Source file is not valid: {src}")
    except Exception as e:
        logging.error(f"Error moving file {episode.filename_clean}: {e}")


if __name__ == "__main__":
    sort()
