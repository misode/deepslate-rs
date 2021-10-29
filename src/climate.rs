use wasm_bindgen::prelude::*;

const SPACE: usize = 7;
type TargetSpace = [i64; SPACE];
type ParamSpace = [Param; SPACE];
const QUANTIZE_SCALE: f64 = 10000.0;
type Biome = i32;

pub fn target(temperature: f64, humidity: f64, continentalness: f64, erosion: f64, weirdness: f64, depth: f64) -> TargetPoint {
	TargetPoint::new(temperature, humidity, continentalness, erosion, weirdness, depth)
}

#[allow(dead_code)]
pub fn parameters(temperature: f64, humidity: f64, continentalness: f64, erosion: f64, weirdness: f64, depth: f64, offset: f64) -> ParamPoint {
	ParamPoint::new(Param::point(temperature), Param::point(humidity), Param::point(continentalness), Param::point(erosion), Param::point(weirdness), Param::point(depth), offset)
}

fn quantize(x: f64) -> i64 {
	(x * QUANTIZE_SCALE) as i64
}

fn unquantize(x: i64) -> f64 {
	x as f64 / QUANTIZE_SCALE
}

#[derive(Clone, Copy)]
pub struct Param {
	min: i64,
	max: i64,
}

impl Param {
	pub fn point(v: f64) -> Param {
		Param::span(v, v)
	}

	pub fn span(min: f64, max: f64) -> Param {
		Param::new(quantize(min), quantize(max))
	}

	fn new(min: i64, max: i64) -> Param {
		assert!(min <= max);
		Param { min, max }
	}

	fn distance(&self, x: i64) -> i64 {
		let diff_max = x - self.max;
		let diff_min = self.min - x;
		if diff_max > 0 { diff_max } else { diff_min.max(0) }
	}

	fn union(&self, other: Option<Param>) -> Param {
		match other {
			Some(p) => Param::new(self.min.min(p.min), self.max.max(p.max)),
			None => *self,
		}
	}
}

pub struct ParamPoint {
	temperature: Param,
	humidity: Param,
	continentalness: Param,
	erosion: Param,
	weirdness: Param,
	depth: Param,
	offset: i64,
}

impl ParamPoint {
	pub fn new(temperature: Param, humidity: Param, continentalness: Param, erosion: Param, weirdness: Param, depth: Param, offset: f64) -> Self {
		Self {
			temperature,
			humidity,
			continentalness,
			erosion,
			weirdness,
			depth,
			offset: quantize(offset),
		}
	}

	fn space(&self) -> ParamSpace {
		[self.temperature, self.humidity, self.continentalness, self.erosion, self.weirdness, self.depth, Param::new(self.offset, self.offset)]
	}
}

pub struct TargetPoint {
	temperature: i64,
	humidity: i64,
	continentalness: i64,
	erosion: i64,
	weirdness: i64,
	depth: i64,
}

impl TargetPoint {
	pub fn new(temperature: f64, humidity: f64, continentalness: f64, erosion: f64, weirdness: f64, depth: f64) -> Self {
		Self {
			temperature: quantize(temperature),
			humidity: quantize(humidity),
			continentalness: quantize(continentalness),
			erosion: quantize(erosion),
			weirdness: quantize(weirdness),
			depth: quantize(depth),
		}
	}

	fn space(&self) -> TargetSpace {
		[self.temperature, self.humidity, self.continentalness, self.erosion, self.weirdness, self.depth, 0]
	}

	pub fn vec(&self) -> Vec<f64> {
		Vec::from(self.space().map(unquantize))
	}
}

#[wasm_bindgen]
pub struct ParameterList {
	root: Node,
}

impl ParameterList {
	pub fn new(biomes: &[(ParamPoint, Biome)]) -> Self {
		let nodes = biomes.iter().map(|(point, biome)| {
			Node::leaf(point, biome.clone())
		}).collect::<Vec<_>>();
		Self {
			root: Node::build(nodes)
		}
	}

	pub fn find(&self, target: TargetPoint) -> Biome {
		let node = self.root.search(target.space());
		node.biome.as_ref().expect("Expected a leaf node").clone()
	}
}

