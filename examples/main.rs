use vozclient::VozCore;
use vozclient::VozResponseMapping;

#[tokio::main]
async fn main() {
    let voz = VozCore::new("voz.vn".to_string());
    voz.set_user("replace your user cookie".to_string(), "replace your session cookie".to_string());
    let user = voz.get_current_user().await.voz_response();
    println!("{:?}", user);
}
