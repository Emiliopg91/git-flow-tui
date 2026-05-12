use std::{collections::HashMap, process::exit};

use color_eyre::eyre::Result;
use git_flow_tui_core::{initialize_and_validate, others::whiteboard::WHITEBOARD};

use crate::ui::main_loop;
mod ui;

fn main() -> Result<()> {
    if let Err(e) = initialize_and_validate() {
        eprintln!("Error on git initialization: {}", e);
        exit(1);
    }

    WHITEBOARD.get_or_init(|| HashMap::new().into());

    main_loop()
}
