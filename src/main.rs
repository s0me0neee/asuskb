use anyhow::bail;
use clap::Parser;
pub mod asusctl;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: KbCommands,
}

#[derive(clap::Subcommand, Debug)]
enum KbCommands {
    Current,

    Set {
        #[arg(value_parser)]
        level: asusctl::KbLevel,
    },

    Inc,
    Dec,
    Step {
        #[arg(allow_hyphen_values = true)]
        step: i8,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if std::env::args().len() == 1 {
        bail!("No arguments are provided");
    }
    let asusctl = asusctl::get_asusctl()?;

    match args.command {
        KbCommands::Current => {
            let kb_level = asusctl::get_kb_light_level(&asusctl)?;
            println!("{kb_level}");
        }
        KbCommands::Set { level } => {
            let kb = asusctl::KbLevel::try_from(level.to_string().as_str())?;
            asusctl::set_kb_light_level(&asusctl, kb)?;
        }
        KbCommands::Inc => asusctl::inc_kb_light_level(&asusctl)?,
        KbCommands::Dec => asusctl::dec_kb_light_level(&asusctl)?,
        KbCommands::Step { step } => asusctl::custom_kb_light_level(&asusctl, step)?,
    }

    Ok(())
}
