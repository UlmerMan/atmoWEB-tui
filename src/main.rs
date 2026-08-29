use atmoWEB_tui::atmoweb::AtmoWeb;
use atmoWEB_tui::cli;

use std::error::Error;

use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();

    let oven = AtmoWeb::new(&args.address);
    println!("Oven created with IP: {}", oven.get_ip_address().await);
    
    println!("Checking if oven is online... ");
    let oven_online = oven.is_online().await;
    println!("Online: {}", oven_online);

    if !oven_online {
        return Err(("Oven is not online").into());
    }
 
    let resp = oven.set_temp(40.0).await?;
    println!("Set: {:#?}", resp);

    let resp = oven.read_temp1().await?;
    println!("Ist: {:#?}", resp);

    let resp = oven.read_flap().await?;
    println!("Flap: {:#?}", resp);

    let resp = oven.read_fan().await?;
    println!("Fan: {:#?}", resp);

    let resp = oven.set_fan(0.0).await?;
    println!("Set Fan: {:#?}", resp);
    
    Ok(())
}
