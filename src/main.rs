pub mod allocator;

use clap::Parser;

#[cfg(feature = "terminal")]
pub mod inter;
#[cfg(feature = "terminal")]
pub mod store;

mod args;
mod daemon;
mod parse;
mod update;
mod utils;

fn main() -> anyhow::Result<()> {
    let opt = args::cmd::Opt::parse();

    #[cfg(all(feature = "serve", not(feature = "terminal")))]
    if let Some(command) = opt.command {
        match command {
            args::ServeSubcommand::Run(args) => daemon::serve(args, false)?,
            #[cfg(target_family = "unix")]
            args::ServeSubcommand::Stop => daemon::serve_stop()?,
            #[cfg(target_family = "unix")]
            args::ServeSubcommand::Start(args) => daemon::serve_start(args)?,
            #[cfg(target_family = "unix")]
            args::ServeSubcommand::Restart(args) => daemon::serve_restart(args)?,
            #[cfg(target_family = "unix")]
            args::ServeSubcommand::Status => daemon::serve_status()?,
            #[cfg(target_family = "unix")]
            args::ServeSubcommand::Log => daemon::serve_log()?,
            args::ServeSubcommand::Genca => {
                let _ = mitm::cagen::gen_ca();
            }
            args::ServeSubcommand::GT { out } => daemon::generate_template(out)?,
            args::ServeSubcommand::Update => update::update()?,
        }
    }

    #[cfg(all(feature = "serve", feature = "terminal"))]
    if let Some(command) = opt.command {
        use args::cmd::SubCommands;
        match command {
            SubCommands::Serve(commands) => match commands {
                args::ServeSubcommand::Run(args) => daemon::serve(args, true)?,
                #[cfg(target_family = "unix")]
                args::ServeSubcommand::Stop => daemon::serve_stop()?,
                #[cfg(target_family = "unix")]
                args::ServeSubcommand::Start(args) => daemon::serve_start(args)?,
                #[cfg(target_family = "unix")]
                args::ServeSubcommand::Restart(args) => daemon::serve_restart(args)?,
                #[cfg(target_family = "unix")]
                args::ServeSubcommand::Status => daemon::serve_status()?,
                #[cfg(target_family = "unix")]
                args::ServeSubcommand::Log => daemon::serve_log()?,
                args::ServeSubcommand::Genca => {
                    let _ = openai::serve::preauth::cagen::gen_ca();
                }
                args::ServeSubcommand::GT { out } => daemon::generate_template(out)?,
                args::ServeSubcommand::Update => update::update()?,
            },
            SubCommands::Terminal => {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .worker_threads(1)
                    .max_blocking_threads(1)
                    .build()?;

                runtime.block_on(inter::prompt())?;
            }
        }
    }

    Ok(())
}
