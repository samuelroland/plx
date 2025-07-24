use std::sync::Arc;

use super::{exo::Exo, exo_state::ExoState};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use specta_macros::Type;

#[serde_as]
#[derive(Serialize, Debug, PartialEq, Eq, Clone, Type)]
pub struct Skill {
    pub name: String,
    pub path: std::path::PathBuf,
    // Fix serialization by using it as a normal Vec
    #[serde_as(as = "Vec<_>")]
    pub exos: Arc<Vec<Exo>>,
}
#[derive(Deserialize, Serialize)]
struct SkillInfo {
    name: String,
    #[serde(rename = "exos")]
    exo_folders: Vec<std::path::PathBuf>,
}
impl Skill {
    pub fn get_next_todo_exo(&self) -> Option<(usize, &Exo)> {
        self.exos
            .iter()
            .enumerate()
            .find(|(_, exo)| exo.state == ExoState::Todo)
    }
}
