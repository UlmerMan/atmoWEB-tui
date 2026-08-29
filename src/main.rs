use atmoWEB_tui::atmoweb::AtmoWeb;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let oven = AtmoWeb::new("192.168.1.25");
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
