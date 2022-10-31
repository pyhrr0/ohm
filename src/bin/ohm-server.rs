use std::{error::Error, fs::File};

use structopt::{clap::AppSettings, StructOpt};

use ohm::{Config, Server};

#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
struct Options {
    #[structopt(short, long, default_value = "/etc/ohm/config.yaml")]
    config_file: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli_opts = Options::from_args();

    let config_file = File::open(cli_opts.config_file)
        .map_err(|msg| format!("Failed to open config file: {}", msg))?;
    let config: Config = serde_yaml::from_reader(config_file)?;
    let address = format!("{}:{}", &config.bind_addr, &config.port);

    let server = Server::new(config)?;
    server.serve(address.parse()?).await?;

    Ok(())
}
