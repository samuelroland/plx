use crate::core::file_utils::{
    file_parser::ParseError as MajorParserIssue, file_utils::list_dir_folders,
};
use log::warn;
use plx_dy::{parse_course, parse_skills};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use specta_macros::Type;
use std::{path::PathBuf, sync::Arc};

use crate::core::{
    file_utils::file_utils::read_file,
    parser::{
        from_dir::FromDir,
        object_creator::{create_object_from_file, write_object_to_file},
    },
};

use super::{
    constants::{COURSE_INFO_FILE, EXO_STATE_FILE, SKILL_INFO_FILE},
    exo::{Exo, ExoStateInfo},
    exo_state::ExoState,
    skill::Skill,
};

#[serde_as]
#[derive(Serialize, Debug, PartialEq, Eq, Type)]
#[typeshare::typeshare]
pub struct Course {
    pub name: String,
    pub code: String,
    pub goal: String,
    // Fix serialization by using it as a normal Vec
    #[serde_as(as = "Vec<_>")]
    pub(crate) skills: Arc<Vec<Skill>>,
    folder: PathBuf,
}

#[derive(Deserialize)]
pub(crate) struct CourseInfo {
    name: String,
    #[serde(rename = "skills")]
    skill_folders: Vec<std::path::PathBuf>,
}
impl Course {
    pub fn get_folder(&self) -> PathBuf {
        self.folder.clone()
    }

    /// Saves exo state to file
    fn save_exo_state(exo: &Exo, info: &ExoStateInfo) {
        if let Err(err) = write_object_to_file(&exo.folder.join(EXO_STATE_FILE), info) {
            warn!("Couldn't save exo state {:?}", err);
        }
    }
    /// Reads current exo state from file
    fn read_exo_state_info(exo: &Exo) -> ExoStateInfo {
        create_object_from_file::<ExoStateInfo>(&exo.folder.join(EXO_STATE_FILE))
            .unwrap_or_default()
    }
    // Set exo state and store it in file
    pub fn set_exo_state(exo: &Exo, state: ExoState) {
        let mut info = Course::read_exo_state_info(exo);
        info.state = state;
        Course::save_exo_state(exo, &info);
    }
    // Set exo as favorite or not and store it in file
    pub fn set_exo_favorite(exo: &Exo, is_favorite: bool) {
        let mut info = Course::read_exo_state_info(exo);
        info.favorite = is_favorite;
        Course::save_exo_state(exo, &info);
    }
}

