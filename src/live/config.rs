use serde::{Deserialize, Serialize};
use specta_macros::Type;
use typeshare::typeshare;

use crate::{
    core::{file_utils::file_parser::ParseError, parser::object_creator::create_object_from_file},
    models::course::Course,
};

pub const LIVECONFIG_FILENAME: &str = "live.toml";

#[derive(Serialize, Deserialize, Debug, Type)]
#[typeshare]
pub struct LiveConfig {
    pub domain: String,
    pub port: u16,
    pub group_id: String,
}

impl LiveConfig {
    pub fn from_course(course: &Course) -> Result<Self, ParseError> {
        create_object_from_file(&course.get_folder().join(LIVECONFIG_FILENAME))
    }
}
