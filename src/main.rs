mod config;

#[tokio::main]
async fn main() {
    let cfg = config::load_cfg("./config.yaml").await;
}
