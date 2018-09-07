use serde_yaml;
use std::convert::From;
use std::env;
use std::fs::File;
use std::io::{Error as IOError, ErrorKind, Result};
use std::path::{Path, PathBuf};
use super::mtklogo::{ColorMode, Endian};

#[derive(Debug, Clone, Deserialize, Serialize)]
/// It's exactly the same definition than ColorMode in the lib...
/// but I did'nt want to force the 'serde_yaml' in the lib, so this little wrapper.
pub enum ConfigColorMode {
    RgbaBig,
    RgbaLittle,
    BgraBig,
    BgraLittle,
    Rgb565Big,
    Rgb565Little,
}

impl ConfigColorMode {
    pub fn to_mtk(&self) -> ColorMode {
        match self {
            &ConfigColorMode::RgbaBig => ColorMode::Rgba(Endian::Big),
            &ConfigColorMode::RgbaLittle => ColorMode::Rgba(Endian::Little),
            &ConfigColorMode::BgraBig => ColorMode::Bgra(Endian::Big),
            &ConfigColorMode::BgraLittle => ColorMode::Bgra(Endian::Little),
            &ConfigColorMode::Rgb565Big => ColorMode::Rgb565(Endian::Big),
            &ConfigColorMode::Rgb565Little => ColorMode::Rgb565(Endian::Little),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub version: String,
    pub profiles: Vec<Profile>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Profile {
    pub name: String,
    pub color_model: ConfigColorMode,
    pub formats: Vec<Format>,
}

impl Profile {
    pub fn with_color_model(self, color_model: ConfigColorMode) -> Profile {
        return Profile { name: self.name, color_model, formats: self.formats };
    }
    pub fn guess_format(&self, size: u32, flip: bool) -> Result<Format> {
        let bpp = self.color_model.to_mtk().bytes_per_pixel();
        let pixels = size / bpp;
        let o = self.formats.iter()
            .find(|f| f.w * f.h == pixels)
            .map(|f| { f.clone() });
        match o {
            Some(f) => {
                if flip {
                    Ok(f.flip())
                } else {
                    Ok(f)
                }
            }
            None => Err(IOError::new(ErrorKind::InvalidData,
                                     format!(
                                         "size '{}' does not correspond to any dimension in profile '{}'", size, self.name)))
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Format {
    pub w: u32,
    pub h: u32,
    pub t: Option<String>,
}

impl Format {
    pub fn flip(&self) -> Format {
        let flipped_title = match &self.t {
            Some(s) => Some(format!("flip({})", s)),
            None => None
        };
        Format {
            w: self.h,
            h: self.w,
            t: flipped_title,
        }
    }
}


impl Config {
    const GLOBAL_CONFIG: &'static str = "/etc/mtklogo.yaml";
    const RELATIVE_CONFIG: &'static str = "mtklogo.yaml";

    /// Resolves configuration path, in this order:
    /// - `"mtklogo.yaml"` in $HOME/.config
    /// - `"mtklogo.yaml"` in /etc
    /// - `"mtklogo.yaml"` in program's installation directory
    fn config_path() -> Result<(PathBuf, File)> {
        // in home directory?
        let home_config = {
            let mut home = env::home_dir().ok_or(IOError::new(ErrorKind::NotFound, "No home directory."))?;
            home.push(".config");
            home.push(Self::RELATIVE_CONFIG);
            File::open(home.as_path()).map(|f| (home, f))
        };
        // in /etc?
        let etc_config = {
            File::open(Config::GLOBAL_CONFIG).map(|f| (PathBuf::from(Config::GLOBAL_CONFIG), f))
        };
        // along with the executable?
        let shipped_config = {
            let self_dir = env::current_exe()?;
            let parent = self_dir.parent()
                .ok_or(IOError::new(ErrorKind::NotFound, "Current executable is not inside a folder"))?; /* seriously ? */
            let mut self_config = PathBuf::from(parent);
            self_config.push(Self::RELATIVE_CONFIG);
            File::open(self_config.as_path()).map(|f| (self_config, f))
        };
        home_config
            .or_else(|_| etc_config)
            .or_else(|_| shipped_config)
            .map_err(|_| IOError::new(ErrorKind::NotFound,
                                      "`mtklogo.yaml` configuration not found, please provide one."))
    }

    fn wrap_read(path: &Path, file: File) -> Result<Config> {
        let config = serde_yaml::from_reader(file);
        config.map_err(
            |e| IOError::new(ErrorKind::InvalidData,
                             format!(
                                 "could not read config {:?} -> '{:?}'", path, e)))
    }

    pub fn from_file(path: &Path) -> Result<Config> {
        let file = File::open(path)?;
        Self::wrap_read(path, file)
    }

    pub fn load() -> Result<Config> {
        Config::config_path().and_then(|(path, file)| Self::wrap_read(path.as_path(), file))
    }
}