use super::umbx_voxel_build::{self as build, Image, MAX_FACES};
use super::{Atlas, Mob, MobData, Part, base_transform, part_matrix, posed_body};
use crate::mesher::Vertex;
use crate::mobs::{Polygon, render_model};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};

pub const PLAYER: &str = "minecraft:player";
pub const LAYERS: [(&str, &str); 6] = [("hat", "head"), ("jacket", "body"), ("left_sleeve", "left_arm"), ("right_sleeve", "right_arm"), ("left_pants", "left_leg"), ("right_pants", "right_leg")];
const MAX_SKINS: usize = 96;
const MAX_WANTED: usize = 64;
const REST: [f32; 9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0];

type Mesh = Arc<Vec<(&'static str, &'static str, Part)>>;

struct Skin {
    width: usize,
    height: usize,
    rgba: Vec<u8>,
}

struct Cache {
    enabled: bool,
    reach: f64,
    skins: HashMap<String, Skin>,
    meshes: HashMap<(String, bool), Option<Mesh>>,
    order: VecDeque<String>,
    wanted: HashSet<String>,
}

static CACHE: LazyLock<Mutex<Cache>> = LazyLock::new(|| {
    Mutex::new(Cache { enabled: false, reach: 16.0, skins: HashMap::new(), meshes: HashMap::new(), order: VecDeque::new(), wanted: HashSet::new() })
});

fn cache() -> MutexGuard<'static, Cache> {
    CACHE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn configure(enabled: bool, reach: f64) {
    let mut cache = cache();
    cache.enabled = enabled;
    cache.reach = reach;
}

pub fn wanted() -> Vec<String> {
    cache().wanted.drain().collect()
}

pub fn offer(location: &str, width: usize, height: usize, rgba: &[u8]) {
    if Image::new(width, height, rgba).is_none() {
        return;
    }
    let mut cache = cache();
    if cache.skins.contains_key(location) {
        return;
    }
    while cache.order.len() >= MAX_SKINS {
        if let Some(old) = cache.order.pop_front() {
            cache.skins.remove(&old);
            cache.meshes.retain(|(skin, _), _| *skin != old);
        }
    }
    cache.order.push_back(location.to_owned());
    cache.skins.insert(location.to_owned(), Skin { width, height, rgba: rgba.to_vec() });
}

fn find<'a>(part: &'a Part, name: &str) -> Option<&'a Part> {
    if part.name == name {
        return Some(part);
    }
    part.children.iter().find_map(|c| find(c, name))
}

fn mesh(model: &Part, skin: &Skin) -> Option<Mesh> {
    let image = Image::new(skin.width, skin.height, &skin.rgba)?;
    let mut parts = Vec::new();
    let mut total = 0;
    for (layer, parent) in LAYERS {
        let Some(source) = find(model, layer) else { continue };
        let quads: Vec<build::Quad> = source.polygons.iter().map(|p| (p.normal, p.vertices)).collect();
        let faces = build::voxels(&quads, &image, MAX_FACES - total);
        total += faces.len();
        if faces.is_empty() {
            continue;
        }
        let polygons = faces.into_iter().map(|(normal, vertices)| Polygon { normal, vertices, cull_back: false }).collect();
        parts.push((layer, parent, Part { name: format!("voxel_{layer}"), pose: REST, polygons, children: Vec::new() }));
    }
    (!parts.is_empty()).then(|| Arc::new(parts))
}

pub fn draw(m: &Mob, alpha: f32, camera: glam::DVec3, atlas: Atlas<'_>, light: u32, data: &MobData, out: &mut Vec<Vertex>) {
    if m.kind.id != PLAYER || super::super::trader::is_invisible(m) {
        return;
    }
    let Some(location) = m.skin.as_deref() else { return };
    let mesh = {
        let mut cache = cache();
        let offset = glam::DVec3::new(m.position.x, m.position.y, m.position.z) - camera;
        if !cache.enabled || offset.length() > cache.reach {
            return;
        }
        let key = (location.to_owned(), m.slim);
        match cache.meshes.get(&key) {
            Some(mesh) => mesh.clone(),
            None => match cache.skins.get(location) {
                Some(skin) => {
                    let (model, _, _) = posed_body(m, alpha, data);
                    let built = mesh(&model, skin);
                    cache.meshes.insert(key, built.clone());
                    built
                }
                None => {
                    if cache.wanted.len() < MAX_WANTED {
                        cache.wanted.insert(location.to_owned());
                    }
                    None
                }
            },
        }
    };
    let (Some(mesh), Some(uv)) = (mesh, atlas(location)) else { return };
    let (model, poses, _) = posed_body(m, alpha, data);
    let base = base_transform(m, alpha, camera);
    for (layer, parent, part) in mesh.iter() {
        if poses.is_hidden_named(parent) || poses.is_hidden_named(layer) || poses.skips_named(layer) {
            continue;
        }
        if let Some(matrix) = part_matrix(&model, base, &poses, layer, 0.0) {
            render_model(part, matrix, &|p| p.pose, uv, light, 0, out);
        }
    }
}
