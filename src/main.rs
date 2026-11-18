use std::{env, error::Error, process::ExitCode};

use mambofinance::{Application, mcp, tui};

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("mambofinance: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args().skip(1);
    let mode = arguments.next();
    if arguments.next().is_some() || !matches!(mode.as_deref(), None | Some("mcp")) {
        return Err("usage: mambofinance [mcp]".into());
    }

    let application = Application::open_default()?;
    match mode.as_deref() {
        None => tui::run(&application),
        Some("mcp") => mcp::run(application).await,
        Some(_) => unreachable!("mode was validated before opening storage"),
    }
}
