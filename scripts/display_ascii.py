#!/usr/bin/env python3

WIDTH = 79
HEIGHT = 6
RESET = "\033[0m"

# Text styles
styles = {
    "normal": 0,
    "bold": 1,
    "dim": 2,
    "italic": 3,
    "underline": 4,
    "reverse": 7,
    "strikethrough": 9
}

# ASCII art ShellShot
ascii_art = [
r"   ______ _           _ _   ______ _                ",
r"  / _____) |         | | | / _____) |           _   ",
r" ( (____ | |__  _____| | |( (____ | |__   ___ _| |_ ",
r"  \____ \|  _ \| ___ | | | \____ \|  _ \ / _ (_   _)",
r"  _____) ) | | | ____| | | _____) ) | | | |_| || |_ ",
r" (______/|_| |_|_____)\_)_|______/|_| |_|\___/  \__)",
r"                                                    "
]

def rainbow_color(i, total):
    hue = i / total
    r = int(255 * max(min(abs(hue*6 - 3) - 1, 1), 0))
    g = int(255 * max(min(2 - abs(hue*6 - 2), 1), 0))
    b = int(255 * max(min(2 - abs(hue*6 - 4), 1), 0))
    return r, g, b

def display_ascii(ascii_lines):
    for i, line in enumerate(ascii_lines):
        r, g, b = rainbow_color(i, len(ascii_lines))
        print(f"\033[1;38;2;{r};{g};{b}m{line}{RESET}")

def rainbow_text(text):
    for i, char in enumerate(text):
        r, g, b = rainbow_color(i, len(text))
        print(f"\033[1;38;2;{r};{g};{b}m{char}{RESET}", end="")
    print("")

def display_styles():
    print("\nAvailable Text Styles:")
    for name, code in styles.items():
        print(f"• {name.capitalize():15}: \033[{code}mThis is an example{RESET}")

def mixed_styles_demo():
    print("\nMixed Styles Showcase:\n")
    demos = [
        {"style": [1, 4], "fg": "38;2;255;100;100", "bg": None, "text": "Bold + Underline + Soft Red"},
        {"style": [3], "fg": "38;2;100;255;255", "bg": "48;2;255;255;0", "text": "Italic Cyan on Yellow BG"},
        {"style": [7], "fg": "38;2;100;255;100", "bg": None, "text": "Reverse Green Effect"},
        {"style": [9], "fg": "38;2;255;0;255", "bg": None, "text": "Strikethrough Magenta"},
        {"style": [1], "fg": "38;2;255;165;0", "bg": None, "text": "TrueColor Orange Text"},
        {"style": [1], "fg": "38;2;255;255;255", "bg": "48;2;0;120;255", "text": "Bold on Custom Blue BG"},
        {"style": [1,3,4], "fg": "38;2;255;0;255", "bg": "48;2;0;0;0", "text": "EXTREME MIX STYLE DEMO"}
    ]
    for combo in demos:
        style_code = ";".join(str(s) for s in combo["style"])
        bg_code = f";{combo['bg']}" if combo["bg"] else ""
        print(f"\033[{style_code};{combo['fg']}{bg_code}m{combo['text']}{RESET}")

def rainbow_line(width):
    print("\nRainbow gradient:\n")
    for i in range(width):
        r, g, b = rainbow_color(i, width)
        print(f"\033[48;2;{r};{g};{b}m \033[0m", end="")
    print("")

if __name__ == "__main__":
    display_ascii(ascii_art)
    rainbow_text("ShellShot")
    display_styles()
    mixed_styles_demo()
    rainbow_line(WIDTH)