#[derive(Clone)]
pub struct Node {
	space: [Param; SPACE],
	children: Vec<Node>,
	biome: Option<Biome>,
}

impl Node {
	fn subtree(children: Vec<Node>) -> Self {
		Self {
			space: Self::build_space(&children),
			children,
			biome: None,
		}
	}

	fn leaf(point: &ParamPoint, biome: Biome) -> Self {
		Self {
			space: point.space(),
			children: Vec::new(),
			biome: Some(biome),
		}
	}

	fn search(&self, values: TargetSpace) -> &Node {
		match self {
			Node { children, biome: None, .. } => {
				let mut dist = i64::MAX;
				let mut result = self;

				for node in children {
					let d1 = node.distance(values);
					if dist <= d1 { continue };
					let child = node.search(values);
					let d2 = if node == child { d1 } else { child.distance(values) };
					if dist <= d2 { continue };
					dist = d2;
					result = child;
				}

				return result
			}
			_ => self
		}
	}

	fn distance(&self, target: TargetSpace) -> i64 {
		let mut dist: i64 = 0;
		for i in 0..SPACE {
			let d = self.space[i].distance(target[i]);
			dist += d * d;
		}
		dist
	}

	fn build(mut nodes: Vec<Node>) -> Node {
		match nodes.len() {
			0 => panic!("Need at least one child to build a node"),
			1 => nodes.pop().unwrap(),
			2..=10 => {
				nodes.sort_by_cached_key(|node| Node::cost(&node.space));
				Node::subtree(nodes)
			},
			_ => {
				let mut min_cost = i64::MAX;
				let mut min_n = 0;
				let mut min_buckets = None;

				for i in 0..SPACE {
					Node::sort(&mut nodes, i, false);
					let buckets = Node::bucketize(nodes.clone());
					let mut cost = 0;
					for bucket in buckets.iter() {
						cost += Node::cost(&bucket.space)
					}
					if min_cost <= cost { continue }; 
					min_cost = cost;
					min_n = i;
					min_buckets = Some(buckets);
				}

				let mut buckets = min_buckets.expect("Error splitting nodes in buckets");
				Node::sort(&mut buckets, min_n, true);
				let mut result: Vec<Node> = Vec::with_capacity(buckets.len());
				for bucket in buckets {
					if let Node { children, biome: None, .. } = bucket {
						result.push(Node::build(children))
					}
				}
				Node::subtree(result)
			},
		}
	}

	fn sort(nodes: &mut Vec<Node>, n: usize, abs: bool) {
		nodes.sort_by_cached_key(|node| {
			[0; SPACE].map(|i| {
				let param = node.space[(n + i) % SPACE];
				let mid = (param.min + param.max) / 2;
				if abs { mid.abs() } else { mid }
			})
		});
	}

	fn bucketize(nodes: Vec<Node>) -> Vec<Node> {
		let mut buckets = Vec::new();
		let mut buffer = Vec::new();

		let n = (10.0 as f64).powi(((nodes.len() as f64 - 0.01).ln() / (10.0 as f64).ln()).floor() as i32) as usize;
		for node in nodes {
			buffer.push(node);
			if buffer.len() >= n {
				buckets.push(Node::subtree(buffer));
				buffer = Vec::new();
			}
		}
		if buffer.len() != 0 {
			buckets.push(Node::subtree(buffer))
		}
		buckets
	}

	fn cost(space: &ParamSpace) -> i64 {
		let mut cost = 0;
		for param in space {
			cost += (param.max - param.min).abs()
		}
		cost
	}

	fn build_space(nodes: &Vec<Node>) -> ParamSpace {
		assert!(!nodes.is_empty(), "SubTree needs at least one child");
		let mut space: [Option<Param>; SPACE] = [None; SPACE];
		for node in nodes {
			for i in 0..SPACE {
				space[i] = Some(node.space[i].union(space[i]))
			}
		}
		let res = space.map(|p| p.unwrap());
		res
	}
}

impl PartialEq for Node {
	fn eq(&self, other: &Node) -> bool {
		std::ptr::eq(self, other)
	}
}
