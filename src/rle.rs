#![allow(unused_imports)]
#![allow(dead_code)]

use std::iter::Peekable;
use integer_sqrt::IntegerSquareRoot;
use crate::geom::Pt;
use std::collections::HashSet;

// Invariants:
// In Run: [a, b) where a < b
// In Set1D: b_i < a_{i+1}
// In Set2D: no leading/trailing empty lines (=> empty set is uniquely represented with lines.len() == 0)

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
struct Run {
	a: i64,
	b: i64
}

#[derive(Eq, PartialEq, Debug, Clone)]
struct Set1D {
	runs: Vec<Run>
}

struct RunMerger<'a> {
	i: Peekable<std::slice::Iter<'a, Run>>,
	j: Peekable<std::slice::Iter<'a, Run>>
}

impl<'a> RunMerger<'a> {
	fn new(set1: &'a Set1D, set2: &'a Set1D) -> RunMerger<'a> {
		RunMerger{
			i: set1.runs.iter().peekable(),
			j: set2.runs.iter().peekable()
		}
	}

	fn next(&mut self) -> Option<&Run> {
		match (self.i.peek(), self.j.peek()) {
			(None, None) => None,
			(Some(_), None) => self.i.next(),
			(None, Some(_)) => self.j.next(),
			(Some(run_i), Some(run_j)) => {
				if run_i.a < run_j.a {
					self.i.next()
				} else {
					self.j.next()
				}
			}
		}
	}
}

impl Set1D {
	pub fn union(&self, other: &Set1D) -> Set1D {
		let mut merged = RunMerger::new(self, other);
		let mut current_run;
		match merged.next() {
			None => return Set1D::empty(),
			Some(&run) => current_run = run
 		}
		let mut runs = vec![];
		loop {
			match merged.next() {
				Some(&run) => {
					assert!(run.a >= current_run.a);
					if run.a <= current_run.b {
						current_run.b = current_run.b.max(run.b);
					} else {
						runs.push(current_run);
						current_run = run;
					}
				}
				None => {
					runs.push(current_run);
					return Set1D{runs};
				}
			}
		}
	}

	pub fn intersection(&self, other: &Set1D) -> Set1D {
		if self.is_empty() || other.is_empty() {
			return Set1D::empty();
		}
		let mut merged = RunMerger::new(self, other);
		let mut max_b;
		match merged.next() {
			None => return Set1D::empty(),
			Some(&run) => max_b = run.b
 		}
		let mut runs = vec![];
		loop {
			match merged.next() {
				Some(&run) => {
					if run.a < max_b {
						runs.push(Run{a: run.a, b: run.b.min(max_b)})
					}
					max_b = max_b.max(run.b);
				}
				None => return Set1D{runs}
			}
		}
	}

	fn is_empty(&self) -> bool {
		self.runs.is_empty()
	}

	fn empty() -> Set1D {
		Set1D{runs: vec![]}
	}
	
	fn one_run(a: i64, b: i64) -> Set1D {
		Set1D{runs: vec![Run{a, b}]}
	}
}

#[derive(Eq, PartialEq, Debug)]
struct Set2D {
	y_start: i64,
	lines: Vec<Set1D>
}

impl Set2D {
	fn y_end(&self) -> i64 {
		self.y_start + self.lines.len() as i64
	}

	fn push_line(&mut self, line: Set1D, empty_lines: &mut usize) {
		if line.is_empty() {
			*empty_lines += 1;
		} else {
			if self.lines.is_empty() {
				self.y_start += *empty_lines as i64;
			} else {
				self.lines.append(&mut vec![Set1D::empty(); *empty_lines]);
			}
			self.lines.push(line);
			*empty_lines = 0;
		}
	}

	pub fn union(&self, other: &Set2D) -> Set2D {
		let y_start = self.y_start.min(other.y_start);
		let y_end = self.y_end().max(other.y_end());
		let self_range = self.y_start..self.y_end();
		let other_range = other.y_start..other.y_end();
		let mut lines = vec![];
		for y in y_start..y_end {
			if self_range.contains(&y) && other_range.contains(&y) {
				lines.push(self.lines[(y - self.y_start) as usize]
					.union(&other.lines[(y - other.y_start) as usize]))
			} else if self_range.contains(&y) {
				lines.push(self.lines[(y - self.y_start) as usize].clone())
			} else if other_range.contains(&y) {
				lines.push(other.lines[(y - other.y_start) as usize].clone())
			} else {
				lines.push(Set1D::empty())
			}
		}
		Set2D{y_start, lines}
	}

