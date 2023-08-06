# /usr/bin/env python

"""
Utility script for building consistent shared libraries for vpuppr.
"""

import os
from os import path
import subprocess
from argparse import ArgumentParser
from typing import Union, List
from enum import Enum
import platform


# NOTE: this is technically the wrong way to name shared libs on Windows but whatever
LIB_NAME_FORMAT: str = "libvpuppr.{}"


def get_output_dir() -> Union[str, None]:
    """
    The output path for `cargo build`.
    """
    script_dir: str = path.dirname(__file__)
    output_dir: str = path.join(script_dir, "target")

    return output_dir if path.isdir(output_dir) else None


class BuildFlag(Enum):
    DEBUG = 0
    RELEASE = 1


def build(flag: BuildFlag) -> None:
    command: List[str] = ["cargo", "build"]

    if flag == BuildFlag.RELEASE:
        command.append("--release")

    ret = subprocess.run(command, text=True)

    if ret.returncode != 0:
        raise Exception("Failed to build: {}".format(ret.stderr))

    output_dir = get_output_dir()
    if not output_dir:
        raise Exception("Unable to get output directory")

    if flag == BuildFlag.DEBUG:
        output_dir = "{}/debug".format(output_dir)
    elif flag == BuildFlag.RELEASE:
        output_dir = "{}/release".format(output_dir)
    else:
        raise Exception("Bad build flag: {}".format(flag))

    if not path.isdir(output_dir):
        raise Exception(
            "{} does not exist, the build probably failed".format(output_dir))

    # Sorry MacOS, you will have to build manually since automating builds
    # on MacOS is a huge pain
    lib_ending: str = "dll" if platform.system() == "Windows" else "so"
    rename_count: int = 0

    for file in os.listdir(os.fsencode(output_dir)):
        file_name: str = os.fsdecode(file)

        if file_name.lower().endswith(lib_ending):
            rename_count += 1
            os.rename("{}/{}".format(output_dir, file_name),
                      "{}/{}".format(output_dir, LIB_NAME_FORMAT.format(lib_ending)))

    if rename_count != 1:
        # TODO just a warning for now
        print("Renamed an unexpected amount of files: {}".format(
            rename_count), flush=True)


def main() -> None:
    # Make sure we are building in the correct directory
    os.chdir(path.dirname(__file__))

    parser = ArgumentParser("libvpuppr-build")
    parser.add_argument("--debug", action="store_true",
                        help="Build the lib in debug mode")
    parser.add_argument("--release", action="store_true",
                        help="Build the lib in release mode")

    args = parser.parse_args()

    changed = False

    # NOTE: I know it's possible to configure argparse to do this automatically, and I don't care
    if args.debug:
        changed = True
        build(BuildFlag.DEBUG)
    if args.release:
        changed = True
        build(BuildFlag.RELEASE)

    if not changed:
        raise Exception("An option must be selected")


if __name__ == "__main__":
    main()
