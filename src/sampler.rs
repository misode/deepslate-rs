use wasm_bindgen::prelude::*;
use super::random::{ LegacyRandomSource as Random };
use super::noise::{ NormalNoise, NoiseParameters };
use super::climate::{ TargetPoint };

#[wasm_bindgen]
pub struct NoiseOctaves {
	temperature: NoiseParameters,
	humidity: NoiseParameters,
	continentalness: NoiseParameters,
	erosion: NoiseParameters,
	weirdness: NoiseParameters,
	shift: NoiseParameters,
}

impl NoiseOctaves {
	pub fn new(temperature: NoiseParameters, humidity: NoiseParameters, continentalness: NoiseParameters, erosion: NoiseParameters, weirdness: NoiseParameters, shift: NoiseParameters ) -> Self {
		Self { temperature, humidity, continentalness, erosion, weirdness, shift }
	}
}

#[wasm_bindgen]
pub struct Sampler {
	temperature_noise: NormalNoise,
	humidity_noise: NormalNoise,
	continentalness_noise: NormalNoise,
	erosion_noise: NormalNoise,
	weirdness_noise: NormalNoise,
	offset_noise: NormalNoise,
}

impl Sampler {
	pub fn new(seed: i64, octaves: &NoiseOctaves) -> Self {
		Self {
			temperature_noise: NormalNoise::new(&mut Random::new(seed), &octaves.temperature),
			humidity_noise: NormalNoise::new(&mut Random::new(seed + 1), &octaves.humidity),
			continentalness_noise: NormalNoise::new(&mut Random::new(seed + 2), &octaves.continentalness),
			erosion_noise: NormalNoise::new(&mut Random::new(seed + 3), &octaves.erosion),
			weirdness_noise: NormalNoise::new(&mut Random::new(seed + 4), &octaves.weirdness),
			offset_noise: NormalNoise::new(&mut Random::new(seed + 5), &octaves.shift),
		}
	}

	pub fn target(&self, x: i64, _y: i64, z: i64) -> TargetPoint {
		let xx = x as f64 + self.offset_noise.sample(x as f64, 0.0, z as f64) * 4.0;
		let zz = z as f64 + self.offset_noise.sample(z as f64, x as f64, 0.0) * 4.0;

		TargetPoint::new(
			self.temperature_noise.sample(xx, 0.0, zz),
			self.humidity_noise.sample(xx, 0.0, zz),
			self.continentalness_noise.sample(xx, 0.0, zz),
			self.erosion_noise.sample(xx, 0.0, zz),
			self.weirdness_noise.sample(xx, 0.0, zz),
			0.0, // depth is set to 0 until terrainshaper is implemented
		)
	}
}
