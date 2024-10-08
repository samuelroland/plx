use super::{
    check::Check,
    constants::{EXO_INFO_FILE, EXO_STATE_FILE},
    exo_state::ExoState,
};
use serde::{Deserialize, Serialize};

use crate::core::{
    compiler::compiler::Compiler,
    file_utils::{
        file_parser::{ParseError, ParseWarning},
        file_utils::list_dir_files,
    },
    parser::{self, from_dir::FromDir},
};

/// Contains the exo info that can be found in exo.toml
#[derive(Serialize, Deserialize, Debug)]
struct ExoInfo {
    name: String,
    instruction: Option<String>,
    #[serde(default)]
    checks: Vec<Check>,
}

/// Contains the exo state info that can be found in .exo-state.toml
#[derive(Serialize, Deserialize, Debug, Default)]
pub(super) struct ExoStateInfo {
    pub(super) state: ExoState,
    pub(super) favorite: bool,
}

/// Represents a Plx Exo
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Exo {
    pub(crate) name: String,
    pub(crate) instruction: Option<String>,
    pub(crate) state: ExoState,
    pub(crate) files: Vec<std::path::PathBuf>,
    pub(crate) solutions: Vec<std::path::PathBuf>,
    pub(crate) checks: Vec<Check>,
    pub(crate) favorite: bool,
    pub(crate) folder: std::path::PathBuf,
}
impl FromDir for Exo {
    /// Tries to build an exo from dir
    /// Returns Ok if we were able to get the exo info and at least 1 exo file
    /// else Error
    fn from_dir(
        dir: &std::path::PathBuf,
    ) -> Result<(Self, Vec<ParseWarning>), (ParseError, Vec<ParseWarning>)> {
        // Get the exo info and the state if it exists.
        let mut warnings = Vec::new();
        let exo_info_file = dir.join(EXO_INFO_FILE);
        let exo_state_file = dir.join(EXO_STATE_FILE);
        let exo_info = parser::object_creator::create_object_from_file::<ExoInfo>(&exo_info_file)
            .map_err(|err| (err, vec![]))?;

        // If the exo hasn't been started, the state file won't exist
        let exo_state =
            parser::object_creator::create_object_from_file::<ExoStateInfo>(&exo_state_file)
                .unwrap_or_default();

        // Get all the dir files and find the exo and solution files
        let files = list_dir_files(&dir)
            .map_err(|err| (ParseError::FileDiscoveryFailed(err.to_string()), vec![]))?;
        let (exo_files, solution_files) = Exo::find_exo_and_solution_files(files);

        if exo_files.is_empty() {
            return Err((ParseError::NoExoFilesFound(dir.to_path_buf()), vec![]));
        }
        if solution_files.is_empty() {
            warnings.push(ParseWarning::NoSolutionFile(format!(
                "No solution found in {:?}",
                dir
            )));
        }

        Exo::check_exo_solutions(&exo_files, &solution_files, &mut warnings);

        Ok((
            Self {
                name: exo_info.name,
                instruction: exo_info.instruction,
                checks: exo_info.checks,
                state: exo_state.state,
                files: exo_files,
                favorite: exo_state.favorite,
                solutions: solution_files,
                folder: dir.to_path_buf(),
            },
            warnings,
        ))
    }
}
impl Exo {
    /// Finds exo and solution from a bunch of folder files
    fn find_exo_and_solution_files(
        files: Vec<std::path::PathBuf>,
    ) -> (Vec<std::path::PathBuf>, Vec<std::path::PathBuf>) {
        let mut exo_files = Vec::new();
        let mut solution_files = Vec::new();
        for file_path in files {
            let file_path_str = file_path.display().to_string();
            let file_extension = file_path
                .extension()
                .and_then(|extension| extension.to_str())
                .unwrap_or_default();

            // Ignore our files
            if file_extension == "toml" {
                continue;
            }
            if file_path_str.contains(".sol.") {
                solution_files.push(file_path);
                continue;
            }
            // TODO maybe make sure we don't mix .c with .java files here ?
            // We need to be careful adding this because .c can be mixed with .cpp, .h,
            // .hpp etc...
            exo_files.push(file_path);
        }
        (exo_files, solution_files)
    }
    // Check every solution file and check that it corresponds to an actual exo file
    fn check_exo_solutions(
        exo_files: &Vec<std::path::PathBuf>,
        solution_files: &Vec<std::path::PathBuf>,
        warnings: &mut Vec<ParseWarning>,
    ) {
        if solution_files.is_empty() {
            return;
        }
        for solution_file in solution_files {
            //try to get solution file name and solution last extension
            match (solution_file.file_stem(), solution_file.extension()) {
                (Some(file_name), Some(extension)) => {
                    //try to parse file name to string
                    match (file_name.to_str(), extension.to_str()) {
                        (Some(file_name), Some(extension)) => {
                            // associated exo file should be of format <file_name>.<extension>
                            // This essentially removes the .sol part
                            let exo_target_name =
                                format!("{}.{}", file_name.replace(".sol", ""), extension);
                            let exo_exists = exo_files
                                .iter()
                                .find(|exo_file| {
                                    if let Some(exo_file_name) = exo_file.file_name() {
                                        *exo_file_name == *exo_target_name
                                    } else {
                                        false
                                    }
                                })
                                .is_some();

                            if !exo_exists {
                                warnings.push(ParseWarning::ExoFileNotFound(format!(
                                    "Solution file {:?} doesn't have an exo associated with it (expected exo file {:?})",
                                    solution_file, exo_target_name)))
                            }
                        }
                        (_, _) => warnings.push(ParseWarning::InvalidFileName(format!(
                            "{:?}",
                            solution_file
                        ))),
                    }
                }
                (_, _) => warnings.push(ParseWarning::InvalidFileName(format!(
                    "{:?}",
                    solution_file
                ))),
            }
        }
    }

