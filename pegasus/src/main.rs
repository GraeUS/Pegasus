mod ui;

fn main() {
    env_logger::Builder::new()
        .format_timestamp(None)
        .filter_module("pegasus", log::LevelFilter::Info)
        .parse_default_env()
        .init();

    ui::run();
}
