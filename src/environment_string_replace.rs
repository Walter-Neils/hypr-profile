use std::{collections::HashMap, env};

use regex::Regex;

pub struct EnvStringReplace {
    use_system_enironment_variables: bool,
    custom_variables: HashMap<String, String>,
}

impl EnvStringReplace {
    pub fn new(use_system_enironment_variables: bool) -> Self {
        EnvStringReplace {
            use_system_enironment_variables,
            custom_variables: HashMap::new(),
        }
    }

    fn get_var_value(&self, key: &str) -> Result<String, ()> {
        match self.custom_variables.get(key) {
            Some(val) => Ok(val.to_owned()),
            None => {
                if !self.use_system_enironment_variables {
                    Err(())
                } else {
                    match env::var(key) {
                        Ok(x) => Ok(x),
                        Err(_) => Err(()),
                    }
                }
            }
        }
    }

    pub fn apply(&self, target: &String) -> String {
        let mut result: String = target.clone();

        let var_regex = Regex::new(r"\$\{(?P<VAR>.+?)\}").unwrap();

        let mut targets: Vec<String> = Vec::new();

        for (_, [var]) in var_regex.captures_iter(target).map(|c| c.extract()) {
            targets.push(var.to_owned());
        }

        for target in targets {
            let formatted_target = format!("${{{}}}", target);
            let value = self.get_var_value(target.as_str());
            match value {
                Err(_) => {
                    result = result.replace(&formatted_target, &"".to_owned());
                }
                Ok(value) => {
                    result = result.replace(&formatted_target, &value);
                }
            }
        }

        result
    }
}
