use std::path::Path;

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
}

impl Default for FgmContext {
    fn default() -> Self {
        FgmContext {
            installations_dir: "/usr/local/share/fgm".to_string(),
            gate_path: "/usr/local/go".to_string(),
            remote_source: "https://go.dev/dl/".to_string(),
            update: false,
        }
    }
}

pub const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
pub const XDG_DATA_HOME: &str = "XDG_DATA_HOME";

pub fn count_config_path() -> Result<String> {
    let cf_dir = std::env::var(XDG_CONFIG_HOME).map(|v| format!("{}/fgm", v));
    let cf_dir = match cf_dir {
        Ok(v) => v,
        Err(_) => std::env::var("HOME").map(|v| format!("{}/.fgm", v))?,
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
