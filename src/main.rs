use ledzilla_server::*;


fn main() {
    simple_logger::SimpleLogger::new()
        .env()
        .with_local_timestamps()
        .with_timestamp_format(time::macros::format_description!(
            "[hour]:[minute]:[second]"
        ))
        .init()
        .unwrap();

    log::info!("Hello, world!");
    run_server();
    return;

}