impl FromDir for Course {
    ///
    /// Tries to build a course from given directory
    /// Returns Ok if we were able to parse the course info and at least one skill
    /// else Error
    ///
    fn from_dir(
        dir: &std::path::PathBuf,
        deep: bool,
    ) -> Result<(Vec<plx_dy::dy::error::ParseError>, Self), MajorParserIssue> {
        // Get course info by searching for the course.toml file
        // TODO magic value maybe change this
        let course_info_file = dir.join(COURSE_INFO_FILE);
        let mut errors = Vec::new();

        let course_file_content = read_file(&course_info_file)
            .map_err(|_| MajorParserIssue::FileNotFound(COURSE_INFO_FILE.to_string()))?;
        let dy_course_result = parse_course(
            &Some(course_info_file.to_str().unwrap_or_default().to_string()),
            &course_file_content,
        );
        let dy_course = dy_course_result
            .items
            .first()
            .ok_or(MajorParserIssue::ParseError(format!(
                "No course has been found into {COURSE_INFO_FILE}"
            )))?;
        let mut course = Course {
            name: dy_course.name.clone(),
            code: dy_course.code.clone(),
            goal: dy_course.goal.clone(),
            skills: Arc::new(vec![]),
            folder: dir.clone(),
        };
        errors.extend(dy_course_result.errors);

        // Don't even parse the skill in non deep mode
        if !deep {
            return Ok((errors, course));
        }

        // Parse the skills list (DYSkill), without the exos
        let skills_file_info = &dir.join(SKILL_INFO_FILE);
        let skills_file_content = read_file(skills_file_info)
            .map_err(|err| MajorParserIssue::ReadFileError(err.to_string()))?;
        let dy_skills_result = parse_skills(
            &Some(skills_file_info.to_str().unwrap_or_default().to_string()),
            &skills_file_content,
        );

        if dy_skills_result.items.is_empty() {
            Err(MajorParserIssue::ErrorParsingSkills(format!(
                "Couldn't find any skill in {SKILL_INFO_FILE}"
            )))
        } else {
            errors.extend(dy_skills_result.errors);
            // Load all exos for each skill and build a Skill from the DYSkill
            let skills = dy_skills_result
                .items
                .iter()
                .map(|dy_skill| {
                    let skill_folder = PathBuf::from(&dy_skill.directory);
                    let exos = list_dir_folders(&dir.join(&skill_folder))
                        .unwrap_or_default()
                        .iter()
                        .filter_map(|f| {
                            if let Ok((exo_errors, exo)) = Exo::from_dir(f, true) {
                                errors.extend(exo_errors);
                                Some(exo)
                            } else {
                                None
                            } // just ignore major issue at exo level for now
                        })
                        .collect();
                    Skill {
                        name: dy_skill.name.clone(),
                        path: skill_folder,
                        exos: Arc::new(exos),
                    }
                })
                .collect();

            course.skills = Arc::new(skills);
            Ok((errors, course))
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use std::{str::FromStr, sync::Arc};

    use crate::models::{
        check::{Check, CheckTest},
        exo::Exo,
        exo_state::ExoState,
    };

    use super::*;

    #[test]
    fn test_full_hierarchy() {
        let course_path = std::path::PathBuf::from_str("examples/new_mock").unwrap();
        let (errors, course) = Course::from_dir(&course_path, true).unwrap();

        assert_eq!(errors, []);
        assert_eq!(course,
        Course {
            name: "PLX demo course".to_string(),
            code: "DEMO".to_string(),
            goal: "This demo course has been created to show the features of PLX.\n".to_string(),
            skills: Arc::new(vec![
                Skill {
                    name: "Introduction".to_string(),
                    path: "intro".into(),
                    exos: vec![
                        Exo {
                            name: "Basic arguments usage".to_string(),
                            instruction: Some(
                                "The 2 first program arguments are the firstname and number of legs of a dog. Print a full sentence about the dog. Make sure there is at least 2 arguments, print an error if not.".to_string(),
                            ),
                            state: ExoState::Todo,
                            files: vec![
                                "examples/new_mock/intro/basic-args/main.c".into(),
                                "examples/new_mock/intro/basic-args/exo.dy".into(),
                            ],
                            solutions: [
                                "examples/new_mock/intro/basic-args/main.sol.c".into(),
                            ].into(),
                            checks: [
                                Check {
                                    name: "Joe + 5 legs".to_string(),
                                    args: [
                                        "Joe".to_string(),
                                        "5".to_string(),
                                    ].into(),
                                    test: CheckTest::Output {
                                        expected: "The dog is Joe and has 5 legs".to_string(),
                                    },
                                },
                                Check {
                                    name: "No arg -> error".to_string(),
                                    args: [].into(),
                                    test: CheckTest::Output {
                                        expected: "Error: missing argument firstname and legs number".to_string(),
                                    },
                                },
                                Check {
                                    name: "One arg -> error".to_string(),
                                    args: [
                                        "Joe".to_string(),
                                    ].into(),
                                    test: CheckTest::Output {
                                        expected: "Error: missing argument firstname and legs number".to_string(),
                                    },
                                },
                            ].into(),
                            favorite: false,
                            folder: "examples/new_mock/intro/basic-args".into(),
                        },
                        Exo {
                            name: "Basic output printing".to_string(),
                            instruction: Some(
                                "Just print 2 lines".to_string(),
                            ),
                            state: ExoState::Todo,
                            files: [
                                "examples/new_mock/intro/basic-output/main.c".into(),
                                "examples/new_mock/intro/basic-output/exo.dy".into(),
                            ].into(),
                            solutions: [
                                "examples/new_mock/intro/basic-output/main.sol.c".into(),
                            ].into(),
                            checks: [
                                Check {
                                    name: "Lines are correct".to_string(),
                                    args: [].into(),
                                    test: CheckTest::Output {
                                        expected: "PLX is amazing !\nThis is a neutral opinion...".to_string(),
                                    },
                                },
                            ].into(),
                            favorite: false,
                            folder: "examples/new_mock/intro/basic-output".into(),
                        },
                        Exo {
                            name: "Salue-moi".to_string(),
                            instruction: Some(
                                "Un petit programme qui te salue avec ton nom complet.".to_string(),
                            ),
                            state: ExoState::Todo,
                            files: [
                                "examples/new_mock/intro/salue-moi/main.c".into(),
                                "examples/new_mock/intro/salue-moi/exo.dy".into(),
                            ].into(),
                            solutions: [].into(),
                            checks: [
                                Check {
                                    name: "Il est possible d'être salué avec son nom complet".into(),
                                    args: [].into(),
                                    test: CheckTest::Output {
                                        expected: "Quel est ton prénom ?".to_string(),
                                    },
                                },
                            ].into(),
                            favorite: false,
                            folder: "examples/new_mock/intro/salue-moi".into(),
                        },
                    ].into(),
                },
                Skill {
                    name: "Enumerations".to_string(),
                    path: "enums".into(),
                    exos: Arc::new(vec![]),
                },
                Skill {
                    name: "Structures".to_string(),
                    path: "structs".into(),
                    exos: vec![
                        Exo {
                            name: "Participants à la réunion".to_string(),
                            instruction: Some(
                                "A chaque réunion d'une association, on prend les présences pour les intégrer au procès verbal. Ce programme permet de rentrer un nombre total de membre, de rentrer les prénoms de chacun et d'afficher la liste à la fin. Pour stocker les personnes intermédiaires, il est nécessaire d'utiliser **des structures**.".to_string(),
                            ),
                            state: ExoState::Todo,
                            files: vec![
                                "examples/new_mock/structs/small-meeting-participants/main.cpp".into(),
                                "examples/new_mock/structs/small-meeting-participants/exo.dy".into(),
                            ],
                            solutions: vec![
                                "examples/new_mock/structs/small-meeting-participants/main.sol.cpp".into(),
                            ],
                            checks: vec![
                                Check {
                                    name: "Petite réunion".to_string(),
                                    args:vec! [],
                                    test: CheckTest::Output {
                                        expected: "nombre de personnes dans l'association ?".to_string(),
                                    },
                                },
                            ],
                            favorite: false,
                            folder: "examples/new_mock/structs/small-meeting-participants".into(),
                        },
                    ].into(),
                },
                Skill {
                    name: "Pointers and memory".to_string(),
                    path: "pointers".into(),
                    exos: Arc::new(vec![]),
                },
                Skill {
                    name: "Parsing".to_string(),
                    path: "parsing".into(),
                    exos: Arc::new(vec![]),
                },
            ]),
            folder: "examples/new_mock".into(),
        },
    );
    }
}
