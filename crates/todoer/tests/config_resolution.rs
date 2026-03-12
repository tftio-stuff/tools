#![allow(unsafe_code)]

use std::path::PathBuf;
use todoer::config::{Config, resolve_config_path, resolve_db_path};

#[test]
fn config_path_respects_xdg_config_home() {
    // SAFETY: test runs in isolation (serial_test or single-threaded)
    unsafe { std::env::set_var("XDG_CONFIG_HOME", "/tmp/xdgconfig") };
    let path = resolve_config_path().unwrap();
    assert_eq!(path, PathBuf::from("/tmp/xdgconfig/todoer/config.toml"));
}

#[test]
fn db_path_defaults_to_xdg_data_home() {
    // SAFETY: test runs in isolation (serial_test or single-threaded)
    unsafe { std::env::set_var("XDG_DATA_HOME", "/tmp/xdgdata") };
    let path = resolve_db_path(&Config { db_path: None }).unwrap();
    assert_eq!(path, PathBuf::from("/tmp/xdgdata/todoer/todoer.db"));
}
