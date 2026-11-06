# 3D Skins for Horizon

![3D Skins icon](icon.png)

A native mod for [Horizon](https://github.com/horizon-rust/horizon). The outer layer of every player's skin (hat, jacket, sleeves and pants) is drawn as real 3D pixels instead of a flat sheet, so hair, hoods, scarves and armor details on a skin stand out from the body.

## What it does

* Every painted pixel of a skin's outer layer becomes real 3D geometry that follows the head, arms and legs exactly as the game moves them: walking, sneaking, swinging, riding and in the inventory preview.
* Works for your own player and for everyone else you can see, with downloaded, launcher and default skins and with classic and slim arms.
* Touching pixels are merged into one piece, so a skin costs far fewer polygons and the game does no extra work per frame.
* Corners of the head, body, arms and legs are closed, so hats and hoods look solid from every side.
* A layer under armor goes back to its flat form, so nothing pokes through helmets, chestplates or leggings.
* Your sleeves are 3D in first person too.
* The hurt flash, Skin Customization and resource pack models work as usual.
* Up close you see the 3D layer; further away the normal flat layer takes over, so crowded places stay fast.

Only what you see changes. Nothing is sent to the server or to other players.

## Compared with Horizon's built-in option

Horizon has its own 3D Skin Layers switch under Skin Customization (off by default). This mod takes over from it while it is on.

| | 3D Skins | Built-in 3D Skin Layers |
|---|---|---|
| Polygons, average of the 18 default skins | 95 | 407 |
| Polygons, a fully painted outer layer | 180 | 2656 |
| Corners of the head and body | Closed | Open notch |
| Under armor | Flat, nothing pokes through | Stays 3D and pokes through |
| Sleeves in first person | 3D | Flat |
| Far away players | Flat beyond the distance you choose | 3D at every distance |

## Settings

Open Options, Mods, 3D Skins.

| Setting | Default | What it does |
|---|---|---|
| 3D layers within | 32 blocks | Players closer than this get the 3D layer. Click to step through 8, 16, 32 and 64 |

Turn the whole mod off on the same page.

## Install

In the Horizon launcher, open Mods and install 3D Skins, or use Add from GitHub with this repository's link. Then build your native mods in the launcher as usual.

## Build from source

You need Rust 1.95 or newer, Python 3.12 or newer, a Horizon checkout at the commit in `compatibility.json`, and the horizon-mixins tool at the revision listed there.

| Command | What it does |
|---|---|
| `cargo test` | Runs the tests of the mesh builder |
| `python tools/bundle.py apply --horizon <horizon> --mixins-tool <horizon-mixins> --out <stage> --check-target <target>` | Applies the mod to Horizon and checks that the game compiles |
| `python tools/bundle.py package` | Writes `build/skins_3d-<version>.hnmod` |
| `python tools/make_icon.py` | Redraws `icon.png` |

Made for Horizon Nightly 2026.11.05 (commit `66f3981`) with horizon-mixins `5a64af9`. Tested on Windows 11.

## License

MIT. See [LICENSE](LICENSE).
