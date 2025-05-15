use dotenv::dotenv;
use log::info;

fn main() -> eyre::Result<()> {
    // Initialize environment and logger
    dotenv()?;
    pretty_env_logger::init();
    
    info!("PiLock started");
    
    Ok(())
}