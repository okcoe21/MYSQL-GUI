slint::include_modules!();

mod state;
mod db;
mod app_controller;
mod explain;
mod tabs;

use std::sync::Arc;
use state::AppState;
use app_controller::AppController;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = AppWindow::new()?;
    let state = Arc::new(AppState::new());
    let controller = Arc::new(AppController::new(state));
    controller.setup_callbacks(&app);

    app.run()?;
    Ok(())
}
