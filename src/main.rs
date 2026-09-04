slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = MainWindow::new()?;

    window.window().set_maximized(true);

    // Handle Quit callback from Slint
    window.on_quit(|| {
        // Exit the application
        std::process::exit(0);
    });

    window.run()?;

    Ok(())
}
