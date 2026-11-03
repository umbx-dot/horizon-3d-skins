use super::Game;
use horizon_client::mobs::render::umbx_skins3d_cache as voxels;
use horizon_client::skins::umbx_skins3d_images as images;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

pub(super) const ID: &str = "skins_3d";
const VERSION: &str = "1.0.1";
const DESCRIPTION: &str = "The outer layer of every skin as real 3D pixels: hat, jacket, sleeves and pants.";
const REACH: &str = "reach";
const REACHES: [u32; 4] = [8, 16, 32, 64];
const DEFAULT_REACH: u32 = 16;
const SKIN_EDGE: usize = 64;
const MAX_MISSING: usize = 512;

const OFFLINE_SKIN: &str = "horizon/offline_skin";

static MISSING: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));
static LOGGED: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

fn reach(game: &Game<'_>) -> u32 {
    game.native_setting(ID, REACH).and_then(|v| v.parse().ok()).filter(|r| REACHES.contains(r)).unwrap_or(DEFAULT_REACH)
}

fn size(bytes: usize) -> Option<(usize, usize)> {
    let pixels = bytes / 4;
    let side = (pixels as f64).sqrt() as usize;
    if bytes % 4 != 0 || pixels == 0 {
        None
    } else if side * side == pixels && side % SKIN_EDGE == 0 {
        Some((side, side))
    } else if pixels == SKIN_EDGE * SKIN_EDGE / 2 {
        Some((SKIN_EDGE, SKIN_EDGE / 2))
    } else {
        None
    }
}

fn texture_file(resources: &Path, location: &str) -> Option<PathBuf> {
    let safe = !location.is_empty()
        && !location.contains("..")
        && !location.starts_with('/')
        && location.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_/.-".contains(&b));
    safe.then(|| resources.join("assets").join("minecraft").join("textures").join(format!("{location}.png")))
}

fn bundled(game: &Game<'_>, location: &str) -> Option<(usize, usize, Vec<u8>)> {
    let file = texture_file(&game.options.resources, location)?;
    let (width, height, rgba) = horizon_client::hud::decode(&file).ok()?;
    Some((width as usize, height as usize, rgba))
}

pub(super) fn frame(game: &mut Game<'_>) {
    let on = game.native_on(ID);
    voxels::configure(on, f64::from(reach(game)));
    if !on {
        return;
    }
    let wanted = voxels::wanted();
    if wanted.is_empty() {
        return;
    }
    let mut missing = MISSING.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut logged = LOGGED.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let downloads = game.skin_downloads.borrow();
    for location in wanted {
        if let Some(rgba) = images::image(&downloads, &location)
            && let Some((width, height)) = size(rgba.len())
        {
            voxels::offer(&location, width, height, rgba);
            continue;
        }
        if missing.contains(&location) {
            continue;
        }
        if location == OFFLINE_SKIN
            && let Some((rgba, _)) = horizon_client::skins::offline_skin()
            && let Some((width, height)) = size(rgba.len())
        {
            voxels::offer(&location, width, height, rgba);
            continue;
        }
        if location.starts_with("skins/") {
            if logged.len() < MAX_MISSING && logged.insert(location.clone()) {
                log::info!("3D Skins: waiting for the pixels of downloaded skin {location}");
            }
            continue;
        }
        match bundled(game, &location) {
            Some((width, height, rgba)) if size(rgba.len()) == Some((width, height)) => voxels::offer(&location, width, height, &rgba),
            _ => {
                log::info!("3D Skins: no pixels for skin {location}; it keeps its flat layer");
                if missing.len() < MAX_MISSING {
                    missing.insert(location);
                }
            }
        }
    }
}

pub(super) fn listed(mods: &mut Vec<horizon_client::mod_api::NativeMod>) {
    mods.push(horizon_client::mod_api::NativeMod {
        id: ID.into(),
        title: format!("3D Skins {VERSION}"),
        description: DESCRIPTION.into(),
        keys: Vec::new(),
    });
}

pub(super) fn page(game: &Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id == ID {
        buttons.push(format!("3D layers within: {} blocks", reach(game)));
    }
}

pub(super) fn click(game: &mut Game<'_>, mod_id: &str, button: usize) {
    if mod_id != ID || button != 0 {
        return;
    }
    let now = reach(game);
    let next = REACHES[(REACHES.iter().position(|r| *r == now).unwrap_or(0) + 1) % REACHES.len()];
    game.native_set_setting(ID, REACH, &next.to_string());
}
