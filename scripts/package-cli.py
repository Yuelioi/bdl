"""Package an already-built CLI; does not publish or modify versions."""
import argparse
import hashlib
from pathlib import Path
import tarfile
import tomllib
import zipfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--target", choices=["windows-x64", "linux-x64", "macos-arm64", "macos-x64"], required=True)
    parser.add_argument("--output", type=Path, default=Path("output/cli"))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    version = tomllib.loads((root / "crates/bdl-cli/Cargo.toml").read_text(encoding="utf-8"))["package"]["version"]
    if not args.binary.is_file() or args.binary.stat().st_size == 0:
        parser.error("--binary must point to a non-empty built CLI executable")
    windows = args.target.startswith("windows")
    files = {
        "bdl.exe" if windows else "bdl": args.binary,
        "CLI.md": root / "docs/CLI.md",
        "LICENSE": root / "LICENSE",
        "LICENSE-bpi-rs": root / "vendor/bpi-rs/LICENSE",
    }
    args.output.mkdir(parents=True, exist_ok=True)
    archive = args.output / f"bdl-cli-{version}-{args.target}.{'zip' if windows else 'tar.gz'}"
    if windows:
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as bundle:
            for name, path in files.items():
                bundle.write(path, name)
    else:
        with tarfile.open(archive, "w:gz") as bundle:
            for name, path in files.items():
                info = bundle.gettarinfo(str(path), arcname=name)
                info.mode = 0o755 if name == "bdl" else 0o644
                with path.open("rb") as contents:
                    bundle.addfile(info, contents)
    checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_name(archive.name + ".sha256").write_text(f"{checksum}  {archive.name}\n", encoding="utf-8")
    print(archive)


if __name__ == "__main__":
    main()
