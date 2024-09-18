pub mod arch;
pub mod cli;
pub mod config;
pub mod mpsc;

use std::{
    fs::{self, create_dir_all},
    path::Path,
    process::Command,
};

use anyhow::{anyhow, Context, Result};
use config::FgmConfig;
use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct DownloadLink {
    pub filename: String,
    pub link: String,
}

pub fn get_remotes(cf: &FgmConfig) -> Result<Vec<DownloadLink>> {
    // 从网络读取一个文件
    let dl = reqwest::blocking::get(&cf.remote_source)
        .unwrap()
        .text()
        .unwrap();
    let dl = dl.as_str();

    // 假设你已经有了HTML内容，这里我们用一个字符串来模拟

    // 解析HTML
    let document = Html::parse_document(dl);

    // 定义一个选择器来匹配所需的<td>元素
    let selector = Selector::parse("td.filename a.download").unwrap();

    let system_arch = arch::system_arch();
    let mut remotes = vec![];

    for element in document.select(&selector) {
        let filename = element.text().collect::<Vec<_>>().join(" ");
        let link = element.value().attr("href").ok_or_else(|| anyhow!(""))?;
        let link = format!("https://go.dev{link}",);
        if filename.contains(&system_arch) {
            remotes.push(DownloadLink { filename, link });
        }
    }

    Ok(remotes)
}

pub fn get_installed(cf: &FgmConfig) -> Result<Vec<String>> {
    let download_dir = Path::new(&cf.installations_dir);
    let entries = fs::read_dir(download_dir)?;
    let mut installed = vec![];
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_dir() {
            let Some(file_name) = path.file_name() else {
                continue;
            };
            let Some(file_name) = file_name.to_str() else {
                continue;
            };
            installed.push(file_name.to_string());
        }
    }
    Ok(installed)
}

pub fn list_remote(cf: &FgmConfig, sort: bool) -> Result<()> {
    let mut d = get_remotes(cf)?;
    let suffix = format!(".{}.tar.gz", arch::system_arch());
    if sort {
        d.sort_by(|a, b| a.filename.cmp(&b.filename));
    }
    for i in d {
        if let Some(version) = i.filename.strip_suffix(&suffix) {
            if let Some(version) = version.strip_prefix("go") {
                println!("{}", version);
            }
        }
    }
    Ok(())
}

pub fn list_installed(cf: &FgmConfig, sort: bool) {
    let mut d = get_installed(cf).unwrap_or_default();
    let suffix = format!(".{}", arch::system_arch());
    if sort {
        d.sort();
    }
    d.retain(|x| x.ends_with(&suffix));
    let current_version = current_version(cf).ok();
    for i in d {
        if let Some(version) = i.strip_suffix(&suffix) {
            if let Some(version) = version.strip_prefix("go") {
                if let Some(current_version) = &current_version {
                    if version == current_version {
                        println!("{} (current)", version);
                    } else {
                        println!("{}", version);
                    }
                } else {
                    println!("{}", version);
                }
            }
        }
    }
}

pub fn install(config: &FgmConfig, version: &str) -> Result<()> {
    let d = find_link(config, version)?;

    let installations_dir: &str = &config.installations_dir;
    create_dir_all(installations_dir)?;
    let download_dir = Path::new(installations_dir).canonicalize()?;
    let download_path = download_dir.join(&d.filename);

    println!("Downloading {} from {}", d.filename, d.link);

    dbg!(&d.link, &download_path);
    Command::new("wget")
        .arg(&d.link)
        .arg("-O")
        .arg(&download_path)
        .status()
        .expect("failed to execute process");

    // 使用tar crate解压缩
    let tar_gz = std::fs::File::open(&download_path)?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(&download_dir)?;

    let output = download_dir.join(d.filename.replace(".tar.gz", ""));
    std::fs::rename(download_dir.join("go"), output)?;
    // 删除下载的压缩包
    std::fs::remove_file(&download_path)?;

    Ok(())
}

pub fn find_link(config: &FgmConfig, version: &str) -> Result<DownloadLink> {
    let d = get_remotes(config)?;
    let target = d
        .iter()
        .find(|x| x.filename.contains(version))
        .ok_or_else(|| {
            anyhow!(
                "
                target version not found
            "
            )
        })?;
    Ok(target.clone())
}

pub fn uninstall(cf: &FgmConfig, version: &str) -> Result<()> {
    println!("Uninstalling {}", version);
    let d = find_link(cf, version)?;
    let download_dir = Path::new(&cf.installations_dir);
    let go_dir = download_dir.join(d.filename.replace(".tar.gz", ""));
    if go_dir.exists() {
        fs::remove_dir_all(go_dir)?;
    }
    Ok(())
}

pub fn _use(cf: &FgmConfig, version: &str) -> Result<()> {
    // 首先查询是否下载了指定的版本
    let d = find_link(cf, version).with_context(|| context!())?;
    let download_dir = Path::new(&cf.installations_dir);
    let go_dir = download_dir.join(d.filename.replace(".tar.gz", ""));
    let go_dir = go_dir.canonicalize().with_context(|| context!())?;
    println!("go_dir: {:?}", go_dir);
    if !go_dir.exists() {
        install(cf, version)?;
    }
    println!("Using {}", version);
    // 如果下载了指定的版本，就直接使用
    let gate_path = Path::new(&cf.gate_path);

    let gate_path_dir = gate_path
        .parent()
        .ok_or_else(|| {
            anyhow!(
                "
    {} is not a valid path
",
                cf.gate_path
            )
        })
        .with_context(|| context!())?;
    create_dir_all(gate_path_dir).with_context(|| context!())?;

    // 如果存在文件
    if gate_path.try_exists().ok() == Some(true) {
        fs::remove_dir_all(gate_path).with_context(|| context!())?;
    }

    #[cfg(target_os = "linux")]
    {
        std::os::unix::fs::symlink(&go_dir, gate_path).with_context(|| context!())?;
    }
    #[cfg(target_os = "windows")]
    {
        todo!();
    }

    Ok(())
}

pub fn current_version(config: &FgmConfig) -> Result<String> {
    // 查询/usr/local/go软链接指向的目录
    let gate_path = Path::new(&config.gate_path);
    let target_path = fs::read_link(gate_path)?;
    let target_path = target_path.to_str().ok_or(anyhow!(""))?;
    let version = target_path
        .strip_prefix(&config.installations_dir)
        .ok_or(anyhow!(""))?;
    let version = version.strip_prefix("go").ok_or(anyhow!(""))?;
    Ok(version.to_string())
}

// 生成设置环境变量的脚本
pub fn env(config: &FgmConfig) -> String {
    format!("export PATH={}/bin:$PATH", config.gate_path)
}
