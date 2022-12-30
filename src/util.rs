#![allow(dead_code)]

use anyhow::Result;
use regex::Regex;
use std::{fs::File, io::Read, path::Path, str::FromStr};

pub fn get_env_str(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

pub fn get_env_bool(name: &str) -> Option<bool> {
    const TRUE_VALUES: &[&str] = &["true", "t", "yes", "y", "1"];
    const FALSE_VALUES: &[&str] = &["false", "f", "no", "n", "0"];
    match get_env_str(name) {
        Some(str_res) if TRUE_VALUES.contains(&str_res.to_lowercase().as_ref()) => Some(true),
        Some(str_res) if FALSE_VALUES.contains(&str_res.to_lowercase().as_ref()) => Some(false),
        _ => None,
    }
}

pub fn get_env<V>(name: &str) -> Option<V>
where
    V: FromStr,
{
    match get_env_str(name) {
        Some(str_res) => match str_res.parse::<V>() {
            Ok(res) => Some(res),
            Err(_) => None,
        },
        None => None,
    }
}

pub fn read_file_string(path: &str) -> Result<String> {
    let mut contents = String::new();
    let mut file = File::open(Path::new(path))?;
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn safe_filename(filename: &str) -> String {
    let invalid = Regex::new(r#"[?|_|*|<|>|\|、|/|"]"#).unwrap();

    invalid.replace_all(filename, " ").to_string()
}


#[test]
fn test_filename() {
    assert_eq!("ABC123 q w e r t y u i o".to_string(), safe_filename("ABC123?q_w*e<r>t|y\"u、i/o"));
}