#![allow(dead_code)]

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(into="(i64, i64)")]
#[serde(from="(i64, i64)")]
#[derive(Clone, Copy, Debug, PartialEq)]
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

fn check_intersection(mut seg1: (Pt, Pt), mut seg2: (Pt, Pt), expected: Intersection) {
    for _ in 0..2 {
        assert_eq!(segment_intersection(seg1, seg2), expected);
        std::mem::swap(&mut seg1, &mut seg2);
    }
}

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

    for edge in poly_edges(poly) {
        match segment_intersection(seg, edge) {
            Intersection::Internal => return false,
            Intersection::No => {}
            Intersection::Endpoint(_) => {}
        }
    }

    true
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

#[test]
fn test_rotate_point() {
    assert_eq!(rotate_point(Pt::new(10,10), Pt::new(10,11), 180), Pt::new(10, 12));
    assert_eq!(rotate_point(Pt::new(0,0), Pt::new(10,10), 180), Pt::new(20, 20));
    assert_eq!(rotate_point(Pt::new(0,0), Pt::new(10,10), 90), Pt::new(20, 0));
}

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

#[test]
fn test_bounding_box() {
    assert_eq!(bounding_box(&vec!{}), None);
    assert_eq!(bounding_box(&vec!{Pt::new(1, 3)}), Some((Pt::new(1, 3), Pt::new(1,3))));
    assert_eq!(bounding_box(&vec!{Pt::new(1, 3), Pt::new(0, 4)}), Some((Pt::new(0, 3), Pt::new(1,4))));
}
