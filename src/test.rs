use super::*;
use super::random::RandomSource;

#[test]
fn legacy_random_int() {
  let mut random = random::LegacyRandomSource::new(123);
  let expected = vec![-1188957731, 1018954901, -39088943, 1295249578, 1087885590, -1829099982, -1680189627, 1111887674, -833784125, -1621910390];
  let actual = (0..10).map(|_| random.next_int()).collect::<Vec<_>>();
  assert_eq!(actual, expected);
}

#[test]
fn legacy_random_int_max() {
  let mut random = random::LegacyRandomSource::new(123);
  assert_eq!(random.next_int_max(256), 185);
  assert_eq!(random.next_int_max(255), 200);
  assert_eq!(random.next_int_max(254), 74);
}

#[test]
fn legacy_random_float() {
  let mut random = random::LegacyRandomSource::new(123);
  let expected = vec![0.72317415, 0.23724389, 0.99089885, 0.30157375, 0.2532931, 0.57412946, 0.60880035, 0.2588815, 0.80586946, 0.6223695];
  let actual = (0..10).map(|_| random.next_float()).collect::<Vec<_>>();
  assert_eq!(actual, expected);
}

#[test]
fn legacy_random_double() {
  let mut random = random::LegacyRandomSource::new(123);
  let expected = vec![0.7231741869568761, 0.990898874798736, 0.2532930999562567, 0.6088003568750999, 0.8058694962089253, 0.8754127658344386, 0.7160484954175045, 0.0719170208985256, 0.7962609541776712, 0.5787169245060814];
  let actual = (0..10).map(|_| random.next_double()).collect::<Vec<_>>();
  assert_eq!(actual, expected);
}

#[test]
fn xoroshiro_random() {
  let mut random = random::XoroshiroRandomSource::default();
  assert_eq!(random.next_int(), 159808533);
  assert_eq!(random.next_long(), 7502368011707135260);
  assert_eq!(random.next_float(), 0.019376636);
  assert_eq!(random.next_double(), -0.03839469124758511);
}

#[test]
fn improved_noise() {
  let mut random = random::LegacyRandomSource::new(123);
  let noise = noise::ImprovedNoise::new(&mut random);
  let noise2 = noise::ImprovedNoise::new(&mut random);

  println!("{}", noise.sample(0.0, 2.0, 1.0, 0.0, 0.0));
  println!("{}", noise2.sample(0.0, 2.0, 1.0, 0.0, 0.0));
  println!("{}", noise.sample(0.0, 2.0, 1.0, 0.0, 0.0));
  println!("{}", noise2.sample(0.0, 2.0, 1.0, 0.0, 0.0));
}

#[test]
fn perlin_noise() {
  let mut random = random::LegacyRandomSource::new(123);
  let params = noise::NoiseParameters::new(-4, &[1.0, 2.0, 0.5]);
  let noise3 = noise::PerlinNoise::new(&mut random, &params);

  println!("{}", noise3.sample(0.0, 3.0, 1.2, 0.0, 0.0, false));
}

#[test]
fn normal_noise() {
  let mut random = random::LegacyRandomSource::new(123);
  let params = noise::NoiseParameters::new(-4, &[1.0, 2.0, 0.5]);
  let noise4 = noise::NormalNoise::new(&mut random, &params);
println!("{}", noise4.sample(0.0, 3.0, 1.2));
println!("{}", noise4.sample(5.4, -4.0, 0.7));
}

#[test]
fn climate() {
  let parameters = climate::ParameterList::new(&[
    (climate::parameters(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0), 2),
    (climate::parameters(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0), 5),
  ]);

  println!("{}", parameters.find(climate::target(1.0, 0.0, 0.0, 0.0, 0.0, 0.0)));
  println!("{}", parameters.find(climate::target(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)));
  println!("{}", parameters.find(climate::target(0.0, 0.0, 0.2, 0.0, 0.0, 0.0)));
  println!("{}", parameters.find(climate::target(0.0, 0.0, 0.6, 0.0, 0.0, 0.0)));
  println!("{}", parameters.find(climate::target(1.0, 0.0, 0.6, 0.0, 0.0, 0.0)));
}
