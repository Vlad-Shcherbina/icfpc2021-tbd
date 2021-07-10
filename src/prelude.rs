// Reexports of the definitions that are useful and has low chance of collisions.
// It should be possible to write `use crate::prelude::*` in most cases.

pub use crate::util::project_path;
pub use crate::geom::Pt;
pub use crate::domain_model::{EPS_BASE, Figure, Problem, Pose, PoseBonus};
