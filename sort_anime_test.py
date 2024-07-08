import logging
import re

# Configure logging
logging.basicConfig(level=logging.INFO)

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
        file_to_clean = re.sub(r'[-_.]', ' ', file_to_clean)

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
        # episode format exemple: E01 or E1 or 01 or 1 or 0001
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


def test_get_series_name():
    test_anime = [
        "A.Sign.of.Affection.S01E01.VOSTFR.1080p.WEB.x264-TsundereRaws-Wawacity.boats.mkv",
        "My Deer Friend Nokotan S01E01 VOSTFR 1080p WEB x264 AAC -Tsundere-Raws (CR).mp4",
        "Edens.Zero.S02E01.FRENCH.1080p.WEB.x264-TsundereRaws-Wawacity.uno.mkv",
        "Dragon Ball 101 Bulma et Son Goku.mkv",
        "[Mixouille] Bleach Kai - 01 - Les gardiens de nos âmes - 720p.MULTI.x264.mkv",
        "Komi-san.wa.Komyushou.Desu.05.VOSTFR.1080p.www.vostfree.tv.mp4",
        "Kaguya-sama.wa.Kokurasetai.Ultra.Romantic.06.VOSTFR.1080p.www.vostfree.tv.mp4",
        "Chou.Kadou.Girl.⅙.Amazing.Stranger.07.VOSTFR.720p.www.vostfree.com.mp4",
        "One.Piece.1010.VOSTFR.1080p.WEB.x264-TsundereRaws-Wawacity.fit.mkv",
    ]

    for anime in test_anime:
        episode = Episode(anime)
        series_name = episode.name
        season_number = episode.season
        episode_number = episode.episode
        print(f"Filename: {series_name}")
        print(f"Season number: {season_number}")
        print(f"Episode number: {episode_number}")
        print("-" * 20)


if __name__ == "__main__":
    test_get_series_name()
