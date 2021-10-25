use wasm_bindgen::prelude::*;
use super::util;
use super::random::RandomSource;

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct NoiseParameters {
	first_octave: i32,
	amplitudes: Vec<f64>,
}

impl NoiseParameters {
	pub fn new(first_octave: i32, amplitudes: &[f64]) -> Self {
		Self {
			first_octave,
			amplitudes: Vec::from(amplitudes)
		}
	}
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct ImprovedNoise {
	xo: f64,
	yo: f64,
	zo: f64,
	p: [u8; 256]
}

impl ImprovedNoise {
	pub fn new(random: &mut dyn RandomSource) -> Self {
		let xo = random.next_double() * 256.0;
		let yo = random.next_double() * 256.0;
		let zo = random.next_double() * 256.0;
		let mut p: [u8; 256] = [0; 256];
		for i in 0..256 {
			p[i] = i as u8
		}
		for i in 0..256 as usize {
			let j = random.next_int_max(256 - i as i32) as usize;
			let temp = p[i];
			p[i] = p[i + j];
			p[i + j] = temp;
		}
		Self { xo, yo, zo, p }
	}

	pub fn sample(&self, x: f64, y: f64, z: f64, y_scale: f64, y_limit: f64) -> f64 {
		let x2 = x + self.xo;
		let y2 = y + self.yo;
		let z2 = z + self.zo;
		let x3 = x2.floor();
		let y3 = y2.floor();
		let z3 = z2.floor();
		let x4 = x2 - x3;
		let y4 = y2 - y3;
		let z4 = z2 - z3;

		let mut y6 = 0.0;
		if y_scale != 0.0 {
			let t = if y_limit >= 0.0 && y_limit < y4 { y_limit } else { y4 };
			y6 = (t / y_scale + 1.0e-7).floor()
		}

		self.sample_and_lerp(x3 as i32, y3 as i32, z3 as i32, x4, y4 - y6, z4, y4)
	}

	fn sample_and_lerp(&self, a: i32, b: i32, c: i32, d: f64, e: f64, f: f64, g: f64) -> f64 {
		let h = self.p(a);
		let i = self.p(a + 1);
		let j = self.p(h + b);
		let k = self.p(h + b + 1);
		let l = self.p(i + b);
		let m = self.p(i + b + 1);

		let n = util::grad_dot(self.p(j + c), d, e, f);
		let o = util::grad_dot(self.p(l + c), d - 1.0, e, f);
		let p = util::grad_dot(self.p(k + c), d, e - 1.0, f);
		let q = util::grad_dot(self.p(m + c), d - 1.0, e - 1.0, f);
		let r = util::grad_dot(self.p(j + c + 1), d, e, f - 1.0);
		let s = util::grad_dot(self.p(l + c + 1), d - 1.0, e, f - 1.0);
		let t = util::grad_dot(self.p(k + c + 1), d, e - 1.0, f - 1.0);
		let u = util::grad_dot(self.p(m + c + 1), d - 1.0, e - 1.0, f - 1.0);

		let v = util::smoothstep(d);
		let w = util::smoothstep(g);
		let x = util::smoothstep(f);

		util::lerp3(v, w, x, n, o, p, q, r, s, t, u)
	}

	fn p(&self, i: i32) -> i32 {
		self.p[(i & 255) as usize] as i32
	}
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct PerlinNoise {
	levels: Vec<Option<(f64, ImprovedNoise)>>,
	lowest_freq_input: f64,
	lowest_freq_value: f64,
}

impl PerlinNoise {
	pub fn new(random: &mut dyn RandomSource, params: &NoiseParameters) -> Self {
		let n = params.amplitudes.len() as i32;
		assert!(1 - params.first_octave >= n, "Positive octaves are disabled");
		let mut levels = Vec::with_capacity(n as usize);
		for _ in 0..=-params.first_octave {
			levels.push(None);
		}
		for i in (0..=-params.first_octave).rev() {
			if i < n && params.amplitudes[i as usize] != 0.0 {
				levels[i as usize] = Some((params.amplitudes[i as usize], ImprovedNoise::new(random)));
			} else {
				random.consume(262);
			}
		}
		Self {
			levels,
			lowest_freq_input: (2 as f64).powi(params.first_octave),
			lowest_freq_value: (2 as f64).powi(n - 1) / ((2 as f64).powi(n) - 1.0),
		}
	}

	// pub fn get_octave(&self, i: i32) -> Option<ImprovedNoise> {
	// 	self.levels[self.levels.len() - 1 - i as usize].as_ref().map(|f| f.1)
	// }

	pub fn sample(&self, x: f64, y: f64, z: f64, y_scale: f64, y_limit: f64, fix_y: bool) -> f64 {
		let mut value = 0.0;
		let mut input_factor = self.lowest_freq_input;
		let mut value_factor = self.lowest_freq_value;
		for i in 0..self.levels.len() {
			if let Some((amplitude, level)) = &self.levels[i as usize] {
				let noise = level.sample(
					util::wrap(x * input_factor),
					if fix_y { -level.yo } else { util::wrap(y * input_factor) },
					util::wrap(z * input_factor),
					y_scale * input_factor,
					y_limit * input_factor,
				);
				value += amplitude * value_factor * noise;
			}
			input_factor *= 2.0;
			value_factor /= 2.0;
		}
		value
	}
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct NormalNoise {
	first: PerlinNoise,
	second: PerlinNoise,
	value_factor: f64,
}

impl NormalNoise {
	const INPUT_FACTOR: f64 = 1.0181268882175227;

	pub fn new(random: &mut dyn RandomSource, params: &NoiseParameters) -> Self {
		let first = PerlinNoise::new(random, params);
		let second = PerlinNoise::new(random, params);

		let mut min = i32::MAX;
		let mut max = i32::MIN;
		for (i, &a) in params.amplitudes.iter().enumerate() {
			if a != 0.0 {
				min = min.min(i as i32);
				max = max.max(i as i32);
			}
		}

		Self {
			first,
			second,
			value_factor: (1.0 / 6.0) / (0.1 * (1.0 + 1.0 / (max - min + 1) as f64)),
		}
	}

	pub fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
		let xx = x * Self::INPUT_FACTOR;
		let yy = y * Self::INPUT_FACTOR;
		let zz = z * Self::INPUT_FACTOR;
		let first = self.first.sample(x, y, z, 0.0, 0.0, false);
		let second = self.second.sample(xx, yy, zz, 0.0, 0.0, false);
		(first + second) * self.value_factor
	}
}
