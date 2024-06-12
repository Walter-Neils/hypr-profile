# `hypr-profile`
`hypr-profile` is a small application for dynamically applying a set of configuration values to your hyprland session. These changes only persist for the duration of the session, and never effect your configuration files.

## Environment Variables
- `$HYPR_PROFILES_DIR` determines where `hypr-profile` looks for profiles
  - Defaults to `~/.config/hypr/profiles/`
- `$HYPR_PERSIST_PROFILE_FILE` determines the location of a file which can be used to persist changes to the hyprland session through sessions.
  - Defaults to `~/.config/hypr/profiles/.hypr_persistant_profile.conf`

## Options
- `hypr-profile apply $PROFILE_NAME` applies a profile found in `$HYPR_PROFILES_DIR` to the current hyprland session. `$PROFILE_NAME` is the name of the `.conf` file *WITHOUT* the `.conf` ending.
  - By appending the `--persist` (`-p` for short) flag, the changes to the current session will *OVERWRITE* the file located at `$HYPR_PERSIST_PROFILE_FILE`, unless the `-a` (short for 'append') is applied.
- `hypr-profile list` lists available profiles located under the `$HYPR_PROFILES_DIR` directory.
  - The list *DOES NOT* include entries which start with a dot (`.`). This can be used to hide profiles. **Profiles marked with a leading dot can still be applied.**