#!/usr/bin/env python3

import platform
import subprocess
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
DOCS_DIR = PROJECT_ROOT / "docs"
SCRIPTS_DIR = PROJECT_ROOT / "scripts"

IS_WINDOWS = platform.system() == "Windows"


def run_shellshot(output_name, command, use_shell=False):
    output_path = DOCS_DIR / output_name

    cmd = ["cargo", "run", "--"]

    if use_shell:
        cmd.append("--shell")

    cmd += ["-o", str(output_path)]
    cmd += command

    result = subprocess.run(cmd)

    if result.returncode != 0:
        print(f"Failed generating {output_name}")
        sys.exit(result.returncode)

    print(f"{output_name} generated!")


def main():
    DOCS_DIR.mkdir(exist_ok=True)

    # Echo example
    if IS_WINDOWS:
        run_shellshot(
            "echo_example.png",
            ["echo", "Hello from ShellShot!"],
            use_shell=True,
        )
    else:
        run_shellshot(
            "echo_example.png",
            ["echo", "Hello from ShellShot!"],
        )

    # ASCII example
    ascii_script = Path("scripts") / "display_ascii.py"

    if not ascii_script.exists():
        print("scripts/display_ascii.py not found")
        sys.exit(1)

    if IS_WINDOWS:
        run_shellshot(
            "ascii_example.png",
            ["python", str(ascii_script)],
        )
    else:
        run_shellshot(
            "ascii_example.png",
            ["python", str(ascii_script)],
        )

    print("\n🎉 All documentation images regenerated successfully!")


if __name__ == "__main__":
    main()