	pub fn intersection(&self, other: &Set2D) -> Set2D {
		let y_start = self.y_start.max(other.y_start);
		if y_start > self.y_end().min(other.y_end()) {
			return Set2D::empty()
		}
		let mut result = Set2D{y_start, lines: vec![]};
		let index = |s: &Set2D| (y_start - s.y_start) as usize;
		let mut empty_lines = 0;
		for (line1, line2) in self.lines[index(self)..].iter()
								.zip(other.lines[index(other)..].iter()) {
			result.push_line(line1.intersection(line2), &mut empty_lines);
		}
		result
	}

	pub fn ring(x_center: i64, y_center: i64, d_min: i64, d_max: i64) -> Set2D {
		let width = d_max.integer_sqrt();
		let y_start = y_center - width;
		let mut result = Set2D{y_start, lines: vec![]};
		let mut empty_lines = 0;
		for y in - width ..= width {
			let x_sq_max = d_max - y * y;
			let x_max = x_sq_max.integer_sqrt();
			let x_sq_min = d_min - y * y;
			// Exactly one run unless there's an integer x satisfying x^2 < x_sq_min,
			// i.e. x <= (x_sq_min - 1).integer_sqrt(). In the latter case either
			// 2 or 0 runs.
			match (x_sq_min - 1).integer_sqrt_checked() {
				None => {
					result.push_line(Set1D::one_run(x_center - x_max, x_center + x_max + 1),
									 &mut empty_lines);
				}
				Some(x_min) => {
					if x_center - x_max < x_center - x_min {
						let run1 = Run{a: x_center - x_max, b: x_center - x_min};
						let run2 = Run{a: x_center + x_min + 1, b: x_center + x_max + 1};
						result.push_line(Set1D{runs: vec![run1, run2]},
										 &mut empty_lines);
					} else {
						empty_lines += 1;
					}
				}
			}
		}
		result
	}

	pub fn as_points(&self) -> Vec<Pt> {
		let mut result = vec![];
		for (y, line) in (self.y_start..).zip(self.lines.iter()) {
			for run in line.runs.iter() {
				result.append(&mut (run.a..run.b).map(|x| Pt{x, y}).collect())
			}
		}
		result
	}

	pub fn empty() -> Set2D {
		Set2D{y_start:0, lines: vec![]}
	}
}

#[test]
fn test_union_1_d() {
	let a = Set1D{runs: vec![]};
	let b = Set1D{runs: vec![
		Run{a: 0, b: 10},
		Run{a: 11, b: 20},
		Run{a: 30, b: 40}
	]};
	let c = Set1D{runs: vec![
		Run{a: 5, b: 12},
		Run{a: 35, b: 37},
		Run{a: 45, b: 47},
		Run{a: 50, b: 55}
	]};
	assert_eq!(a.union(&a), a);
	assert_eq!(a.union(&b), b);
	assert_eq!(b.union(&a), b);
	assert_eq!(b.union(&c), Set1D{runs: vec![
		Run{a: 0, b: 20},
		Run{a: 30, b: 40},
		Run{a: 45, b: 47},
		Run{a: 50, b: 55}
	]})
}

#[test]
fn test_intersection_1_d() {
	let a = Set1D{runs: vec![]};
	let b = Set1D{runs: vec![
		Run{a: 0, b: 10},
		Run{a: 11, b: 20},
		Run{a: 30, b: 40},
		Run{a: 48, b: 50},
		Run{a: 50, b: 52},
		Run{a: 54, b: 56},
		Run{a: 65, b: 70}
	]};
	let c = Set1D{runs: vec![
		Run{a: 5, b: 12},
		Run{a: 35, b: 37},
		Run{a: 45, b: 47},
		Run{a: 50, b: 60}
	]};
	assert_eq!(a.intersection(&a), a);
	assert_eq!(a.intersection(&b), a);
	assert_eq!(b.intersection(&a), a);
	assert_eq!(b.intersection(&c), Set1D{runs: vec![
		Run{a: 5, b: 10},
		Run{a: 11, b: 12},
		Run{a: 35, b: 37},
		Run{a: 50, b: 52},
		Run{a: 54, b:56}
	]})
}

