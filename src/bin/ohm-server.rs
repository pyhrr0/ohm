use std::fs::File;
use std::process::exit;

use serde_yaml;
use structopt::{clap::AppSettings, StructOpt};

use ohm::create_server;
use ohm::Config;


#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
struct Options {
    #[structopt(short, long, default_value = "/etc/ohm/config.yaml")]
    config_file: String,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_opts = Options::from_args();

    match File::open(cli_opts.config_file) {
        Ok(config_file) => {
            let config: Config = serde_yaml::from_reader(config_file)?;
            let address = format!("{}:{}", &config.bind_addr, &config.port);

            let server = create_server(config)?;
            server.serve(address.parse()?).await?;
        }
        Err(msg) => {
            eprintln!("Failed to open config file: {}", msg);
            exit(1)
        }
    }

    Ok(())
}
