#[derive(serde::Serialize, serde::Deserialize)]
#[serde(into="(i64, i64)")]
#[serde(from="(i64, i64)")]
#[derive(Clone, Copy, Debug)]
pub struct Pt {
    pub x: i64,
    pub y: i64,  // y axis points down
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