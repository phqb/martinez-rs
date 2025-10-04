#[derive(Default)]
pub(crate) struct Contour {
    pub points: Vec<[f64; 2]>,
    pub hole_ids: Vec<i64>,
    pub hole_of: Option<i64>,
    pub depth: i64,
}

impl Contour {
    pub fn is_exterior(&self) -> bool {
        self.hole_of.is_none()
    }
}
