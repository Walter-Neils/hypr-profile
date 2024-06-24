use std::fs;
use std::path::PathBuf;

use crate::environment_string_replace::EnvStringReplace;

pub struct HyprConfigObject {
    pub key: String,
    pub value: String,
}

impl HyprConfigObject {
    pub fn collection_from_vector(input: Vec<&str>) -> Vec<Self> {
        let mut result = Vec::new();

        // TODO: Make this global / configurable
        let env_replacer = EnvStringReplace::new(true);

        let mut scope = Vec::new();

        for line in input {
            let trimmed_line = line.trim();
            let allow_macro_expansion = trimmed_line.starts_with("#!");
            let trimmed_line = match allow_macro_expansion {
                true => &trimmed_line[2..],
                false => trimmed_line,
            };

            if trimmed_line.starts_with("#") {
                continue;
            }
            if trimmed_line.ends_with("{") {
                let new_scope = trimmed_line[0..(trimmed_line.len() - 1)].to_owned();
                scope.push(new_scope);
            }
            if trimmed_line.starts_with("}") {
                scope.pop();
            }
            if !trimmed_line.contains("=") {
                continue;
            }
            let index = trimmed_line.find("=").unwrap();
            let key = &trimmed_line[0..(index)].trim();
            let value = match &trimmed_line.find("#") {
                None => &trimmed_line[index + 1..],
                Some(position) => &trimmed_line[index + 1..*position],
            }
            .trim();
            let full_key = {
                let mut full_key = "".to_owned();
                for scope_level in scope.iter().rev() {
                    full_key += scope_level.trim();
                    full_key += ":";
                }
                full_key += key;
                full_key
            };
            result.push(HyprConfigObject {
                key: full_key,
                value: match allow_macro_expansion {
                    false => value.to_owned(),
                    true => env_replacer.apply(&value.to_owned()),
                },
            })
        }

        result
    }

    pub fn collection_from_file(path: impl Into<PathBuf>) -> Vec<Self> {
        let raw_string = fs::read_to_string(path.into()).unwrap();
        let contents = raw_string.split("\n").collect();
        HyprConfigObject::collection_from_vector(contents)
    }
}
