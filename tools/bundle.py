import argparse
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile
import zipfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
BUNDLE = ROOT / "mixins" / "voxel_skins"
COMPATIBILITY = json.loads((ROOT / "compatibility.json").read_text(encoding="utf-8"))


def manifest_field(name):
    for line in (BUNDLE / "horizon-mixin.toml").read_text(encoding="utf-8").splitlines():
        if line.startswith(f"{name} = "):
            return line.split("=", 1)[1].strip().strip('"')
    raise SystemExit(f"{name} missing in horizon-mixin.toml")


def apply(horizon, mixins_tool, out, extra):
    with tempfile.TemporaryDirectory() as scratch:
        mods = pathlib.Path(scratch) / "mods"
        shutil.copytree(BUNDLE, mods / manifest_field("id"))
        for bundle in extra:
            shutil.copytree(bundle, mods / bundle.name)
        fresh = out.with_name(out.name + ".fresh")
        if fresh.exists():
            shutil.rmtree(fresh)
        subprocess.run([str(mixins_tool), "apply", "--workspace", str(horizon), "--mods", str(mods), "--out", str(fresh)], check=True)
    for source in fresh.rglob("*"):
        if source.is_file():
            target = out / source.relative_to(fresh)
            if not target.exists() or target.read_bytes() != source.read_bytes():
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(source, target)
    shutil.rmtree(fresh)


def check(out, target_dir):
    subprocess.run(["cargo", "check", "--locked", "--offline", "-p", "horizon-client",
                    "--target-dir", str(target_dir), "--message-format", "short"], cwd=out, check=True)


def package():
    mod_id, version = manifest_field("id"), manifest_field("version")
    if manifest_field("horizon_revision") != COMPATIBILITY["horizon_revision"]:
        raise SystemExit("horizon_revision differs between horizon-mixin.toml and compatibility.json")
    target = ROOT / "build" / f"{mod_id}-{version}.hnmod"
    target.parent.mkdir(parents=True, exist_ok=True)
    info = {
        "schema": 1,
        "id": mod_id,
        "version": version,
        "game": COMPATIBILITY["game"],
        "horizon": COMPATIBILITY["horizon_revision"],
        "mixins": COMPATIBILITY["horizon_mixins_revision"],
    }
    with zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED) as archive:
        archive.writestr("horizon-native.json", json.dumps(info, indent=2))
        for file in sorted(BUNDLE.rglob("*")):
            if file.is_file():
                archive.write(file, f"mixins/{mod_id}/{file.relative_to(BUNDLE).as_posix()}")
    return target


def main():
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    a = sub.add_parser("apply")
    a.add_argument("--horizon", required=True)
    a.add_argument("--mixins-tool", required=True)
    a.add_argument("--out", required=True)
    a.add_argument("--check-target")
    a.add_argument("--with", dest="extra", action="append", default=[], help="another bundle folder to apply together")
    sub.add_parser("package")
    args = parser.parse_args()
    if args.command == "apply":
        apply(pathlib.Path(args.horizon), pathlib.Path(args.mixins_tool), pathlib.Path(args.out), [pathlib.Path(e) for e in args.extra])
        if args.check_target:
            check(pathlib.Path(args.out), pathlib.Path(args.check_target))
    else:
        print(package())


if __name__ == "__main__":
    sys.exit(main())
