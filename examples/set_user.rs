use vozclient::core::voz_core::VozCore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let voz = VozCore::new("voz.vn".to_string());
    voz.set_user("1948176%2Cc4_Xyp99lbz3KmKZKjAXHuU8QuNhMg-X1FyW80iv".to_string(), "5ewKtjvOia2NZE9c8rk8fmc_cNgVT1-T".to_string(), None);

    let result = voz.get_current_user().await?;
    println!("{:?}", result);
    Ok(())
}