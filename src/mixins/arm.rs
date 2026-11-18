fn arm(game: &Game<'_>, left: bool, matrix: glam::Mat4, light: u32, out: &mut Vec<Vertex>) -> ::core::ops::ControlFlow<()> {
    crate::game::umbx_skins3d_glue::arm(game, left, matrix, light, out)
}
