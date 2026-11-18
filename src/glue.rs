use super::Game;
use super::umbx_skins3d_cache as voxels;
use horizon_client::mesher::Vertex;
use horizon_client::mobs::{Mob, MobType, Part, gear, render_model};
use std::collections::HashSet;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};

const SKIN_EDGE: usize = 64;
const MAX_MISSING: usize = 512;

const OFFLINE_SKIN: &str = "horizon/offline_skin";
const ELYTRA: &str = "minecraft:elytra";

static MISSING: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));
static LOGGED: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

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

fn shown(head: bool, chest: Option<&str>, legs: bool) -> u8 {
    let mut shown = voxels::ALL;
    if head {
        shown &= !voxels::HAT;
    }
    if chest.is_some_and(|item| item != ELYTRA) {
        shown &= !voxels::JACKET_AND_SLEEVES;
    }
    if legs {
        shown &= !voxels::PANTS;
    }
    shown
}

fn swap(m: &mut Mob, near: bool, shown: u8) {
    let Some(location) = m.skin.as_deref() else { return };
    let model = voxels::model(&m.kind, location, m.slim, if near { shown } else { 0 });
    let same = match (&model, &m.skin_3d) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    };
    if !same {
        m.skin_3d = model;
    }
}

pub(super) fn model(kind: &Arc<MobType>, skin: &str, slim: bool) -> ControlFlow<Option<Arc<Part>>> {
    ControlFlow::Break(voxels::model(kind, skin, slim, voxels::ALL))
}

pub(super) fn arm(game: &Game<'_>, left: bool, matrix: glam::Mat4, light: u32, out: &mut Vec<Vertex>) -> ControlFlow<()> {
    let player = game.player_model.as_ref();
    let name = if left { "left_arm" } else { "right_arm" };
    let part = player.and_then(|m| m.skin_3d.as_ref()).and_then(|model| model.children.iter().find(|c| c.name == name));
    let sprite = player.and_then(|m| m.skin.as_deref()).and_then(|skin| game.gui.atlas.sprites.get(skin));
    let (Some(part), Some(sprite)) = (part, sprite) else {
        return ControlFlow::Continue(());
    };
    let z_rot = if left { -0.1 } else { 0.1 };
    let pose = |p: &Part| {
        let mut pose = p.pose;
        pose[3] = 0.0;
        pose[4] = 0.0;
        pose[5] = if p.name == name { z_rot } else { 0.0 };
        pose
    };
    render_model(part, matrix, &pose, [sprite.u0, sprite.v0, sprite.u1, sprite.v1], light, 0, out);
    ControlFlow::Break(())
}

pub(super) fn frame(game: &mut Game<'_>) {
    let reach = f64::from(mod_skins_3d::reach());
    let camera = game.camera().map(|c| c.position);
    let near = |x: f64, y: f64, z: f64| camera.is_none_or(|c| (x - c[0]).powi(2) + (y - c[1]).powi(2) + (z - c[2]).powi(2) <= reach * reach);
    let [head, chest, legs, _] = game.armor.each_ref().map(|s| s.as_ref().map(|s| &*s.item));
    let own = shown(head.is_some(), chest, legs.is_some());
    if let Some(m) = game.player_model.as_mut() {
        let close = near(m.position.x, m.position.y, m.position.z);
        swap(m, close, own);
    }
    for m in game.mobs.iter_mut().filter(|m| &*m.kind.id == voxels::PLAYER) {
        let equipment = &m.extra.equipment;
        let worn = shown(equipment.get(gear::HEAD).is_some(), equipment.get(gear::CHEST).map(|s| &*s.item), equipment.get(gear::LEGS).is_some());
        let close = near(m.position.x, m.position.y, m.position.z);
        swap(m, close, worn);
    }
    let wanted = voxels::wanted();
    if wanted.is_empty() {
        return;
    }
    let mut missing = MISSING.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut logged = LOGGED.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let downloads = game.skin_downloads.borrow();
    for location in wanted {
        if let Some(rgba) = downloads.image(&location)
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
