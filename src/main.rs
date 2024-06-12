use anyhow::{anyhow, Result};
use clap::{Arg, ArgAction, Command};
use hyprland::keyword::Keyword;
use log::{debug, error, warn};
use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::exit;
struct HyprConfigObject {
    unscoped_key: String,
    value: String,
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
                unscoped_key: full_key,
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

enum ProfileIdentifier {
    ByName(String),
}

fn load_config_from_profile(ident: ProfileIdentifier) -> Result<Vec<HyprConfigObject>> {
    match ident {
        ProfileIdentifier::ByName(name) => {
            let path = get_profiles_directory().join(name + ".conf");
            if !path.exists() {
                return Err(anyhow!("Profile does not exist"));
            }
            Ok(HyprConfigObject::collection_from_file(path))
        }
    }
}

fn get_profiles_directory() -> std::path::PathBuf {
    match std::env::var("HYPR_PROFILES_DIR") {
        Ok(x) => {
            debug!("$HYPR_PROFILES_DIR is set, using it!");
            x.to_owned()
        }
        Err(_) => {
            warn!("$HYPR_PROFILES_DIR is not set. Defaulting to ~/.config/hypr/profiles/");
            std::env::var("HOME").unwrap() + "/.config/hypr/profiles/"
        }
    }
    .into()
}

fn get_hypr_profile_persistent_profile() -> std::path::PathBuf {
    match std::env::var("HYPR_PERSIST_PROFILE_FILE") {
        Ok(x) => x.to_owned(),
        Err(_) => {
            std::env::var("HOME").unwrap() + "/.config/hypr/profiles/.hypr_persistant_profile.conf"
        }
    }
    .into()
}

fn command_line_args() -> clap::ArgMatches {
    Command::new("hypr-profile")
        .subcommand(
            Command::new("apply")
                .arg(Arg::new("target").required(true).help("The target profile to apply, stored in $HYPR_PROFILES_DIR"))
                .arg(
                    Arg::new("persist")
                        .long("persist")
                        .short('p')
                        .action(ArgAction::SetTrue).help("If applied, profile values will be written to $HYPR_PERSIST_PROFILE_FILE"),
                )
                .arg(Arg::new("append").short('a').action(ArgAction::SetTrue).help("If applied, profile values will be APPENDED to $HYPR_PERSIST_PROFILE_FILE. Only valid when --persist is applied.")),
        )
        .subcommand(Command::new("list"))
        .get_matches()
}

fn main() {
    env_logger::init();
    let args = command_line_args();

    match args.subcommand() {
        None => {
            println!("Specify a command, or use --help");
            exit(1);
        }
        Some(("apply", matches)) => {
            let mut persistant_output_handle = {
                if matches.get_one::<bool>("persist").is_some() {
                    let append = {
                        match matches.get_one::<bool>("append") {
                            None => false,
                            Some(x) => *x,
                        }
                    };
                    let truncate = !append;
                    match OpenOptions::new()
                        .append(append)
                        .truncate(truncate)
                        .write(true)
                        .create(true)
                        .open(get_hypr_profile_persistent_profile())
                    {
                        Ok(x) => Some(x),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            };

            let target = matches.get_one::<String>("target").unwrap();
            let profile_vars =
                match load_config_from_profile(ProfileIdentifier::ByName(target.to_owned())) {
                    Ok(x) => x,
                    Err(_) => {
                        error!("Failed to load configuration '{}': file not found", target);
                        exit(1);
                    }
                };
            for var in profile_vars {
                if var.unscoped_key == "env" {
                    warn!("'env' directives have no effect");
                }
                match Keyword::set(&var.unscoped_key, var.value.clone()) {
                    Ok(_) => (),
                    Err(_) => error!("Failed to apply value for keyword '{}'", &var.unscoped_key),
                }
                match persistant_output_handle {
                    None => (),
                    Some(ref mut handle) => {
                        if let Err(_) = writeln!(handle, "{}={}", var.unscoped_key, var.value) {
                            error!("Failed to write to persistant profile!");
                        }
                    }
                }
            }
        }
        Some(("list", _)) => {
            let profiles_directory = get_profiles_directory();
            let dir_iter = match std::fs::read_dir(&profiles_directory) {
                Ok(x) => x,
                Err(_) => {
                    error!(
                        "Cannot read profiles directory ({})",
                        profiles_directory.to_str().unwrap()
                    );
                    exit(1);
                }
            };
            for possible_path in dir_iter {
                if let Ok(path) = possible_path {
                    let full_file_name = path.file_name().into_string().unwrap();
                    if !full_file_name.contains(".conf") {
                        continue;
                    }

                    if full_file_name.starts_with(".") {
                        continue; // Dot files are hidden
                    }

                    let profile_name = full_file_name.replace(".conf", "");
                    println!("{}", profile_name);
                }
            }
        }
        Some((&_, _)) => panic!("Unhandled"),
    }
}
