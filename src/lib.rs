use wasm_bindgen::prelude::*;

mod random;
mod noise;
mod climate;
mod sampler;
mod util;

#[cfg(test)]
mod test;

fn iterate_grid<F, T>(f: F, x_from: f64, x_to: f64, x_step: f64, y_from: f64, y_to: f64, y_step: f64, z_from: f64, z_to: f64, z_step: f64) -> Vec<T> where F: Fn(f64, f64, f64) -> T {
  let x_count = ((x_to - x_from) / x_step).floor() as usize;
  let y_count = ((y_to - y_from) / y_step).floor() as usize;
  let z_count = ((z_to - z_from) / z_step).floor() as usize;
  let mut result = Vec::with_capacity(x_count * y_count * z_count);
  for x in 0..x_count {
    for y in 0..y_count {
      for z in 0..z_count {
        let xx = (x as f64) * x_step + x_from;
        let yy = (y as f64) * y_step + y_from;
        let zz = (z as f64) * z_step + z_from;
        result.push(f(xx, yy, zz));
      }
    }
  }
  result
}

#[wasm_bindgen]
pub fn improved_noise(seed: i64, x_from: f64, x_to: f64, x_step: f64, y_from: f64, y_to: f64, y_step: f64, z_from: f64, z_to: f64, z_step: f64) -> Vec<f64> {
  let mut random = random::LegacyRandomSource::new(seed);
  let noise = noise::ImprovedNoise::new(&mut random);
  iterate_grid(|x, y, z| noise.sample(x, y, z, 0.0, 0.0), x_from, x_to, x_step, y_from, y_to, y_step, z_from, z_to, z_step)
}

#[wasm_bindgen]
pub fn perlin_noise(seed: i64, first_octave: i32, amplitudes: &[f64], x_from: f64, x_to: f64, x_step: f64, y_from: f64, y_to: f64, y_step: f64, z_from: f64, z_to: f64, z_step: f64) -> Vec<f64> {
  let mut random = random::LegacyRandomSource::new(seed);
  let params = noise::NoiseParameters::new(first_octave, amplitudes);
  let noise = noise::PerlinNoise::new(&mut random, &params);
  iterate_grid(|x, y, z| noise.sample(x, y, z, 0.0, 0.0, false), x_from, x_to, x_step, y_from, y_to, y_step, z_from, z_to, z_step)
}

#[wasm_bindgen]
pub fn normal_noise(seed: i64, first_octave: i32, amplitudes: &[f64], x_from: f64, x_to: f64, x_step: f64, y_from: f64, y_to: f64, y_step: f64, z_from: f64, z_to: f64, z_step: f64) -> Vec<f64> {
  let mut random = random::LegacyRandomSource::new(seed);
  let params = noise::NoiseParameters::new(first_octave, amplitudes);
  let noise = noise::NormalNoise::new(&mut random, &params);
  iterate_grid(|x, y, z| noise.sample(x, y, z), x_from, x_to, x_step, y_from, y_to, y_step, z_from, z_to, z_step)
}

#[wasm_bindgen]
pub fn biome_parameters(t_min: Vec<f64>, t_max: Vec<f64>, h_min: Vec<f64>, h_max: Vec<f64>, c_min: Vec<f64>, c_max: Vec<f64>, e_min: Vec<f64>, e_max: Vec<f64>, w_min: Vec<f64>, w_max: Vec<f64>, d_min: Vec<f64>, d_max: Vec<f64>, offset: Vec<f64>, biome: Vec<i32>) -> climate::ParameterList {
  let n = t_min.len();
  let mut biomes = Vec::with_capacity(n);
  for i in 0..n {
    biomes.push((climate::ParamPoint::new(
      climate::Param::span(t_min[i], t_max[i]),
      climate::Param::span(h_min[i], h_max[i]),
      climate::Param::span(c_min[i], c_max[i]),
      climate::Param::span(e_min[i], e_max[i]),
      climate::Param::span(w_min[i], w_max[i]),
      climate::Param::span(d_min[i], d_max[i]),
      offset[i],
    ), biome[i]));
  }
  climate::ParameterList::new(biomes.as_slice())
}

#[wasm_bindgen]
pub fn noise_parameters(first_octave: i32, amplitudes: Vec<f64>) -> noise::NoiseParameters {
  noise::NoiseParameters::new(first_octave, amplitudes.as_slice())
}

#[wasm_bindgen]
pub fn climate_sampler(seed: i64, t_first: i32, t_amplitudes: Vec<f64>, h_first: i32, h_amplitudes: Vec<f64>, c_first: i32, c_amplitudes: Vec<f64>, e_first: i32, e_amplitudes: Vec<f64>, w_first: i32, w_amplitudes: Vec<f64>, s_first: i32, s_amplitudes: Vec<f64>) -> sampler::Sampler {
  let octaves = sampler::NoiseOctaves::new(
    noise::NoiseParameters::new(t_first, t_amplitudes.as_slice()),
    noise::NoiseParameters::new(h_first, h_amplitudes.as_slice()),
    noise::NoiseParameters::new(c_first, c_amplitudes.as_slice()),
    noise::NoiseParameters::new(e_first, e_amplitudes.as_slice()),
    noise::NoiseParameters::new(w_first, w_amplitudes.as_slice()),
    noise::NoiseParameters::new(s_first, s_amplitudes.as_slice()),
  );
  sampler::Sampler::new(seed, &octaves)
}

#[wasm_bindgen]
pub fn find_biome(parameters: &climate::ParameterList, target: Vec<f64>) -> i32 {
  assert_eq!(target.len(), 6);
  parameters.find(climate::target(target[0], target[1], target[2], target[3], target[4], target[5]))
}

#[wasm_bindgen]
pub fn multi_noise(parameters: &climate::ParameterList, sampler: &sampler::Sampler, x_from: f64, x_to: f64, x_step: f64, y_from: f64, y_to: f64, y_step: f64, z_from: f64, z_to: f64, z_step: f64) -> Vec<i32> {
  iterate_grid(|x, y, z| {
    let target = sampler.target(x as i64, y as i64, z as i64);
    parameters.find(target)
  }, x_from, x_to, x_step, y_from, y_to, y_step, z_from, z_to, z_step)
}

#[wasm_bindgen]
pub fn climate_noise(sampler: &sampler::Sampler, x_from: f64, x_to: f64, x_step: f64, y_from: f64, y_to: f64, y_step: f64, z_from: f64, z_to: f64, z_step: f64) -> Vec<f64> {
  iterate_grid(|x, y, z| {
    sampler.target(x as i64, y as i64, z as i64).vec()
  }, x_from, x_to, x_step, y_from, y_to, y_step, z_from, z_to, z_step)
    .into_iter().flatten().collect::<Vec<_>>()
}

#[wasm_bindgen]
pub struct Test {
  x: i32,
  y: f64,
}

#[wasm_bindgen]
pub fn create_test(x: i32) -> Test {
  Test { x, y: x as f64 / 10.0 }
}

#[wasm_bindgen]
pub fn calc_test(test: &Test) -> f64 {
  test.x as f64 + test.y
}