    /// Tries to find a `main` file or returns the first file in the list of exo files
    pub fn get_main_file(&self) -> Option<&std::path::PathBuf> {
        match self.files.iter().find(|file| {
            if let Some(file_name) = file.file_stem() {
                return file_name == "main";
            }
            return false;
        }) {
            Some(file) => Some(file),
            None => self.files.first(),
        }
    }

    /// Computes the required compiler based on the file extension
    pub fn compiler(&self) -> Option<Compiler> {
        let mut compiler = None;
        for file in &self.files {
            let extension = file
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            if extension == "cpp" || extension == "cc" {
                compiler = Some(Compiler::Gxx);
                break;
            } else if extension == "c" {
                compiler = Some(Compiler::Gcc);
            }
        }
        compiler
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::models::check::CheckTest;

    use super::*;

    #[test]
    fn test_parse_full_intro_basic_args() {
        let file_path = "examples/mock-plx-project/intro/basic-args";

        let expected = Exo {
        name: String::from("Basic arguments usage"),
        instruction: Some(
                String::from(
                    "The 2 first program arguments are the firstname and number of legs of a dog. Print a full sentence about the dog. Make sure there is at least 2 arguments, print an error if not.")
            ),
        state: ExoState::Todo,
        folder: file_path.into(),
        files: vec![
            std::path::PathBuf::from_str(file_path)
                .unwrap()
                .join("main.c")],

        solutions: vec![
            std::path::PathBuf::from_str(file_path)
                .unwrap()
                .join("main.sol.c")],
        checks: vec![
            Check {
                name: String::from("Joe + 5 legs"),
                args: vec![
                    String::from("Joe"),
                    String::from("5"),
                ],
                test: CheckTest::Output {expected : String::from("The dog is Joe and has 5 legs")},
            },
            Check {
                name: String::from("No arg -> error"),
                args: vec![],
                test: CheckTest::Output {expected: String::from("Error: missing argument firstname and legs number")},
            },
            Check {
                name: String::from("One arg -> error"),
                args: vec![
                    String::from("Joe"),
                ],
                test: CheckTest::Output {expected: String::from("Error: missing argument firstname and legs number")},
            },
        ],
        favorite: false,
    };
        assert_eq!(
            expected,
            Exo::from_dir(&(file_path.into()))
                .expect("Couldn't parse file")
                .0
        );
    }
    #[test]
    fn test_exo_done() {
        let file_path = "examples/mock-plx-project/mock-skill/exo-done";
        let (exo, _warnings) = Exo::from_dir(&(file_path.into())).unwrap();
        let expected = Exo {
            name: String::from("Exo Done"),
            instruction: None,
            folder: file_path.into(),
            checks: vec![],
            files: vec![std::path::PathBuf::from_str(file_path)
                .unwrap()
                .join("main.c")],
            favorite: false,
            state: ExoState::Done,
            solutions: vec![],
        };
        assert_eq!(expected, exo);
        println!("{:#?}", exo);
    }
    #[test]
    fn test_exo_favorite() {
        let file_path = "examples/mock-plx-project/mock-skill/exo-favorite";
        let (exo, _warnings) = Exo::from_dir(&(file_path.into())).unwrap();
        let expected = Exo {
            name: String::from("Favorite Exercise"),
            instruction: None,
            checks: vec![],
            folder: file_path.into(),
            files: vec![std::path::PathBuf::from_str(file_path)
                .unwrap()
                .join("main.c")],
            favorite: true,
            state: ExoState::Todo,
            solutions: vec![],
        };
        assert_eq!(expected, exo);
        println!("{:#?}", exo);
    }

    #[test]
    fn test_exo_in_progress() {
        let file_path = "examples/mock-plx-project/mock-skill/exo-in-progress";
        let (exo, _warnings) = Exo::from_dir(&(file_path.into())).unwrap();
        let expected = Exo {
            name: String::from("In Progress"),
            instruction: None,
            checks: vec![],
            folder: file_path.into(),
            files: vec![std::path::PathBuf::from_str(file_path)
                .unwrap()
                .join("main.c")],
            favorite: false,
            state: ExoState::InProgress,
            solutions: vec![],
        };
        assert_eq!(expected, exo);
        println!("{:#?}", exo);
    }

    #[test]
    fn test_exo_multiple_sols() {
        let file_path = "examples/mock-plx-project/mock-skill/multiple-sols";
        let sol_files = vec![
            std::path::PathBuf::from_str(file_path)
                .unwrap()
                .join("main.sol.c"),
            std::path::PathBuf::from_str(file_path)
                .unwrap()
                .join("whyisthishere.sol.c"),
        ];
        let (exo, warnings) = Exo::from_dir(&(file_path.into())).unwrap();
        let expected = Exo {
            name: String::from("Multiple Sols"),
            instruction: None,
            folder: file_path.into(),
            checks: vec![],
            files: vec![std::path::PathBuf::from_str(file_path)
                .unwrap()
                .join("main.c")],
            favorite: false,
            state: ExoState::Todo,
            solutions: sol_files.clone(),
        };
        assert_eq!(expected.solutions.len(), exo.solutions.len());
        assert_eq!(warnings.len(), 1);
        assert!(matches!(warnings[0], ParseWarning::ExoFileNotFound(_)));
    }
    #[test]
    fn test_no_exo_info() {
        let file_path = "examples/mock-plx-project/mock-skill/no-exo-info";
        let ret = Exo::from_dir(&(file_path.into()));
        assert!(ret.is_err());

        let err = match ret {
            Ok(_) => panic!("Exo can't be constructed with no exo info"),
            Err((error, _warnings)) => error,
        };
        assert!(matches!(err, ParseError::ReadFileError(_)));
    }
    #[test]
    fn test_no_files() {
        let file_path = "examples/mock-plx-project/mock-skill/no-files";
        let ret = Exo::from_dir(&(file_path.into()));
        assert!(ret.is_err());

        let err = match ret {
            Ok(_) => panic!("Exo can't be constructed with no files"),
            Err((error, _warnings)) => error,
        };
        assert!(matches!(err, ParseError::NoExoFilesFound(_)));
    }
    #[test]
    fn test_no_solution() {
        let file_path = "examples/mock-plx-project/mock-skill/no-sol";
        let (exo, warnings) = Exo::from_dir(&(file_path.into())).unwrap();
        let expected = Exo {
            name: String::from("No Sol"),
            folder: file_path.into(),
            instruction: None,
            checks: vec![],
            files: vec![std::path::PathBuf::from_str(file_path)
                .unwrap()
                .join("main.c")],
            favorite: false,
            state: ExoState::Todo,
            solutions: vec![],
        };
        assert_eq!(expected, exo);
        assert_eq!(warnings.len(), 1);
        assert!(matches!(warnings[0], ParseWarning::NoSolutionFile(_)));
    }
}
