use crate::command::CommandResult;
use crate::AppError;
use clap::Parser;
use log::{debug, info, warn};

#[derive(Parser)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, long, default_value = "ockam.toml")]
    config: String,

    #[clap(short, long, default_value = "ockam_secrets.toml")]
    secrets: String,

    #[clap(subcommand)]
    command: Option<OckamCommand>,
}

#[derive(clap::Subcommand)]
enum OckamCommand {
    Outlet,
}

const OCKAM_ENV_PREFIX: &str = "OCKAM";

pub struct AppConfig {}

impl AppConfig {
    pub fn evaluate() -> Result<CommandResult, AppError> {
        let mut config = config::Config::default();

        let args = Args::parse();

        if config.merge(config::File::with_name(&args.config)).is_ok() {
            info!("Loaded settings from {}.", args.config)
        } else {
            debug!("No config file present.")
        }

        if config.merge(config::File::with_name(&args.secrets)).is_ok() {
            info!("Loaded secrets from {}.", args.secrets)
        } else {
            debug!("No secrets file present.")
        }

        config
            .merge(config::Environment::with_prefix(OCKAM_ENV_PREFIX))
            .ok();

        /*




            let (command_name, command_args) = args.subcommand();
            let mut command: Command = command_name.parse()?;
            command.run(command_args)
        */
        Err(AppError::Unknown)
    }
}
