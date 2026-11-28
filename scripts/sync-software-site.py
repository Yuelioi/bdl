"""Publish the latest stable BDL release using the configured local BDL publisher."""

import argparse
import re
import subprocess
import sys
from pathlib import Path
from urllib.parse import urlencode


ROOT = Path(__file__).resolve().parents[1]


def version_number(version):
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version):
        raise ValueError(f"Unexpected stable version: {version}")
    return tuple(map(int, version.split(".")))


def latest_release(publisher):
    release = publisher.request_json(f"https://api.github.com/repos/{publisher.REPO}/releases/latest")
    if release["draft"] or release["prerelease"] or not release["tag_name"].startswith("v"):
        raise ValueError("Expected a published stable BDL release")
    version_number(release["tag_name"][1:])
    return release


def release_identity(release):
    # GitHub increments download counters when checking routes.
    assets = sorted((item["id"], item["name"], item["digest"], item["size"]) for item in release["assets"])
    return release["id"], release["tag_name"], assets


def site_matches(publisher, slots, version, directory_url):
    matches = True
    for slot in slots:
        query = urlencode({key: slot[key] for key in ("platform", "architecture")})
        site = publisher.request_json(
            f"{publisher.DISTRIBUTION_ORIGIN}/api/v1/software/bdl/download-config?channel=stable&{query}"
        )
        current = site["release"]
        if version_number(current["version"]) > version_number(version):
            raise ValueError("The software site already has a newer release")
        if current["version"] != version:
            matches = False
            continue
        if (current["sha256"], current["sizeBytes"], current["fileName"]) != (
            slot["sha256"], slot["size"], slot["primary"]
        ):
            raise ValueError(f"Same version has different package data: {query}")
        official = slot["github_url"]
        expected = {
            ("github", official),
            ("external_link", "https://ghfast.top/" + official.removeprefix("https://")),
            ("external_link", "https://gh-proxy.com/" + official),
        }
        if slot["quark"]:
            expected.add(("cloud_drive", directory_url))
        public = {(s["kind"], s["delivery"].get("url")) for s in site["sources"]
                  if not s["delivery"]["authenticationRequired"]}
        if not expected.issubset(public) or not any(s["kind"] == "hosted" for s in site["sources"]):
            matches = False
    return matches


def sync(publisher, releases_dir, apply, expected_tag):
    release = latest_release(publisher)
    tag = release["tag_name"]
    if expected_tag and tag != expected_tag:
        raise ValueError(f"Requested {expected_tag}, but GitHub's latest published release is {tag}")
    version = tag[1:]
    slots = publisher.render_slots(version, release)
    directory = publisher.load_json(publisher.CONFIG_PATH)["quarkDirectory"]["shareUrl"]
    if site_matches(publisher, slots, version, directory):
        print(f"BDL {tag}: all five public download configurations are up to date.")
        return
    if not apply:
        print(f"BDL {tag}: software site update required (five packages, three GitHub routes).")
        return
    work = releases_dir / version
    work.mkdir(exist_ok=True)
    publisher.find_software(publisher.load_env(publisher.DEFAULT_SECRETS)["DISTRIBUTION_TOKEN"])
    metadata = work / "upstream-release.json"
    publisher.save_json(metadata, release)
    subprocess.run([sys.executable, str(ROOT / "scripts/verify-release-channels.py"), tag,
                    "--release-json", str(metadata)], check=True)
    if release_identity(latest_release(publisher)) != release_identity(release):
        raise ValueError("Latest release changed during verification")
    if not site_matches(publisher, slots, version, directory):
        # Use the same pinned release object for verification and publication.
        config = publisher.load_json(publisher.CONFIG_PATH)
        publisher.apply(version, release, slots, work, publisher.DEFAULT_SECRETS,
                        directory=config["quarkDirectory"])
    if not site_matches(publisher, slots, version, directory):
        raise ValueError("The public download configurations do not yet match all verified routes")
    print(f"BDL {tag}: published and verified all five public download configurations.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", help="require this exact deployment tag to be the latest published release")
    parser.add_argument("--apply", action="store_true", help="publish verified updates")
    parser.add_argument("--releases-dir", type=Path, default=Path("D:/coding/bdl/releases"))
    args = parser.parse_args()
    publisher_dir = ROOT / "local-release"
    if not (publisher_dir / "publish_bdl.py").is_file():
        parser.error("Use the configured release workstation with the BDL publisher installed")
    sys.path.insert(0, str(publisher_dir))
    import publish_bdl as publisher
    if not args.apply:
        sync(publisher, args.releases_dir, False, args.tag)
        return
    # Windows releases this lock even if the publishing process is interrupted.
    import msvcrt
    args.releases_dir.mkdir(parents=True, exist_ok=True)
    with (args.releases_dir / ".bdl-site-sync.lock").open("a+b") as lock:
        lock.seek(0)
        msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
        sync(publisher, args.releases_dir, True, args.tag)


if __name__ == "__main__":
    main()
