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
