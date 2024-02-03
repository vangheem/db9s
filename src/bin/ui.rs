use db9s::app;
use db9s::ui;
use fern;

fn setup_logging() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info) // Set the default log level
        .level_for("escuell", log::LevelFilter::Info) // Set specific log level for your crate
        .chain(fern::log_file("escuell.log")?) // Log to a file
        .apply()?;
    Ok(())
}

fn main() {
    setup_logging().unwrap();
    let app = app::Application::new();
    ui::run_ui(app).unwrap();
}
