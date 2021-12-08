use ohm::grpc::create_server;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short, long, default_value = "127.0.0.1")]
    host: String,

    #[structopt(short, long, default_value = "1234")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opt::from_args();

    let server = create_server()?;
    server
        .serve(format!("{}:{}", opts.host, opts.port).parse().unwrap())
        .await?;

    Ok(())
}
