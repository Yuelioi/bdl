"""Verify public BDL downloads against the SHA-256 digests on a published GitHub release."""

import argparse
import concurrent.futures
import hashlib
import http.client
import json
import re
import sys
import time
import urllib.request
from pathlib import Path


CHANNELS = {
    "GitHub": "https://github.com/",
    "GHFast": "https://ghfast.top/github.com/",
    "GH-Proxy": "https://gh-proxy.com/https://github.com/",
}
HEADERS = {"User-Agent": "BDL-release-check", "Accept-Encoding": "identity"}


def retry(label, check):
    for attempt in range(3):
        try:
            return check()
        except (OSError, http.client.HTTPException, ValueError) as error:
            if attempt == 2:
                raise ValueError(f"{label}: {error}") from error
            delay = 10 * (attempt + 1)
            print(f"{label}: {error}; retrying in {delay}s", file=sys.stderr, flush=True)
            time.sleep(delay)


def verify_file(url, asset, release, collect_manifest):
    size = asset["size"]
    offset = 0
    digest = hashlib.sha256()
    manifest = bytearray()
    while offset < size:
        request = urllib.request.Request(url, headers={**HEADERS, "Range": f"bytes={offset}-"})
        with urllib.request.urlopen(request, timeout=45) as response:
            if response.headers.get_content_type() == "text/html":
                raise ValueError("received HTML instead of a release asset")
            if response.status == 206:
                match = re.fullmatch(r"bytes (\d+)-(\d+)/(\d+)", response.headers.get("Content-Range", ""))
                if not match:
                    raise ValueError("missing or invalid Content-Range")
                start, end, total = map(int, match.groups())
                if start != offset or not start <= end < size or total != size:
                    raise ValueError(f"unexpected range {start}-{end}/{total} at offset {offset}")
            elif response.status == 200:
                # A server may ignore Range and return the complete asset.
                offset, end = 0, size - 1
                digest = hashlib.sha256()
                manifest.clear()
            else:
                raise ValueError(f"unexpected HTTP status {response.status}")
            while chunk := response.read(256 * 1024):
                offset += len(chunk)
                if offset > end + 1:
                    raise ValueError("body exceeds the declared release size or byte range")
                digest.update(chunk)
                if collect_manifest:
                    manifest.extend(chunk)
            if offset != end + 1:
                raise ValueError(f"body ended at {offset}, expected {end + 1}")

    if f"sha256:{digest.hexdigest()}" != asset["digest"]:
        raise ValueError("SHA-256 differs from GitHub release metadata")
    if collect_manifest:
        document = json.loads(manifest)
        if document.get("version") != release["tag_name"][1:] or not document.get("platforms"):
            raise ValueError("manifest version or platforms do not match the release")
        known_urls = {item["browser_download_url"] for item in release["assets"]}
        for platform, item in document["platforms"].items():
            if item.get("url") not in known_urls or not item.get("signature"):
                raise ValueError(f"invalid updater target {platform}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tag")
    parser.add_argument("--repository", default="Yuelioi/bdl")
    parser.add_argument("--release-json", type=Path, help="GitHub API release metadata read by CI")
    args = parser.parse_args()
    if not re.fullmatch(r"v\d+\.\d+\.\d+", args.tag):
        parser.error("expected a stable release tag such as v0.13.2")
    if not re.fullmatch(r"[\w.-]+/[\w.-]+", args.repository):
        parser.error("expected owner/repository")

    def fetch_release():
        url = f"https://api.github.com/repos/{args.repository}/releases/tags/{args.tag}"
        with urllib.request.urlopen(urllib.request.Request(url, headers=HEADERS), timeout=30) as response:
            return json.load(response)

    if args.release_json:
        release = json.loads(args.release_json.read_text(encoding="utf-8"))
    else:
        release = retry("GitHub release metadata", fetch_release)
    if release["tag_name"] != args.tag or release["draft"] or release["prerelease"]:
        raise ValueError("expected the published stable release for the requested tag")
    version = args.tag[1:]
    names = [
        "latest.json",
        f"BDL_{version}_x64-setup.exe",
        f"BDL_{version}_aarch64.dmg",
        f"BDL_{version}_x64.dmg",
        f"BDL_{version}_amd64.AppImage",
        f"BDL-{args.tag}-android-arm64.apk",
    ]
    assets = {item["name"]: item for item in release["assets"]}
    for name in names:
        asset = assets.get(name)
        if not asset or asset["size"] <= 0 or not re.fullmatch(r"sha256:[0-9a-f]{64}", asset.get("digest") or ""):
            raise ValueError(f"{name}: missing release asset, size or SHA-256")
        if asset["browser_download_url"] != f"https://github.com/{args.repository}/releases/download/{args.tag}/{name}":
            raise ValueError(f"{name}: unexpected GitHub release URL")

    def verify_channel(channel, prefix):
        for name in names:
            url = f"{prefix}{args.repository}/releases/download/{args.tag}/{name}"
            label = f"{channel} {name}"
            retry(label, lambda: verify_file(url, assets[name], release, name == "latest.json"))
            print(f"{label}: {assets[name]['size']} bytes, SHA-256 verified", flush=True)

    passed = True
    with concurrent.futures.ThreadPoolExecutor(max_workers=len(CHANNELS)) as pool:
        jobs = {pool.submit(verify_channel, channel, prefix): channel for channel, prefix in CHANNELS.items()}
        for job in concurrent.futures.as_completed(jobs):
            try:
                job.result()
            except Exception as error:
                print(f"{jobs[job]} failed: {error}", file=sys.stderr, flush=True)
                passed = False
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
