use std::{
    fmt::Display,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};

#[derive(Debug, Clone, clap::ValueEnum)]
pub(crate) enum KbLevel {
    #[clap(name = "off")]
    Off,
    #[clap(name = "low")]
    Low,
    #[clap(name = "med")]
    Med,
    #[clap(name = "high")]
    High,
}

#[derive(Debug)]
pub(crate) struct Kbu8Level(u8);

impl Display for KbLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            KbLevel::Off => "off",
            KbLevel::Low => "low",
            KbLevel::Med => "med",
            KbLevel::High => "high",
        };
        write!(f, "{}", s)
    }
}

impl TryFrom<&str> for KbLevel {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "off" => Ok(KbLevel::Off),
            "low" => Ok(KbLevel::Low),
            "med" => Ok(KbLevel::Med),
            "high" => Ok(KbLevel::High),
            &_ => bail!("Invalid keyboard level, expected 'Off', Low', 'Med', or 'High'"),
        }
    }
}

impl TryFrom<Kbu8Level> for KbLevel {
    type Error = anyhow::Error;

    fn try_from(value: Kbu8Level) -> std::prelude::v1::Result<Self, Self::Error> {
        match value.0 {
            0 => Ok(KbLevel::Off),
            1 => Ok(KbLevel::Low),
            2 => Ok(KbLevel::Med),
            3 => Ok(KbLevel::High),
            _ => bail!("Invalid number keyboard level, expected '0', '1', '2' or '3'"),
        }
    }
}

impl From<KbLevel> for Kbu8Level {
    fn from(value: KbLevel) -> Self {
        match value {
            KbLevel::Off => Kbu8Level(0),
            KbLevel::Low => Kbu8Level(1),
            KbLevel::Med => Kbu8Level(2),
            KbLevel::High => Kbu8Level(3),
        }
    }
}

const ASUS_UTIL: &str = "asusctl";

pub(crate) fn get_asusctl() -> Result<PathBuf> {
    let asusctl_path = which::which_global(ASUS_UTIL)
        .context("Can not find asusctl on system, install asusctl first")?;

    if !asusctl_path.exists() {
        bail!("Asusctl is found but its path is invalid");
    }

    if asusctl_path.is_dir() {
        bail!("Asusctl path is a directory");
    }

    std::path::absolute(asusctl_path)
        .context("Failed to convert asusctl path into a system absolute path")
}

pub(crate) fn get_kb_light_level(asusctl: &Path) -> Result<KbLevel> {
    let get_cmd = Command::new(asusctl)
        .args(["leds", "get"])
        .output()
        .context("Failed to execute asusctl command")?;

    if !get_cmd.status.success() {
        let stderr = String::from_utf8_lossy(&get_cmd.stderr);
        bail!(
            "Asusctl exited with exit code {} \n {}",
            get_cmd.status.code().unwrap_or(1),
            stderr.trim()
        );
    }

    let kb_level = if let Some(level) = String::from_utf8_lossy(&get_cmd.stdout)
        .trim()
        .split(": ")
        .nth(1)
    {
        KbLevel::try_from(level).map_err(|e| anyhow::anyhow!(e))?
    } else {
        bail!("Can not parse output into a valid keyboard light level");
    };

    Ok(kb_level)
}

pub(crate) fn set_kb_light_level(asusctl: &Path, kb_level: KbLevel) -> Result<()> {
    let mut cmd = Command::new(asusctl);
    let set_cmd = cmd.args(["leds", "set"]);
    let kb_level_arg = kb_level.to_string();
    set_cmd.arg(kb_level_arg);

    if let Ok(mut child) = set_cmd.spawn() {
        if let Ok(exit_code) = child.wait() {
            if !exit_code.success() {
                let mut stderr_buffer = String::new();
                let mut stderr = child
                    .stderr
                    .ok_or_else(|| anyhow::anyhow!("Failed to get stderr"))?;
                stderr.read_to_string(&mut stderr_buffer)?;
                bail!(
                    "Asusctl exited with exit code {} \n {}",
                    exit_code.code().unwrap_or(1),
                    stderr_buffer.trim()
                );
            }
        } else {
            bail!("Can not run chile process")
        }
    } else {
        bail!("Failed to spawn child process")
    }

    Ok(())
}

fn change_kb_light_level(asusctl: &Path, step: i8) -> Result<()> {
    let cur_level = Kbu8Level::from(get_kb_light_level(asusctl)?);
    let new_level = (cur_level.0 as i8 + step).clamp(0, 3);
    set_kb_light_level(asusctl, KbLevel::try_from(Kbu8Level(new_level as u8))?)?;
    Ok(())
}

pub(crate) fn inc_kb_light_level(asusctl: &Path) -> Result<()> {
    change_kb_light_level(asusctl, 1)
}

pub(crate) fn dec_kb_light_level(asusctl: &Path) -> Result<()> {
    change_kb_light_level(asusctl, -1)
}

pub(crate) fn custom_kb_light_level(asusctl: &Path, step: i8) -> Result<()> {
    match step {
        (-3..=3) => change_kb_light_level(asusctl, step),
        _ => {
            bail!("Step have to be between -3 and 3 inclusive")
        }
    }
}
