use flat3d_inject::api::LinkSnapshot;
use flat3d_inject::setup::Packages;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub title: &'static str,
    pub done: bool,
    pub manual: bool,
    pub hint: &'static str,
}
pub fn steps(_packages: Option<&Packages>, _snapshot: Option<&LinkSnapshot>) -> Vec<Step> {
    Vec::new()
}
pub fn all_done(_steps: &[Step]) -> bool {
    false
}
