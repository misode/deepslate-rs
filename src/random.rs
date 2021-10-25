pub trait RandomSource {
	fn set_seed(&mut self, seed: i64) -> ();
	fn consume(&mut self, n: i32) -> ();
	fn next_int(&mut self) -> i32;
	fn next_int_max(&mut self, max: i32) -> i32;
	fn next_long(&mut self) -> i64;
	fn next_float(&mut self) -> f32;
	fn next_double(&mut self) -> f64;
}

pub struct LegacyRandomSource {
	seed: i64,
}

impl LegacyRandomSource {
	const MODULUS_BITS: i32 = 48;
	const MODULUS_MASK: i64 = 0xFFFFFFFFFFFF;
	const MULTIPLIER: i64 = 25214903917;
	const INCREMENT: i64 = 11;
	const FLOAT_MULTIPLIER: f32 = 5.9604645E-8;
	const DOUBLE_MULTIPLIER: f64 = 1.110223E-16;

	#[allow(dead_code)]
	pub fn new(seed: i64) -> Self {
		Self {
			seed: Self::initial_seed(seed)
		}
	}

	fn initial_seed(seed: i64) -> i64 {
		(seed ^ 0x5DEECE66D) & Self::MODULUS_MASK
	}

	fn next(&mut self, n: i32) -> i32 {
		self.seed = self.seed.wrapping_mul(Self::MULTIPLIER).wrapping_add(Self::INCREMENT) & Self::MODULUS_MASK;
		(self.seed >> Self::MODULUS_BITS - n) as i32
	}
}

impl RandomSource for LegacyRandomSource {
	fn set_seed(&mut self, seed: i64) {
		self.seed = Self::initial_seed(seed)
	}

	fn consume(&mut self, n: i32) {
		for _ in 0..n {
			self.next_int();
		}
	}

	fn next_int(&mut self) -> i32 {
		self.next(32)
	}

	fn next_int_max(&mut self, max: i32) -> i32 {
		assert!(max > 0);
		if (max & max - 1) == 0 {
			return (max as i64 * self.next(31) as i64 >> 31) as i32;
		}
		loop {
			let a = self.next(31);
			let b = a % max;
			if a - b + max - 1 >= 0 {
				return b;
			};
		}
	}

	fn next_long(&mut self) -> i64 {
		let lo = self.next(32) as i64;
		let hi = self.next(32) as i64;
		(lo << 32) + hi
	}

	fn next_float(&mut self) -> f32 {
		self.next(24) as f32 * Self::FLOAT_MULTIPLIER
	}

	fn next_double(&mut self) -> f64 {
		let lo = self.next(26) as i64;
		let hi = self.next(27) as i64;
		((lo << 27) + hi) as f64 * Self::DOUBLE_MULTIPLIER
	}
}

pub struct XoroshiroRandomSource {
	lo: i64,
	hi: i64,
}

impl XoroshiroRandomSource {
	const FLOAT_UNIT: f32 = 5.9604645E-8;
	const DOUBLE_UNIT: f64 = 1.110223E-16;
	pub fn new(lo: i64, hi: i64) -> Self {
		Self { lo, hi }
	}

	pub fn from(seed: i64) -> Self {
		Self {
			lo: Self::mix_stafford_13(seed ^ 0x6A09E667F3BCC909),
			hi: Self::mix_stafford_13(seed - 7046029254386353131),
		}
	}

	fn mix_stafford_13(mut a: i64) -> i64 {
		a = (a ^ a >> 30).wrapping_mul(-4658895280553007687);
		a = (a ^ a >> 27).wrapping_mul(-7723592293110705685);
		a ^ a >> 31
	}

	fn next_bits(&mut self, n: i32) -> i64 {
		self.next_long() >> 64 - n
	}
}

impl Default for XoroshiroRandomSource {
	fn default() -> Self {
		Self::new(-7046029254386353131, 7640891576956012809)
	}
}

impl RandomSource for XoroshiroRandomSource {
	fn set_seed(&mut self, seed: i64) {
		Self::from(seed);
	}

	fn consume(&mut self, n: i32) {
		for _ in 0..n {
			self.next_long();
		}
	}

	fn next_int(&mut self) -> i32 {
		self.next_long() as i32
	}

	fn next_int_max(&mut self, max: i32) -> i32 {
		assert!(max > 0);
		((self.next_long() as i32) % max).abs()
	}

	fn next_long(&mut self) -> i64 {
		let res = (self.lo.wrapping_add(self.hi)).wrapping_shl(17).wrapping_add(self.lo);
		let a = self.hi ^ self.lo;
		self.lo = (self.lo << 49) ^ a ^ self.hi << 21;
		self.hi = a << 28;
		res
	}

	fn next_float(&mut self) -> f32 {
		self.next_bits(24) as f32 * Self::FLOAT_UNIT
	}

	fn next_double(&mut self) -> f64 {
		self.next_bits(53) as f64 * Self::DOUBLE_UNIT
	}
}
