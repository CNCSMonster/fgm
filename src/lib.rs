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
use config::{count_remotes_index_path, FgmContext};
use mpsc::must_write;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DownloadLink {
    pub filename: String,
    pub link: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Links {
    pub links: Vec<DownloadLink>,
}

pub fn get_remotes(cf: &FgmContext) -> Result<Vec<DownloadLink>> {
    if !cf.update {
        if let Ok(remote_index) = count_remotes_index_path() {
            if let Ok(toml_str) = fs::read_to_string(&remote_index) {
                let links = toml::from_str::<Links>(&toml_str).with_context(|| context!())?;
                let remotes: Vec<DownloadLink> = links.links;
                return Ok(remotes);
            }
        }
    }

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

    let remote_index = count_remotes_index_path().with_context(|| context!())?;
    let toml_str = toml::to_string_pretty(&Links {
        links: remotes.to_vec(),
    })
    .with_context(|| context!())?;
    must_write(remote_index, &toml_str).with_context(|| context!())?;

    Ok(remotes)
}

pub fn get_installed(cf: &FgmContext) -> Result<Vec<String>> {
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

pub fn update(cf: &FgmContext) -> Result<()> {
    let mut ctx = cf.clone();
    ctx.update = true;
    println!("Updating remotes index in {}", count_remotes_index_path()?);
    list_remote(&ctx, false)?;
    Ok(())
}

pub fn list_remote(cf: &FgmContext, sort: bool) -> Result<()> {
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

pub fn list_installed(cf: &FgmContext, sort: bool) {
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

pub fn install(config: &FgmContext, version: &str) -> Result<()> {
    let d = find_link(config, version)?;

    let installations_dir: &str = &config.installations_dir;
    create_dir_all(installations_dir)?;
    let download_dir = Path::new(installations_dir).canonicalize()?;
    let download_path = download_dir.join(&d.filename);

    println!("Downloading {:?} from {}", download_path, d.link);

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

pub fn find_link(config: &FgmContext, version: &str) -> Result<DownloadLink> {
    let d = get_remotes(config)?;
    let suffix = format!(".{}.tar.gz", arch::system_arch());
    let target = d
        .iter()
        .find(|x| x.filename.eq(&format!("go{}{}", version, suffix)))
        .ok_or_else(|| {
            anyhow!(
                "
                target version not found
            "
            )
        })?;
    Ok(target.clone())
}

pub fn uninstall(cf: &FgmContext, version: &str) -> Result<()> {
    println!("Uninstalling {}", version);
    let d = find_link(cf, version)?;
    let download_dir = Path::new(&cf.installations_dir);
    let go_dir = download_dir.join(d.filename.replace(".tar.gz", ""));
    if go_dir.exists() {
        fs::remove_dir_all(go_dir)?;
    }
    Ok(())
}

pub fn _use(cf: &FgmContext, version: &str) -> Result<()> {
    // 首先查询是否下载了指定的版本
    let installed = get_installed(cf).unwrap_or_default();
    if !installed.contains(&format!("go{}.{}", version, arch::system_arch())) {
        install(cf, version)?;
    }
    let go_dir =
        Path::new(&cf.installations_dir).join(format!("go{}.{}", version, arch::system_arch()));
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

    let _ = fs::remove_file(gate_path);

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

pub fn current_version(ctx: &FgmContext) -> Result<String> {
    let gate_path = Path::new(&ctx.gate_path);
    let target_path = fs::read_link(gate_path)?.canonicalize()?;
    let target_path = target_path.to_str().ok_or(anyhow!(""))?;

    let installations_dir = Path::new(&ctx.installations_dir).canonicalize()?;

    let prefix = format!("{}/", installations_dir.to_str().ok_or(anyhow!(""))?);
    let version = target_path.strip_prefix(&prefix).ok_or(anyhow!(""))?;

    let suffix = format!(".{}", arch::system_arch());
    let version = version
        .strip_prefix("go")
        .ok_or(anyhow!(""))?
        .strip_suffix(&suffix)
        .ok_or(anyhow!(""))?;

    Ok(version.to_string())
}

// 生成设置环境变量的脚本
pub fn init_script(config: &FgmContext) -> String {
    format!("export PATH={}/bin:$PATH", config.gate_path)
}
