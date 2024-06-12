use std::fs;
use std::path::PathBuf;

pub struct HyprConfigObject {
    pub key: String,
    pub value: String,
}

impl HyprConfigObject {
    pub fn collection_from_vector(input: Vec<&str>) -> Vec<Self> {
        let mut result = Vec::new();

        let mut scope = Vec::new();

        for line in input {
            let trimmed_line = line.trim();
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
                value: value.to_owned(),
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
