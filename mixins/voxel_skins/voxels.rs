pub type Quad = ([f32; 3], [[f32; 5]; 4]);

pub const INNER: f32 = -0.2;
pub const OUTER: f32 = 0.55;
pub const OPAQUE: u8 = 128;
pub const MAX_FACES: usize = 24_000;

pub struct Image<'a> {
    pub width: usize,
    pub height: usize,
    pub rgba: &'a [u8],
}

impl Image<'_> {
    pub fn new(width: usize, height: usize, rgba: &[u8]) -> Option<Image<'_>> {
        (width > 0 && height > 0 && rgba.len() == width * height * 4).then_some(Image { width, height, rgba })
    }

    fn solid(&self, x: i64, y: i64) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height && self.rgba[(y as usize * self.width + x as usize) * 4 + 3] >= OPAQUE
    }
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + b[i])
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}

fn scale(a: [f32; 3], k: f32) -> [f32; 3] {
    a.map(|v| v * k)
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalized(a: [f32; 3]) -> [f32; 3] {
    let length = dot(a, a).sqrt();
    if length > 0.0 { scale(a, 1.0 / length) } else { a }
}

fn quad(points: [[f32; 3]; 4], normal: [f32; 3], winding: f32, uv: [f32; 2]) -> Quad {
    let facing = dot(cross(sub(points[1], points[0]), sub(points[2], points[0])), normal);
    let mut points = points;
    if facing * winding < 0.0 {
        points.reverse();
    }
    (normal, points.map(|p| [p[0], p[1], p[2], uv[0], uv[1]]))
}

pub fn voxels(polygons: &[Quad], image: &Image, limit: usize) -> Vec<Quad> {
    let (w, h) = (image.width as f32, image.height as f32);
    let mut out = Vec::new();
    for (normal, v) in polygons {
        let position = |i: usize| [v[i][0], v[i][1], v[i][2]];
        let (p0, p1, p2) = (position(0), position(1), position(2));
        let (ua, ub, va, vb) = (v[1][3] * w, v[0][3] * w, v[1][4] * h, v[2][4] * h);
        if (ub - ua).abs() < 0.5 || (vb - va).abs() < 0.5 {
            continue;
        }
        let n = normalized(*normal);
        let winding = dot(cross(sub(p1, p0), sub(p2, p0)), n).signum();
        if winding == 0.0 {
            continue;
        }
        let at = |tu: f32, tv: f32| add(p1, add(scale(sub(p0, p1), (tu - ua) / (ub - ua)), scale(sub(p2, p1), (tv - va) / (vb - va))));
        let (x0, x1) = (ua.min(ub).round() as i64, ua.max(ub).round() as i64);
        let (y0, y1) = (va.min(vb).round() as i64, va.max(vb).round() as i64);
        let inside = |x: i64, y: i64| (x0..x1).contains(&x) && (y0..y1).contains(&y);
        for ty in y0..y1 {
            for tx in x0..x1 {
                if !image.solid(tx, ty) {
                    continue;
                }
                let uv = [(tx as f32 + 0.5) / w, (ty as f32 + 0.5) / h];
                let (fx, fy) = (tx as f32, ty as f32);
                let cell = [at(fx, fy), at(fx + 1.0, fy), at(fx + 1.0, fy + 1.0), at(fx, fy + 1.0)];
                let outer = cell.map(|c| add(c, scale(n, OUTER)));
                let inner = cell.map(|c| add(c, scale(n, INNER)));
                let center = scale(add(add(cell[0], cell[1]), add(cell[2], cell[3])), 0.25);
                out.push(quad(outer, n, winding, uv));
                for (edge, (nx, ny)) in [(tx, ty - 1), (tx + 1, ty), (tx, ty + 1), (tx - 1, ty)].into_iter().enumerate() {
                    if inside(nx, ny) && image.solid(nx, ny) {
                        continue;
                    }
                    let (a, b) = (edge, (edge + 1) % 4);
                    let side = normalized(sub(scale(add(cell[a], cell[b]), 0.5), center));
                    out.push(quad([outer[a], outer[b], inner[b], inner[a]], side, winding, uv));
                }
                if out.len() > limit {
                    return Vec::new();
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRONT: Quad = ([0.0, 0.0, -1.0], [[4.0, -8.0, -4.0, 0.25, 0.125], [-4.0, -8.0, -4.0, 0.125, 0.125], [-4.0, 0.0, -4.0, 0.125, 0.25], [4.0, 0.0, -4.0, 0.25, 0.25]]);

    fn skin(opaque: &[(usize, usize)]) -> Vec<u8> {
        let mut rgba = vec![0; 64 * 64 * 4];
        for (x, y) in opaque {
            rgba[(y * 64 + x) * 4..][..4].copy_from_slice(&[200, 100, 50, 255]);
        }
        rgba
    }

    fn faces(opaque: &[(usize, usize)]) -> Vec<Quad> {
        let rgba = skin(opaque);
        voxels(&[FRONT], &Image::new(64, 64, &rgba).unwrap(), MAX_FACES)
    }

    #[test]
    fn a_lone_pixel_becomes_a_closed_voxel() {
        let out = faces(&[(12, 12)]);
        assert_eq!(out.len(), 5);
        let (normal, outer) = out[0];
        assert_eq!(normal, [0.0, 0.0, -1.0]);
        assert!(outer.iter().all(|v| (v[2] - (-4.0 - OUTER)).abs() < 1e-5));
        assert!(outer.iter().all(|v| (v[3] - 12.5 / 64.0).abs() < 1e-6 && (v[4] - 12.5 / 64.0).abs() < 1e-6));
        let xs: Vec<f32> = outer.iter().map(|v| v[0]).collect();
        let span = xs.iter().cloned().fold(f32::MIN, f32::max) - xs.iter().cloned().fold(f32::MAX, f32::min);
        assert!((span - 1.0).abs() < 1e-5);
    }

    #[test]
    fn touching_pixels_share_no_inner_walls() {
        assert_eq!(faces(&[(12, 12), (13, 12)]).len(), 2 + 6);
        assert!(faces(&[]).is_empty());
        assert!(faces(&[(2, 2)]).is_empty());
    }

    #[test]
    fn faces_keep_the_source_winding_and_point_out() {
        let source = dot(cross(sub([-4.0, -8.0, -4.0], [4.0, -8.0, -4.0]), sub([-4.0, 0.0, -4.0], [4.0, -8.0, -4.0])), [0.0, 0.0, -1.0]).signum();
        for (normal, v) in faces(&[(12, 12)]) {
            let p = |i: usize| [v[i][0], v[i][1], v[i][2]];
            assert_eq!(dot(cross(sub(p(1), p(0)), sub(p(2), p(0))), normal).signum(), source);
        }
        let sides: Vec<[f32; 3]> = faces(&[(12, 12)]).iter().skip(1).map(|f| f.0).collect();
        assert!(sides.iter().all(|n| n[2] == 0.0 && (dot(*n, *n) - 1.0).abs() < 1e-5));
    }

    #[test]
    fn oversized_or_broken_input_is_refused() {
        let rgba = skin(&[(10, 10), (12, 12), (14, 14)]);
        assert!(voxels(&[FRONT], &Image::new(64, 64, &rgba).unwrap(), 4).is_empty());
        assert!(Image::new(64, 64, &rgba[..100]).is_none());
        assert!(Image::new(0, 64, &[]).is_none());
        let hd = vec![255u8; 128 * 128 * 4];
        let out = voxels(&[FRONT], &Image::new(128, 128, &hd).unwrap(), MAX_FACES);
        assert_eq!(out.len(), 16 * 16 + 4 * 16);
    }
}
