use std::{env, path::PathBuf};

pub(crate) fn home() -> PathBuf {
    env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

pub(crate) fn data_home() -> PathBuf {
    env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/share"))
}

pub(crate) fn config_home() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"))
}

pub(crate) fn log_path() -> PathBuf {
    env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/state"))
        .join("diablo-launcher/game.log")
}

pub(crate) fn game_log(game: crate::game::Game) -> PathBuf {
    if game == crate::game::Game::DiabloII {
        log_path().with_file_name("diablo2.log")
    } else {
        log_path()
    }
}
