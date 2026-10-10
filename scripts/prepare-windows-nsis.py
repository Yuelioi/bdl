"""Prefetch Tauri's pinned NSIS tools with bounded retries and upstream hashes."""
import argparse
import hashlib
from pathlib import Path, PurePosixPath
import time
import urllib.request
import zipfile

BASE = "https://github.com/tauri-apps/"
# Match tauri-bundler's pins; review these and the CI cache key when upgrading the CLI.
NSIS_URL = BASE + "binary-releases/releases/download/nsis-3.11/nsis-3.11.zip"
NSIS_SHA1 = "ef7ff767e5cbd9edd22add3a32c9b8f4500bb10d"
PLUGIN_URL = BASE + "nsis-tauri-utils/releases/download/nsis_tauri_utils-v0.5.3/nsis_tauri_utils.dll"
PLUGIN_SHA1 = "75197fee3c6a814fe035788d1c34ead39349b860"
PLUGIN_PATH = Path("Plugins/x86-unicode/additional/nsis_tauri_utils.dll")


def valid(path, expected):
    return path.is_file() and hashlib.sha1(path.read_bytes()).hexdigest() == expected


def download(url, target, expected, opener=urllib.request.urlopen, sleep=time.sleep):
    target.parent.mkdir(parents=True, exist_ok=True)
    partial = target.with_name(target.name + ".download")
    for attempt in range(3):
        try:
            with opener(url, timeout=120) as response, partial.open("wb") as output:
                while chunk := response.read(65536):
                    output.write(chunk)
            if not valid(partial, expected):
                raise ValueError("Upstream checksum mismatch")
            partial.replace(target)
            return
        except Exception:
            if attempt == 2:
                raise
            print(f"NSIS dependency download failed; retry {attempt + 1}/2", flush=True)
            sleep(2 ** (attempt + 1))
        finally:
            partial.unlink(missing_ok=True)


def extract(archive, destination):
    with zipfile.ZipFile(archive) as bundle:
        for member in bundle.infolist():
            path = PurePosixPath(member.filename)
            if not path.parts or path.parts[0] != "nsis-3.11" or ".." in path.parts or "\\" in member.filename or any(":" in part for part in path.parts):
                raise ValueError("Unexpected NSIS archive path")
            target = destination.joinpath(*path.parts[1:])
            if member.is_dir():
                target.mkdir(parents=True, exist_ok=True)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(bundle.read(member))


def prepare(tools):
    nsis = tools / "NSIS"
    required = ["makensis.exe", "Bin/makensis.exe", "Stubs/lzma-x86-unicode",
                "Stubs/lzma_solid-x86-unicode", "Include/MUI2.nsh", "Include/FileFunc.nsh",
                "Include/x64.nsh", "Include/nsDialogs.nsh", "Include/WinMessages.nsh",
                "Include/Win/COM.nsh", "Include/Win/Propkey.nsh", "Include/Win/RestartManager.nsh"]
    if not all((nsis / path).is_file() for path in required):
        archive = tools / "nsis-3.11.zip"
        try:
            download(NSIS_URL, archive, NSIS_SHA1)
            extract(archive, nsis)
        finally:
            archive.unlink(missing_ok=True)
    plugin = nsis / PLUGIN_PATH
    if not valid(plugin, PLUGIN_SHA1):
        download(PLUGIN_URL, plugin, PLUGIN_SHA1)
    print(f"Verified NSIS 3.11 and Tauri plugin 0.5.3: {nsis}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--tools-dir", type=Path, required=True)
    prepare(parser.parse_args().tools_dir.resolve())
