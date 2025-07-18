/// Trait to standardize the creation using a directory folder
/// Basically every level of a PLX course can be created using a directory path.
/// See `models::Course`, `models::Skill` and `models::Exo`
use crate::core::file_utils::file_parser::ParseError as MajorParserIssue;
pub trait FromDir {
    fn from_dir(
        dir: &std::path::PathBuf,
        deep: bool, // whether the element must be deeply extracted
    ) -> Result<(Vec<dy::error::ParseError>, Self), MajorParserIssue>
    where
        Self: Sized;
}
