use log::{debug, info, trace, warn};
use ockam_command::{config::AppConfig, console::Console, AppError};
use std::time::Duration;

use human_panic::setup_panic;
use ockam_command::command::CommandResult;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct App {
    console: Console,
    shutdown: Arc<AtomicBool>,
}

impl Default for App {
    fn default() -> Self {
        Self::load_environment();
        Self::init_logging();

        Self {
            console: Console::default(),
            shutdown: Arc::new(AtomicBool::default()),
        }
    }
}

impl App {
    pub fn load_environment() {
        dotenv::dotenv().ok();
    }

    pub fn init_logging() {
        setup_panic!();

        // TODO: Clashing with ockam logging
        // env_logger::init();
    }

    fn run(&mut self) -> Result<CommandResult, AppError> {
        let shutdown = self.shutdown.clone();

        let ctrlc_set = ctrlc::set_handler(move || {
            shutdown.store(true, Ordering::SeqCst);
        });

        if ctrlc_set.is_err() {
            warn!("Failed to set Ctrl-C handler");
        }

        AppConfig::evaluate()
    }

    fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
    }
}

#[ockam::node]
async fn main(mut ctx: ockam::Context) {
    let mut app = App::default();

    let _command_result = match app.run() {
        Ok(command) => command,
        Err(error) => {
            app.console.error(&error);
            std::process::exit(exitcode::SOFTWARE)
        }
    };

    while !app.is_shutdown() {
        info!("doing stuff");
        debug!("debug");
        trace!("trace");
        std::thread::sleep(Duration::from_secs(1))
    }
    ctx.stop().await.unwrap();
}
