use self::lgtv::LgTv;
use clap::Parser;
use clap::Subcommand;

mod lgtv;

#[derive(Parser)]
#[command(author, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Lgtv(LgTv),
}

async fn true_main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();

    match args.cmd {
        Commands::Lgtv(lg) => lg.handle().await?,
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    true_main().await
}
