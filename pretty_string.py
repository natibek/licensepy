import enum

class Style(str, enum.Enum):
    """Container for string formatting console codes."""

    BLACK = "\033[30m"
    RED = "\033[31m"
    GREEN = "\033[32m"
    YELLOW = "\033[33m"
    BLUE = "\033[34m"
    MAGENTA = "\033[35m"
    CYAN = "\033[36m"
    WHITE = "\033[37m"
    BOLD = "\033[1m"
    UNDERLINE = "\033[4m"
    RESET = "\033[0m"


def styled(text: str, style_code: str) -> str:
    return style_code + text + Style.RESET


def warning(text: str) -> str:
    return styled(text, Style.BOLD + Style.YELLOW)


def failure(text: str) -> str:
    return styled(text, Style.BOLD + Style.RED)


def success(text: str) -> str:
    return styled(text, Style.BOLD + Style.GREEN)