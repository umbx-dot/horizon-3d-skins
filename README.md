# 3D Skins for Horizon

![3D Skins icon](icon.png)

A native mod for [Horizon](https://github.com/horizon-rust/horizon). The outer layer of every player's skin (hat, jacket, sleeves and pants) is drawn as real 3D pixels instead of a flat sheet, so hair, hoods, scarves and armor details on a skin stand out from the body.

## What it does

* Every opaque pixel of a skin's outer layer becomes a small block that sits on the body and follows the head, arms and legs exactly as the game moves them: walking, sneaking, swinging, riding and in the inventory preview.
* Works for your own player and for everyone else you can see, with classic and slim arms and with HD skins.
* Parts a player turned off in Skin Customization (for example the hat) stay hidden.
* Up close you see the 3D layer; further away the normal flat layer takes over, so crowded places stay fast.

Only what you see changes. Nothing is sent to the server or to other players.

## Settings

Open Options, Mods, 3D Skins.

| Setting | Default | What it does |
|---|---|---|
| 3D layers within | 16 blocks | Players closer than this get the 3D layer. Click to step through 8, 16, 32 and 64 |

Turn the whole mod off on the same page.

## Install

In the Horizon launcher, open Mods and install 3D Skins, or use Add from GitHub with this repository's link. Then build your native mods in the launcher as usual.

## Build from source

You need Rust 1.95 or newer, Python 3.12 or newer, a Horizon checkout at the commit in `compatibility.json`, and the horizon-mixins tool at the revision listed there.

| Command | What it does |
|---|---|
| `cargo test` | Runs the tests of the voxel builder |
| `python tools/bundle.py apply --horizon <horizon> --mixins-tool <horizon-mixins> --out <stage> --check-target <target>` | Applies the mod to Horizon and checks that the game compiles |
| `python tools/bundle.py package` | Writes `build/skins_3d-<version>.hnmod` |
| `python tools/make_icon.py` | Redraws `icon.png` |

Made for Horizon Nightly 2026.11.03 (commit `2944480`) with horizon-mixins `ddd9700`. Tested on Windows 11.

## License

MIT. See [LICENSE](LICENSE).
