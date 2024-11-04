use core::panic;
use std::{path::Path, sync::OnceLock};

use anyhow::Result;

#[derive(Clone)]
pub struct FgmContext {
    // 下载位置
    pub installations_dir: String,
    // 使用位置
    pub gate_path: String,
    // 下载索引界面
    pub remote_source: String,
    // 是否更新
    pub update: bool,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct FgmConfig {
    // set ignore if is None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installations_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_source: Option<String>,
}

impl FgmContext {
    pub fn update_from_config(&mut self, cf: &FgmConfig) {
        if let Some(v) = &cf.installations_dir {
            self.installations_dir = v.to_owned();
        }
        if let Some(v) = &cf.gate_path {
            self.gate_path = v.to_owned();
        }
        if let Some(v) = &cf.remote_source {
            self.remote_source = v.to_owned();
        }
    }

    pub fn legalize_home_dir(&mut self) {
        legalize(&mut self.installations_dir);
        legalize(&mut self.gate_path);
        legalize(&mut self.remote_source);
    }
}

fn legalize(path: &mut String) {
    static HOME_DATA: OnceLock<String> = OnceLock::new();
    if path.contains("~") {
        let home_dir = HOME_DATA.get_or_init(|| match dirs::home_dir() {
            Some(d) => d.display().to_string(),
            None => panic!("fail to get home dir"),
        });
        *path = path.replace("~", home_dir);
    }
}

impl Default for FgmContext {
    fn default() -> Self {
        FgmContext {
            installations_dir: "/usr/local/share/fgm".to_string(),
            gate_path: "~/.local/share/fgm/go".to_string(),
            remote_source: "https://go.dev/dl/".to_string(),
            update: false,
        }
    }
}

pub const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
pub const XDG_DATA_HOME: &str = "XDG_DATA_HOME";

pub fn count_config_path() -> Result<String> {
    let cf_dir = std::env::var(XDG_CONFIG_HOME).map(|v| format!("{v}/fgm/config.toml"));
    let cf_dir = match cf_dir {
        Ok(v) => v,
        Err(_) => std::env::var("HOME").map(|v| format!("{v}/.fgm/config.toml"))?,
    };

    Ok(cf_dir)
}

pub fn count_remotes_index_path() -> Result<String> {
    let data_home = std::env::var(XDG_DATA_HOME).map(|v| format!("{}/fgm", v));
    let data_home = match data_home {
        Ok(v) => v,
        Err(_) => std::env::var("HOME").map(|v| format!("{}/.config/fgm", v))?,
    };
    let index_path = format!("{}/remotes", data_home);
    Ok(index_path)
}

impl FgmConfig {
    pub fn load<P: AsRef<Path>>(cf_path: P) -> Result<FgmConfig> {
        let toml_str = std::fs::read_to_string(cf_path)?;
        Ok(toml::from_str(&toml_str)?)
    }

    pub fn save<P: AsRef<Path>>(&self, cf_path: P) -> Result<()> {
        let toml_str = toml::to_string(self)?;
        std::fs::write(cf_path, toml_str)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legalize() {
        let mut m = "~/gg".to_string();
        legalize(&mut m);
        dbg!(m);
    }
}
