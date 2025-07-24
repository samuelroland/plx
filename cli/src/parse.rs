use std::{
    ffi::OsString,
    fs::read_to_string,
    path::{Path, PathBuf},
    process::exit,
};

use plx_core::{core::parser::from_dir::FromDir, dy::ParseResult, models::course::Course};
use plx_dy::{COURSE_FILE, EXO_FILE, SKILLS_FILE, parse_course, parse_exo, parse_skills};
use serde::Serialize;

/// The "parse" subcommand implementation
pub fn parse_command(path: PathBuf, full: bool) -> Result<(), std::io::Error> {
    if path.is_dir() {
        let course_file_info = path.join(COURSE_FILE);
        if course_file_info.exists() {
            if full {
                parse_and_show_full_plx_course(&path)?;
            } else {
                parse_and_show_course_file(&course_file_info)?;
            }
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
            parse_and_show_course_file(&path)?;
        } else if filename == Some(&OsString::from(SKILLS_FILE)) {
            parse_and_show_skills_file(&path)?;
        } else if filename == Some(&OsString::from(EXO_FILE)) {
            parse_and_show_exo_file(&path)?;
        } else {
            eprintln!(
                "Invalid file given, only {COURSE_FILE}, {SKILLS_FILE} or {EXO_FILE} can be parsed."
            );
            exit(1);
        }
    }
    Ok(())
}

fn parse_and_show_course_file(course_file_path: &Path) -> Result<(), std::io::Error> {
    let course_file_content = read_to_string(course_file_path)?;
    let dy_result = parse_course(
        &Some(course_file_path.to_str().unwrap_or_default().to_string()),
        &course_file_content,
    );
    print_result(dy_result, true)
}

fn parse_and_show_full_plx_course(path: &PathBuf) -> Result<(), std::io::Error> {
    match Course::from_dir(path, true) {
        Ok((errors, course)) => {
            if errors.is_empty() {
                println!("{}", serde_json::to_string_pretty(&course)?);
            } else {
                eprintln!("Not better errors output for now sorry...");
                dbg!(errors); // TODO: better output when ParseResult available from FromDir ??
                eprintln!("Parsed full course");
                println!("{}", serde_json::to_string_pretty(&course)?);
            }
        }
        Err(e) => {
            eprintln!("{e}");
            exit(1);
        }
    }
    Ok(())
}

fn parse_and_show_skills_file(skills_file_path: &Path) -> Result<(), std::io::Error> {
    let file_content = read_to_string(skills_file_path)?;
    let dy_result = parse_skills(
        &Some(skills_file_path.to_str().unwrap_or_default().to_string()),
        &file_content,
    );
    print_result(dy_result, false)
}

fn parse_and_show_exo_file(exo_file_path: &Path) -> Result<(), std::io::Error> {
    let file_path = exo_file_path;
    let file_content = read_to_string(file_path)?;

    let dy_result = parse_exo(
        &Some(file_path.to_str().unwrap_or_default().to_string()),
        &file_content,
    );
    print_result(dy_result, true)
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
