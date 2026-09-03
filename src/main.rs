slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = MainWindow::new()?;

    window.window().set_maximized(true);

    window.run()?;

    Ok(())
}
