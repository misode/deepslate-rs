
pub fn lerp(a: f64, b: f64, c: f64) -> f64 {
	b + a * (c - b)
}

pub fn lerp2(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> f64 {
	lerp(b, lerp(a, c, d), lerp(a, e, f))
}

pub fn lerp3(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64, g: f64, h: f64, i: f64, j: f64, k: f64) -> f64 {
	lerp(c, lerp2(a, b, d, e, f, g), lerp2(a, b, h, i, j, k))
}

pub fn smoothstep(x: f64) -> f64 {
	x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

const GRADIENT: [(f64, f64, f64); 16] = [(1.0, 1.0, 0.0), (-1.0, 1.0, 0.0), (1.0, -1.0, 0.0), (-1.0, -1.0, 0.0), (1.0, 0.0, 1.0), (-1.0, 0.0, 1.0), (1.0, 0.0, -1.0), (-1.0, 0.0, -1.0), (0.0, 1.0, 1.0), (0.0, -1.0, 1.0), (0.0, 1.0, -1.0), (0.0, -1.0, -1.0), (1.0, 1.0, 0.0), (0.0, -1.0, 1.0), (-1.0, 1.0, 0.0), (0.0, -1.0, -1.0)];

pub fn grad_dot(a: i32, b: f64, c: f64, d: f64) -> f64 {
	let grad = GRADIENT[(a & 15) as usize];
	return grad.0 * b + grad.1 * c + grad.2 * d
}

pub fn wrap(value: f64) -> f64 {
	value - (value / 3.3554432e7 + 0.5).floor() * 3.3554432e7
}
