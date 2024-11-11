mod actions;
mod app;
mod run;
mod script_execution;
mod ui;
mod utils;
mod widgets;

use app::App;
use ui::run_event_loop;

pub use run::run_tui;
