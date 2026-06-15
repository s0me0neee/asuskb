use std::{
    fmt::Display,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
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
            Self::Off => "off",
            Self::Low => "low",
            Self::Med => "med",
            Self::High => "high",
        };
        write!(f, "{s}")
    }
}

impl TryFrom<&str> for KbLevel {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "off" => Ok(Self::Off),
            "low" => Ok(Self::Low),
            "med" => Ok(Self::Med),
            "high" => Ok(Self::High),
            _ => bail!("Invalid keyboard level, expected 'off', 'low', 'med', or 'high'"),
        }
    }
}

impl TryFrom<Kbu8Level> for KbLevel {
    type Error = anyhow::Error;

    fn try_from(value: Kbu8Level) -> Result<Self, Self::Error> {
        match value.0 {
            0 => Ok(Self::Off),
            1 => Ok(Self::Low),
            2 => Ok(Self::Med),
            3 => Ok(Self::High),
            _ => bail!("Invalid number keyboard level, expected '0', '1', '2' or '3'"),
        }
    }
}

impl From<KbLevel> for Kbu8Level {
    fn from(value: KbLevel) -> Self {
        match value {
            KbLevel::Off => Self(0),
            KbLevel::Low => Self(1),
            KbLevel::Med => Self(2),
            KbLevel::High => Self(3),
        }
    }
}

const ASUS_UTIL: &str = "asusctl";

pub(crate) fn get_asusctl() -> Result<PathBuf> {
    which::which_global(ASUS_UTIL)
        .context("Can not find asusctl on system, install asusctl first")
}

pub(crate) fn get_kb_light_level(asusctl: &Path) -> Result<KbLevel> {
    let output = Command::new(asusctl)
        .args(["leds", "get"])
        .output()
        .context("Failed to execute asusctl command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Asusctl exited with exit code {} \n {}",
            output.status.code().unwrap_or(1),
            stderr.trim()
        );
    }

    if let Some(level) = String::from_utf8_lossy(&output.stdout)
        .trim()
        .split(": ")
        .nth(1)
    {
        KbLevel::try_from(level).map_err(|e| anyhow::anyhow!(e))
    } else {
        bail!("Can not parse output into a valid keyboard light level");
    }
}

pub(crate) fn set_kb_light_level(asusctl: &Path, kb_level: KbLevel) -> Result<()> {
    let output = Command::new(asusctl)
        .args(["leds", "set", &kb_level.to_string()])
        .output()
        .context("Failed to execute asusctl command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Asusctl exited with exit code {} \n {}",
            output.status.code().unwrap_or(1),
            stderr.trim()
        );
    }

    Ok(())
}

fn change_kb_light_level(asusctl: &Path, step: i8) -> Result<()> {
    let cur = Kbu8Level::from(get_kb_light_level(asusctl)?).0;
    let new_level = match (i16::from(cur) + i16::from(step)).clamp(0, 3) {
        0 => KbLevel::Off,
        1 => KbLevel::Low,
        2 => KbLevel::Med,
        _ => KbLevel::High,
    };
    set_kb_light_level(asusctl, new_level)
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
        _ => bail!("Step has to be between -3 and 3 inclusive"),
    }
}
