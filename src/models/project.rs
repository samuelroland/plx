use std::sync::Arc;

use log::warn;
use serde::{Deserialize, Serialize};

use crate::core::{
    file_utils::file_parser::{ParseError, ParseWarning},
    parser::{
        from_dir::FromDir,
        object_creator::{self, create_object_from_file, write_object_to_file},
    },
};

use super::{
    constants::{COURSE_INFO_FILE, COURSE_STATE_FILE, EXO_STATE_FILE},
    exo::{Exo, ExoStateInfo},
    exo_state::ExoState,
    skill::Skill,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Project {
    pub(crate) name: String,
    pub(crate) skills: Arc<Vec<Skill>>,
    pub(crate) state: ProjectState,
    folder: std::path::PathBuf,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
pub(crate) struct ProjectState {
    pub(crate) curr_skill_idx: usize,
    pub(crate) curr_exo_idx: usize,
}
#[derive(Deserialize)]
pub(crate) struct ProjectInfo {
    name: String,
    #[serde(rename = "skills")]
    skill_folders: Vec<std::path::PathBuf>,
}
impl Project {
    /// returns the current exo pointed by the state
    /// This can fail if for instance the skills/exos were moved/deleted between two plx runs
    pub fn resume(&mut self) -> Option<&Exo> {
        if self.state.curr_skill_idx < self.skills.len()
            && self.state.curr_exo_idx < self.skills[self.state.curr_skill_idx].exos.len()
        {
            return Some(&self.skills[self.state.curr_skill_idx].exos[self.state.curr_exo_idx]);
        }
        None
    }
    fn is_first_skill(&self) -> bool {
        self.state.curr_skill_idx == 0
    }
    fn is_first_exo(&self) -> bool {
        self.state.curr_exo_idx == 0
    }

    fn is_last_skill(&self) -> bool {
        self.state.curr_skill_idx == self.skills.len() - 1
    }
    fn is_last_exo(&self) -> bool {
        self.state.curr_exo_idx == self.skills[self.state.curr_skill_idx].exos.len() - 1
    }

    /// Saves state to file
    fn save_state(&self) {
        if let Err(err) = write_object_to_file(&self.folder.join(COURSE_STATE_FILE), &self.state) {
            warn!("Couldn't store project state {:?}", err);
        }
    }

    /// Sets current exo index
    /// This function should be used instead of assigning to curr_exo directly as it handles
    /// storage of the project state
    fn set_curr_exo(&mut self, i: usize) {
        // Checking so we don't save the state unnecessarily
        if self.state.curr_exo_idx != i {
            self.state.curr_exo_idx = i;
            self.save_state();
        }
    }

    /// Sets current skill index
    /// This function should be used instead of assigning to curr_skill directly as it handles
    /// storage of the project state
    fn set_curr_skill(&mut self, i: usize) {
        // Checking so we don't save the state unnecessarily
        if self.state.curr_skill_idx != i {
            self.state.curr_skill_idx = i;
            self.save_state();
        }
    }

    /// Change to the previous exo
    /// if we are already at the first exo in a skill and if wrap is set,
    /// it will search for the last exo in the previous skill
    pub fn prev_exo(&mut self, wrap: bool) {
        if !self.is_first_exo() {
            self.state.curr_exo_idx -= 1
        } else if wrap {
            if !self.is_first_skill() {
                self.set_curr_skill(self.state.curr_skill_idx - 1);
            } else {
                self.set_curr_skill(self.skills.len() - 1);
            }
            self.set_curr_exo(self.skills[self.state.curr_skill_idx].exos.len() - 1);
        }
    }

    /// Change to the previous skill
    /// if we are already at the first skill if wrap is set,
    /// it will go back go to the last skill
    pub fn prev_skill(&mut self, wrap: bool) {
        if self.is_first_skill() {
            if wrap {
                self.set_curr_skill(self.skills.len() - 1);
            }
        } else {
            self.set_curr_skill(self.state.curr_skill_idx - 1);
        }
        self.set_curr_exo(0);
    }

    /// Change to the next exo
    /// if we are already at the last exo in a skill and if wrap is set,
    /// it will search for the first exo in the next skill
    pub fn next_exo(&mut self, wrap: bool) {
        if !self.is_last_exo() {
            self.state.curr_exo_idx += 1
        } else if wrap {
            self.set_curr_exo(0);
            if !self.is_last_skill() {
                self.set_curr_skill(self.state.curr_skill_idx + 1);
            } else {
                self.set_curr_skill(0);
            }
        }
    }

    /// Change to the next skill
    /// if we are already at the last skill if wrap is set,
    /// it will wrap to the first skill
    pub fn next_skill(&mut self, wrap: bool) {
        if !self.is_last_skill() {
            self.set_curr_skill(self.state.curr_skill_idx + 1);
        } else if wrap {
            self.set_curr_skill(0);
        }
        self.set_curr_exo(0);
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
        let mut info = Project::read_exo_state_info(exo);
        info.state = state;
        Project::save_exo_state(exo, &info);
    }
    // Set exo as favorite or not and store it in file
    pub fn set_exo_favorite(exo: &Exo, is_favorite: bool) {
        let mut info = Project::read_exo_state_info(exo);
        info.favorite = is_favorite;
        Project::save_exo_state(exo, &info);
    }
}

impl FromDir for Project {
    ///
    /// Tries to build a project from dir
    /// Returns Ok if we were able to parse the project info and at least one skill
    /// else Error
    ///
    fn from_dir(
        dir: &std::path::PathBuf,
    ) -> Result<(Self, Vec<ParseWarning>), (ParseError, Vec<ParseWarning>)> {
        // Get course info by searching for the course.toml file
        // TODO magic value maybe change this
        let course_info_file = dir.join(COURSE_INFO_FILE);
        let course_state_file = dir.join(COURSE_STATE_FILE);
        let course_info = object_creator::create_object_from_file::<ProjectInfo>(&course_info_file)
            .map_err(|err| (err, vec![]))?;
        let project_state =
            object_creator::create_object_from_file::<ProjectState>(&course_state_file)
                .unwrap_or_default();

        // Using the skill folders found in the course.toml file, parse every skill
        // /!\ Folders not found in the course.toml file are ignored /!\
        // TODO maybe warn if there are folder that aren't included in course.toml ?
        let mut warnings = Vec::new();
        let skills = course_info
            .skill_folders
            .iter()
            .filter_map(
                |skill_folder| match Skill::from_dir(&dir.join(skill_folder)) {
                    Ok((skill, mut skill_warnings)) => {
                        warnings.append(&mut skill_warnings);
                        Some(skill)
                    }
                    Err(error) => {
                        warnings.push(ParseWarning::ParseSkillFail(format!(
                            "Couldn't parse skill in {:?}: {:?}",
                            skill_folder, error
                        )));
                        None
                    }
                },
            )
            .collect::<Vec<Skill>>();

        if skills.is_empty() {
            Err((
                ParseError::ErrorParsingSkills(format!(
                    "Couldn't parse any skill folders in {:?}",
                    dir
                )),
                warnings,
            ))
        } else {
            Ok((
                Self {
                    name: course_info.name,
                    skills: Arc::new(skills),
                    state: project_state,
                    folder: dir.to_path_buf(),
                },
                warnings,
            ))
        }
    }
}

#[cfg(test)]
mod tests {

    use std::{str::FromStr, sync::Arc};

    use crate::models::{
        check::{Check, CheckTest},
        exo::Exo,
        exo_state::ExoState,
    };

    use super::*;

    #[test]
    fn test_example_full() {
        let project_path = std::path::PathBuf::from_str("examples/full").unwrap();
        let ret = Project::from_dir(&project_path);

        println!("{:#?}", ret);
        assert!(ret.is_ok());
        let (_project, warnings) = ret.unwrap();
        assert!(warnings.len() < 2);
    }
    #[test]
    fn test_full_hierarchy() {
        let project_path = std::path::PathBuf::from_str("examples/mock-plx-project").unwrap();
        let project = Project::from_dir(&project_path);
        let expected  = Project {
            name: String::from("Full fictive course"),
            folder: project_path.clone(),
            skills: Arc::new(vec![
                Skill {
                    name: String::from("Introduction"),
                    path: project_path.join("intro"),
                    exos: Arc::new(vec![
                        Exo {
                            name: String::from("Basic arguments usage"),
                            instruction: Some(
                                String::from("The 2 first program arguments are the firstname and number of legs of a dog. Print a full sentence about the dog. Make sure there is at least 2 arguments, print an error if not."),
                            ),
                            folder: "examples/mock-plx-project/intro/basic-args".into(),
                            state: ExoState::Todo,
                            files: vec![
                               project_path.join("intro").join("basic-args").join("main.c"),
                            ],
                            solutions: vec![
                                project_path.join("intro").join("basic-args").join("main.sol.c").into(),
                            ],
                            checks: vec![
                                Check {
                                    name: String::from("Joe + 5 legs"),
                                    args: vec![
                                        String::from("Joe"),
                                        String::from("5"),
                                    ],
                                    test: CheckTest::Output{expected: String::from("The dog is Joe and has 5 legs")},
                                },
                                Check {
                                    name: String::from("No arg -> error"),
                                    args: vec![],
                                    test: CheckTest::Output{ expected : String::from("Error: missing argument firstname and legs number")},
                                },
                                Check {
                                    name: String::from("One arg -> error"),
                                    args: vec![
                                        String::from("Joe"),
                                    ],
                                    test: CheckTest::Output {expected : String::from("Error: missing argument firstname and legs number")},
                                },
                            ],
                            favorite: false,
                        },
                        Exo {
                            name: String::from("Basic output printing"),
                            folder: "examples/mock-plx-project/intro/basic-output".into(),
                            instruction: Some(
                                String::from("Just print 2 lines"),
                            ),
                            state: ExoState::Todo,
                            files: vec![
                               project_path.join("intro").join("basic-output").join("main.c"),
                            ],
                            solutions: vec![
                               project_path.join("intro").join("basic-output").join("main.sol.c"),
                            ],
                            checks: vec![
                                Check {
                                    name: String::from("Lines are correct"),
                                    args: vec![],
                                    test: CheckTest::Output{ expected: String::from("PLX is amazing !\nThis is a neutral opinion...\n")},
                                },
                            ],
                            favorite: false,
                        },
                    ]),
                },
            ]),
            state:ProjectState{curr_exo_idx: 0, curr_skill_idx:0}
        };
        let (actual, warnings) = project.unwrap();
        assert_eq!(expected, actual);
        assert!(matches!(warnings[0], ParseWarning::ParseSkillFail(_)));
    }
}