// picked up from rail.rs (deltas()) for testing
// actually does too much work (ranges should be ~ -sqrt(max_d)..sqrt(max_d))
#[allow(dead_code)]
fn ring(x_center: i64, y_center: i64, min_d: i64, max_d: i64) -> Vec<Pt> {
    let mut result = vec![];
    for y in -max_d..=max_d {
        for x in -max_d..=max_d {
            let d = Pt { x, y }.len2();
            if min_d <= d && d <= max_d {
                result.push(Pt {x: x_center + x, y: y_center + y});
            }
        }
    }
    result
}

#[test]
fn test_ring() {
	fn check_eq(x: i64, y: i64, min_d: i64, max_d: i64) {
		// NB: assumes same ordering
		assert_eq!(Set2D::ring(x, y, min_d, max_d).as_points(),
				   ring(x, y, min_d, max_d));
	}
	check_eq(1, 2, 0, 0);
	check_eq(1, 2, 1, 10);
	check_eq(0, 0, 10, 100);
	check_eq(0, 0, 80, 81);
	// A tricky case which goes staight to 2-run lines.
	check_eq(0, 0, 79, 80);
}

#[test]
fn test_union_2d() {
	let empty = Set2D::empty();
	let a = Set2D::ring(0, 0, 10, 100);
	assert_eq!(a.union(&empty), a);
	assert_eq!(empty.union(&a), a);
	assert_eq!(a.union(&a), a);
	
	fn check_eq(x1: i64, y1: i64, min_d1: i64, max_d1: i64,
				x2: i64, y2: i64, min_d2: i64, max_d2: i64) {
		let set_rle: HashSet<Pt> = Set2D::ring(x1, y1, min_d1, max_d1)
			.union(&Set2D::ring(x2, y2, min_d2, max_d2))
			.as_points().iter().cloned().collect();
		let ring1: HashSet<Pt> = ring(x1, y1, min_d1, max_d2).iter().cloned().collect();
		let ring2: HashSet<Pt> = ring(x2, y2, min_d2, max_d2).iter().cloned().collect();
		let set_hash: HashSet<Pt> = ring1.union(&ring2).cloned().collect();
		assert_eq!(set_rle, set_hash);
	}

	check_eq(0, 0, 10, 100, 0, 0, 10, 100);
	check_eq(10, 0, 10, 100, 0, 0, 10, 100);
	check_eq(10, 10, 10, 100, 0, 0, 10, 100);
	check_eq(0, 0, 10, 100, 30, 0, 10, 100);
	check_eq(0, 0, 10, 100, 0, 30, 10, 100);
	check_eq(30, 0, 10, 100, 0, 0, 10, 100);
	check_eq(0, 30, 10, 100, 0, 0, 10, 100);	
}

#[test]
fn test_intersect_2d() {
	let empty = Set2D::empty();
	let a = Set2D::ring(0, 0, 1, 10);
	assert_eq!(a.intersection(&empty), empty);
	assert_eq!(empty.intersection(&a), empty);
	assert_eq!(a.intersection(&a), a);
	
	fn check_eq(x1: i64, y1: i64, min_d1: i64, max_d1: i64,
				x2: i64, y2: i64, min_d2: i64, max_d2: i64) {
		let set_rle: HashSet<Pt> = Set2D::ring(x1, y1, min_d1, max_d1)
			.intersection(&Set2D::ring(x2, y2, min_d2, max_d2))
			.as_points().iter().cloned().collect();
		let ring1: HashSet<Pt> = ring(x1, y1, min_d1, max_d2).iter().cloned().collect();
		let ring2: HashSet<Pt> = ring(x2, y2, min_d2, max_d2).iter().cloned().collect();
		let set_hash: HashSet<Pt> = ring1.intersection(&ring2).cloned().collect();
		assert_eq!(set_rle, set_hash);
	}

	check_eq(0, 0, 10, 100, 0, 0, 10, 100);
	check_eq(10, 0, 10, 100, 0, 0, 10, 100);
	check_eq(10, 0, 10, 15, 0, 0, 10, 15);
	check_eq(10, 10, 10, 100, 0, 0, 10, 100);
	check_eq(0, 0, 10, 100, 30, 0, 10, 100);
	check_eq(0, 0, 10, 100, 0, 30, 10, 100);
	check_eq(30, 0, 10, 100, 0, 0, 10, 100);
	check_eq(0, 30, 10, 100, 0, 0, 10, 100);	
}