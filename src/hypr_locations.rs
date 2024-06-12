use log::{debug, warn};
pub fn get_profiles_directory() -> std::path::PathBuf {
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

pub fn get_hypr_profile_persistent_profile() -> std::path::PathBuf {
    match std::env::var("HYPR_PERSIST_PROFILE_FILE") {
        Ok(x) => x.to_owned(),
        Err(_) => {
            std::env::var("HOME").unwrap() + "/.config/hypr/profiles/.hypr_persistant_profile.conf"
        }
    }
    .into()
}
