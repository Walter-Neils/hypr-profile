mod hypr_config;
mod hypr_locations;
use anyhow::{anyhow, Result};
use clap::{Arg, ArgAction, ArgMatches, Command};
use hypr_config::*;
use hypr_locations::*;
use hyprland::keyword::Keyword;
use log::{error, warn};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::process::exit;

fn main() {
    env_logger::init();
    match command_line_args().subcommand() {
        None => {
            println!("Specify a command, or use --help");
            exit(1);
        }
        Some(("apply", matches)) => subcommand_apply(&matches),
        Some(("list", matches)) => subcommand_list(&matches),
        Some((&_, _)) => panic!("Unhandled"),
    }
}

fn subcommand_list(_matches: &ArgMatches) {
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

fn subcommand_apply(matches: &ArgMatches) {
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
    let profile_vars = match load_config_from_profile(ProfileIdentifier::ByName(target.to_owned()))
    {
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
