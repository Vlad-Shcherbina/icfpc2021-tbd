#![allow(dead_code)]

use std::cmp::{max, min};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(into="(i64, i64)")]
#[serde(from="(i64, i64)")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pt {
    pub x: i64,
    pub y: i64,  // y axis points down
}

impl Pt {
    pub fn new(x: i64, y: i64) -> Self {
        Pt { x, y }
    }

    pub fn cross(self, other: Pt) -> i64 {
        self.x * other.y - self.y * other.x
    }

    /// length squared
    pub fn len2(self) -> i64 {
        self.x * self.x + self.y * self.y
    }

    /// distance squared
    pub fn dist2(self, other: Pt) -> i64 {
        (self - other).len2()
    }

    pub fn angle(self) -> f64 {
        (self.y as f64).atan2(self.x as f64)
    }
}

impl std::fmt::Debug for Pt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pt({}, {})", self.x, self.y)
    }
}

impl From<Pt> for (i64, i64) {
    fn from(p: Pt) -> Self {
        (p.x, p.y)
    }
}

impl From<(i64, i64)> for Pt {
    fn from(p: (i64, i64)) -> Self {
        Pt { x: p.0, y: p.1 }
    }
}

impl std::ops::Add for Pt {
    type Output = Pt;

    fn add(self, rhs: Self) -> Self::Output {
        Pt { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl std::ops::Sub for Pt {
    type Output = Pt;

    fn sub(self, rhs: Self) -> Self::Output {
        Pt { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

#[derive(Debug, PartialEq)]
pub enum Intersection {
    No,  // either no intersection or collinear
    Endpoint(Pt),  // intersect at one of the segment ends
    Internal,
}

pub fn segment_intersection((pt1, pt2): (Pt, Pt), (pt3, pt4): (Pt, Pt)) -> Intersection {
    assert_ne!(pt1, pt2);
    assert_ne!(pt3, pt4);

    let bb_x1 = pt1.x.min(pt2.x).max(pt3.x.min(pt4.x));
    let bb_y1 = pt1.y.min(pt2.y).max(pt3.y.min(pt4.y));
    let bb_x2 = pt1.x.max(pt2.x).min(pt3.x.max(pt4.x));
    let bb_y2 = pt1.y.max(pt2.y).min(pt3.y.max(pt4.y));

    if bb_x1 > bb_x2 || bb_y1 > bb_y2 {
        return Intersection::No;  // bounding boxes don't overlap
    }

    let d1 = pt2 - pt1;
    let d2 = pt4 - pt3;

    if d1.cross(d2) == 0 {
        return Intersection::No;  // collinear
    }

    let alpha = d1.cross(pt3 - pt1);
    let beta = d1.cross(pt4 - pt1);
    assert_ne!(alpha, beta);

    let mut denom = beta - alpha;
    let mut x_numer = pt3.x * beta - pt4.x * alpha;
    let mut y_numer = pt3.y * beta - pt4.y * alpha;
    // actual intersection coords are (x_numer/denom, y_numer/denom),
    // but we don't want to deal with fractions

    if denom < 0 {
        denom = -denom;
        x_numer = -x_numer;
        y_numer = -y_numer;
    }

    for &pt in &[pt1, pt2, pt3, pt4] {
        if pt.x * denom == x_numer && pt.y * denom == y_numer {
            return Intersection::Endpoint(pt);
        }
    }

    if bb_x1 * denom <= x_numer && x_numer <= bb_x2 * denom
    && bb_y1 * denom <= y_numer && y_numer <= bb_y2 * denom {
        Intersection::Internal
    } else {
        Intersection::No
    }
}

#[cfg(test)]
fn check_intersection(mut seg1: (Pt, Pt), mut seg2: (Pt, Pt), expected: Intersection) {
    for _ in 0..2 {
        assert_eq!(segment_intersection(seg1, seg2), expected);
        std::mem::swap(&mut seg1, &mut seg2);
    }
}

#[cfg(test)]
#[test]
fn test_intersection() {
    check_intersection(
        (Pt::new(0, 0), Pt::new(1, 1)),
        (Pt::new(1, 10), Pt::new(0, 11)),
        Intersection::No);

    check_intersection(
        (Pt::new(0, 0), Pt::new(1, 1)),
        (Pt::new(1, 0), Pt::new(0, 1)),
        Intersection::Internal);

    check_intersection(
        (Pt::new(0, 0), Pt::new(10, 20)),
        (Pt::new(5, 10), Pt::new(6, 8)),
        Intersection::Endpoint(Pt::new(5, 10)));

    check_intersection(
        (Pt::new(0, 0), Pt::new(10, 20)),
        (Pt::new(5, 10), Pt::new(30, 60)),
        Intersection::No);
}

pub fn poly_edges(poly: &[Pt]) -> impl Iterator<Item=(Pt, Pt)> + '_ {
    poly.iter().copied().zip(
        poly.iter().copied().skip(1).chain(std::iter::once(poly[0])))
}

// positive if CCW
pub fn poly_area_doubled(poly: &[Pt]) -> i64 {
    poly_edges(poly).map(|(a, b)| (b.x - a.x) * (a.y + b.y)).sum()
}

// boundary included
pub fn pt_in_poly(pt: Pt, poly: &[Pt]) -> bool {
    let mut odd = false;
    for (pt1, pt2) in poly_edges(poly) {
        if pt == pt1 {
            return true;
        }
        if pt1.y == pt2.y {
            if pt.y == pt1.y &&
               pt1.x.min(pt2.x) <= pt.x && pt.x <= pt1.x.max(pt2.x) {
                return true;
            }
            continue;
        }
        if pt1.y.min(pt2.y) <= pt.y && pt.y < pt1.y.max(pt2.y) {
            let t_numer = pt.y - pt1.y;
            let mut denom = pt2.y - pt1.y;
            assert_ne!(denom, 0);
            let mut x_numer = pt1.x * denom + t_numer * (pt2.x - pt1.x);
            if denom < 0 {
                denom = -denom;
                x_numer = -x_numer;
            }
            match x_numer.cmp(&(pt.x * denom)) {
                std::cmp::Ordering::Equal => return true,
                std::cmp::Ordering::Less => {}
                std::cmp::Ordering::Greater => odd = !odd,
            }
        }
    }
    odd
}

#[cfg(test)]
fn check_pt_in_poly(pt: Pt, poly: &[Pt], expected: bool) {
    let mut poly = poly.to_owned();
    for _ in 0..2 {
        for _ in 0..poly.len() {
            assert_eq!(pt_in_poly(pt, &poly), expected);
            poly.rotate_left(1);
        }
        poly.reverse();
    }
}

#[cfg(test)]
#[test]
fn test_pt_in_poly() {
    let quad = &[
        Pt::new(100, 200),
        Pt::new(110, 200),
        Pt::new(110, 210),
        Pt::new(100, 210),
    ];
    for &pt in quad {
        check_pt_in_poly(pt, quad, true);
    }

    // edges
    check_pt_in_poly(Pt::new(105, 200), quad, true);
    check_pt_in_poly(Pt::new(105, 210), quad, true);
    check_pt_in_poly(Pt::new(100, 205), quad, true);
    check_pt_in_poly(Pt::new(110, 205), quad, true);

    // internal
    check_pt_in_poly(Pt::new(105, 205), quad, true);

    // outside
    check_pt_in_poly(Pt::new(115, 205), quad, false);
    check_pt_in_poly(Pt::new(105, 215), quad, false);
    check_pt_in_poly(Pt::new(95, 205), quad, false);
    check_pt_in_poly(Pt::new(105, 195), quad, false);

    //////////////////////////////////////////

    let triangle = &[
        Pt::new(100, 200),
        Pt::new(110, 200),
        Pt::new(100, 210),
    ];
    for &pt in triangle {
        check_pt_in_poly(pt, triangle, true);
    }

    // edges
    check_pt_in_poly(Pt::new(105, 200), triangle, true);
    check_pt_in_poly(Pt::new(100, 205), triangle, true);
    check_pt_in_poly(Pt::new(105, 205), triangle, true);

    // internal
    check_pt_in_poly(Pt::new(102, 202), triangle, true);

    // outside
    check_pt_in_poly(Pt::new(108, 208), triangle, false);

    check_pt_in_poly(Pt::new(115, 205), triangle, false);
    check_pt_in_poly(Pt::new(105, 215), triangle, false);
    check_pt_in_poly(Pt::new(95, 205), triangle, false);
    check_pt_in_poly(Pt::new(105, 195), triangle, false);
}

pub fn pt_in_segment(pt: Pt, seg: (Pt, Pt)) -> bool {
    if pt.x < seg.0.x && pt.x < seg.1.x {
        return false;
    }
    if pt.y < seg.0.y && pt.y < seg.1.y {
        return false;
    }
    if pt.x > seg.0.x && pt.x > seg.1.x {
        return false;
    }
    if pt.y > seg.0.y && pt.y > seg.1.y {
        return false;
    }
    (seg.0 - pt).cross(seg.0 - seg.1) == 0
}

// including boundary
pub fn segment_in_poly(seg: (Pt, Pt), poly: &[Pt]) -> bool {
    // TODO: This check is incorrect in some cases.
    // I think I know how to do the check correctly,
    // but it's hard, this is a placeholder for now.

    if !pt_in_poly(seg.0, poly) || !pt_in_poly(seg.1, poly) {
        return false;
    }

    if seg.0 == seg.1 {
        return true;
    }

    let sign = poly_area_doubled(poly).signum();

    for edge in poly_edges(poly) {
        match segment_intersection(seg, edge) {
            Intersection::Internal => return false,
            Intersection::No => {}
            Intersection::Endpoint(pt) => {
                if pt != edge.0 && pt != edge.1 {
                    assert!(pt == seg.0 || pt == seg.1);
                    let other = seg.0 + seg.1 - pt;
                    let d = other - pt;
                    let d1 = edge.0 - pt;
                    let d2 = edge.1 - pt;
                    if sign * qqqqqq(d1, d, d2) < 0 {
                        return false;
                    }
                }
            }
        }
    }

    for &(a, b) in &[seg, (seg.1, seg.0)] {
        if let Some(i) = poly.iter().position(|&p| p == a) {
            let i1 = if i == 0 { poly.len() - 1 } else { i - 1 };
            let i2 = if i == poly.len() - 1 { 0 } else { i + 1 };
            let d1 = poly[i1] - poly[i];
            let d2 = poly[i2] - poly[i];
            let d = b - a;
            if sign * qqqqqq(d1, d, d2) < 0 {
                return false;
            }
        }
    }

    for i in 0..poly.len() {
        if !pt_in_segment(poly[i], seg) {
            continue;
        }
        let i1 = if i == 0 { poly.len() - 1 } else { i - 1 };
        let i2 = if i == poly.len() - 1 { 0 } else { i + 1 };
        for &other in &[seg.0, seg.1] {
            if other == poly[i] {
                continue;
            }
            let d1 = poly[i1] - poly[i];
            let d2 = poly[i2] - poly[i];
            let d = other - poly[i];
            if sign * qqqqqq(d1, d, d2) < 0 {
                return false;
            }
        }
    }

    true
}

fn qqqqqq(v1: Pt, v2: Pt, v3: Pt) -> i64 {
    let a = [v1.angle(), v2.angle(), v3.angle()];
    let mut res = 1;
    for i in 0..3 {
        for j in 0..i {
            #[allow(clippy::float_cmp)]
            if a[i] == a[j] {
                return 0;
            }
            if a[i] < a[j] {
                res = -res;
            }
        }
    }
    res
}

#[cfg(test)]
fn check_segment_in_poly(mut seg: (Pt, Pt), poly: &[Pt], expected: bool) {
    let mut poly = poly.to_owned();
    for _ in 0..2 {
        for _ in 0..2 {
            for _ in 0..poly.len() {
                assert_eq!(segment_in_poly(seg, &poly), expected);
                poly.rotate_left(1);
            }
            poly.reverse();
        }
        seg = (seg.1, seg.0);
    }
}

#[cfg(test)]
#[test]
fn test_segment_in_poly_bug() {
    let poly = vec![
        Pt::new(100, 200),
        Pt::new(110, 200),
        Pt::new(105, 205),
        Pt::new(110, 210),
        Pt::new(100, 210),
    ];

    check_segment_in_poly((Pt::new(100, 200), Pt::new(105, 205)), &poly, true);
    check_segment_in_poly((Pt::new(101, 200), Pt::new(107, 203)), &poly, true);
    check_segment_in_poly((Pt::new(100, 200), Pt::new(107, 203)), &poly, true);

    check_segment_in_poly((Pt::new(110, 200), Pt::new(110, 210)), &poly, false);
    check_segment_in_poly((Pt::new(110, 200), Pt::new(109, 209)), &poly, false);
    check_segment_in_poly((Pt::new(108, 202), Pt::new(109, 209)), &poly, false);

    let poly = vec![
        Pt::new(0, 0),
        Pt::new(4, 4),
        Pt::new(1, 3),
        Pt::new(0, 1),
        Pt::new(-1, 3),
        Pt::new(-4, 4),
    ];
    check_segment_in_poly((Pt::new(-2, 3), Pt::new(2, 3)), &poly, false);
}

// Rotate a point around another point
pub fn rotate_point(pt: Pt, pivot: Pt, angle: i16) -> Pt {
    let rad = angle as f64 * std::f64::consts::PI / 180.0;

    let d_x = (pt.x-pivot.x) as f64;
    let d_y = (pt.y-pivot.y) as f64;

    let new_x = rad.cos() * d_x - rad.sin() * d_y + pivot.x as f64;
    let new_y = rad.sin() * d_x + rad.cos() * d_y + pivot.y as f64;

    Pt::new(new_x.round() as i64, new_y.round() as i64)
}

// Rotate the collection of points around a given point
pub fn rotate_poly(poly: &[Pt], pivot: Pt, angle: i16) -> Vec<Pt> {
    poly.iter().map(|pt| rotate_point(*pt, pivot, angle)).collect::<Vec<Pt>>()
}

pub fn bounding_box(points: &[Pt]) -> Option<(Pt, Pt)> {
    let mut first = true;
    let mut x_min = -1;
    let mut x_max = -1;
    let mut y_min = -1;
    let mut y_max = -1;
    for pt in points {
        if first {
            x_min = pt.x;
            x_max = pt.x;
            y_min = pt.y;
            y_max = pt.y;
            first = false;
        } else {
            x_min = x_min.min(pt.x);
            x_max = x_max.max(pt.x);
            y_min = y_min.min(pt.y);
            y_max = y_max.max(pt.y);
       }
    }
    if first {
        None
    } else {
        Some((Pt::new(x_min, y_min), Pt::new(x_max, y_max)))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BBox {
    pub min_x: i64,
    pub max_x: i64,
    pub min_y: i64,
    pub max_y: i64,
}

impl BBox {
    pub fn from_pts(pts: &[Pt]) -> Self {
        let (pt_min, pt_max) = bounding_box(pts).unwrap();
        BBox{
            min_x: pt_min.x,
            max_x: pt_max.x,
            min_y: pt_min.y,
            max_y: pt_max.y
        }

    }

    pub fn intersect(&self, other: &Self) -> Option<BBox> {
        let min_x = max(self.min_x, other.min_x);
        let max_x = min(self.max_x, other.max_x);
        if min_x > max_x { return None; }

        let min_y = max(self.min_y, other.min_y);
        let max_y = min(self.max_y, other.max_y);
        if min_y > max_y { return None; }

        Some(BBox {
            min_x,
            max_x,
            min_y,
            max_y
        })
    }
}

#[cfg(test)]
#[test]
fn test_rotate_point() {
    assert_eq!(rotate_point(Pt::new(10,10), Pt::new(10,11), 180), Pt::new(10, 12));
    assert_eq!(rotate_point(Pt::new(0,0), Pt::new(10,10), 180), Pt::new(20, 20));
    assert_eq!(rotate_point(Pt::new(0,0), Pt::new(10,10), 90), Pt::new(20, 0));
}

#[cfg(test)]
#[test]
fn test_rotate_poly() {
    let quad = &[
        Pt::new(0, 0),
        Pt::new(0, 10),
        Pt::new(10, 0),
        Pt::new(10, 10),
    ];
    let expected = &[
        Pt::new(20, 20),
        Pt::new(20, 10),
        Pt::new(10, 20),
        Pt::new(10, 10),
    ];
    assert_eq!(rotate_poly(quad, Pt::new(10,10), 0), quad);
    assert_eq!(rotate_poly(quad, Pt::new(10,10), 180), expected);
}

#[cfg(test)]
#[test]
fn test_bounding_box() {
    assert_eq!(bounding_box(&vec!{}), None);
    assert_eq!(bounding_box(&vec!{Pt::new(1, 3)}), Some((Pt::new(1, 3), Pt::new(1,3))));
    assert_eq!(bounding_box(&vec!{Pt::new(1, 3), Pt::new(0, 4)}), Some((Pt::new(0, 3), Pt::new(1,4))));
}
