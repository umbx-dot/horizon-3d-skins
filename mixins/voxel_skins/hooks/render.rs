fn voxel_render(m: &Mob, alpha: f32, camera: glam::DVec3, atlas: Atlas<'_>, light: u32, data: &MobData, out: &mut Vec<Vertex>) {
    crate::mobs::render::umbx_voxel_cache::draw(m, alpha, camera, atlas, light, data, out);
}
