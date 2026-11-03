use super::Game;
use horizon_client::mobs::render::umbx_skins3d_cache as voxels;
use horizon_client::skins::umbx_skins3d_images as images;

pub(super) const ID: &str = "skins_3d";
const VERSION: &str = "1.0.0";
const DESCRIPTION: &str = "The outer layer of every skin as real 3D pixels: hat, jacket, sleeves and pants.";
const REACH: &str = "reach";
const REACHES: [u32; 4] = [8, 16, 32, 64];
const DEFAULT_REACH: u32 = 16;
const SKIN_EDGE: usize = 64;

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
    let downloads = game.skin_downloads.borrow();
    for location in wanted {
        if let Some(rgba) = images::image(&downloads, &location)
            && let Some((width, height)) = size(rgba.len())
        {
            voxels::offer(&location, width, height, rgba);
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
