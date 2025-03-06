use retumi::run_main;

fn main() {
    env_logger::init();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            if let Err(err) = run_main().await {
                log::error!("{err}");
                std::process::exit(1);
            }
        })
}
