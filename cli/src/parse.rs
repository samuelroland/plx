use std::{
    ffi::OsString,
    fs::read_to_string,
    path::{Path, PathBuf},
    process::exit,
};

use plx_core::{
    core::{file_utils::file_parser::ParseError, parser::from_dir::FromDir},
    dy::{ParseResult, error},
    models::course::Course,
};
use plx_dy::{COURSE_FILE, SKILLS_FILE, parse_course, parse_exos, parse_skills};
use serde::Serialize;

/// The "parse" subcommand implementation
pub fn parse_command(path: PathBuf) -> Result<(), std::io::Error> {
    if path.is_dir() {
        let course_file_info = path.join(COURSE_FILE);
        if course_file_info.exists() {
            parse_course_file(&course_file_info)?;
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "File {COURSE_FILE} doesn't exist under {}",
                    path.to_string_lossy()
                ),
            ));
        }
    } else {
        let filename = path.file_name();
        if filename == Some(&OsString::from(COURSE_FILE)) {
            parse_course_file(&path)?;
        } else if filename == Some(&OsString::from(SKILLS_FILE)) {
            parse_skills_file(&path)?;
        } else {
            parse_exo_file(&path)?;
        }
    }
    Ok(())
}

fn parse_course_file(course_file_path: &Path) -> Result<(), std::io::Error> {
    let course_file_content = read_to_string(course_file_path)?;
    let dy_result = parse_course(
        &Some(course_file_path.to_str().unwrap_or_default().to_string()),
        &course_file_content,
    );
    print_result(dy_result, true)
}

fn parse_skills_file(skills_file_path: &Path) -> Result<(), std::io::Error> {
    let file_content = read_to_string(skills_file_path)?;
    let dy_result = parse_skills(
        &Some(skills_file_path.to_str().unwrap_or_default().to_string()),
        &file_content,
    );
    print_result(dy_result, false)
}

fn parse_exo_file(exo_file_path: &Path) -> Result<(), std::io::Error> {
    let file_path = exo_file_path;
    let file_content = read_to_string(file_path)?;

    let dy_result = parse_exos(
        &Some(file_path.to_str().unwrap_or_default().to_string()),
        &file_content,
    );
    print_result(dy_result, true)
}

// TODO use it
fn parse_entire_course(dir: &PathBuf) -> Result<(Vec<error::ParseError>, Course), ParseError> {
    Course::from_dir(dir, true)
}

fn print_result<T>(dy_result: ParseResult<T>, single_element: bool) -> Result<(), std::io::Error>
where
    T: Serialize,
{
    if dy_result.errors.is_empty() && !dy_result.items.is_empty() {
        // Print on standard output the single line of success, so it's separated from JSON output
        eprintln!("{dy_result}");
        if single_element {
            println!("{}", serde_json::to_string_pretty(&dy_result.items[0])?);
        } else {
            println!("{}", serde_json::to_string_pretty(&dy_result.items)?);
        };
    } else {
        eprintln!("{dy_result}");
        exit(2);
    }
    Ok(())
}
