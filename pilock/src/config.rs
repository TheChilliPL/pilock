//! Loading and saving configuration for the application.

use std::env::var_os;
use std::ffi::OsStr;
use std::num::NonZero;
use std::path::Path;
use dotenv::var;
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Contrast(u8);

impl Contrast {
    pub fn new(value: u8) -> Self {
        if value > 0b111111 {
            panic!("Contrast value must be between 0 and 63");
        }
        Contrast(value)
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Default for Contrast {
    fn default() -> Self {
        Contrast::new(32)
    }
}

/// A struct keeping all the configuration for the application.
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// The password used to unlock the application.
    pub password: Vec<char>,
    /// The number of seconds the application will remain unlocked after entering the password.
    pub unlock_seconds: NonZero<u8>,
    /// The contrast setting for the display. Range: 0-63.
    pub contrast: Contrast,
}

impl Config {
    /// Loads the configuration from the environment variable `CONFIG_FILE` or defaults to `config.json`.
    /// If the file does not exist, it returns `None`.
    pub fn try_load() -> Option<Self> {
        let config_str = var_os("CONFIG_FILE");
        let config_str: &OsStr = config_str.as_deref().unwrap_or(OsStr::new("config.json"));
        let config_path = Path::new(config_str);
        if config_path.exists() {
            let file = std::fs::File::open(config_path).ok()?;
            let reader = std::io::BufReader::new(file);
            serde_json::from_reader(reader).ok()
        } else {
            None
        }
    }
    
    /// Saves the current configuration to the file specified by the `CONFIG_FILE` environment variable or defaults to `config.json`.
    pub fn save(&mut self) -> std::io::Result<()> {
        let config_str = var("CONFIG_FILE").unwrap_or_else(|_| "config.json".to_string());
        let config_path = Path::new(&config_str);
        let file = std::fs::File::create(config_path)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }
}

impl Default for Config {
    /// Returns a default configuration with a password of "1234", unlock time of 5 seconds, and a contrast of 32.
    fn default() -> Self {
        Config {
            password: vec!['1', '2', '3', '4'],
            unlock_seconds: NonZero::new(5).unwrap(),
            contrast: Contrast::default(),
        }
    }
}
