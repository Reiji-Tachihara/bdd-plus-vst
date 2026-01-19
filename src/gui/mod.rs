use nih_plug::prelude::Editor;
use std::sync::Arc;

use crate::params::BddPlusParams;

mod editor;
mod theme;

pub fn create_editor(params: Arc<BddPlusParams>) -> Option<Box<dyn Editor>> {
    editor::create_editor(params)
}
