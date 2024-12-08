use std::sync::{Arc, Mutex};
use crate::{LCDdriver, GpioUi};
use crate::GlobalIoHandlers;
use crate::ui_pages::{MenuPage, UiPages};
use crate::walk_engine;

pub (crate) struct ManualControllPage {
    pub (crate)  global_io: GlobalIoHandlers,
    pub (crate)  current_selection: usize,
    pub (crate)  position: u8,
}
impl MenuPage for ManualControllPage {
    
    fn get_lcd(&mut self) -> Arc<Mutex<LCDdriver>> {
        self.global_io.lcd.clone()
    }

    fn get_gpio_controller(&mut self) -> Arc<Mutex<GpioUi>> {
        self.global_io.gpio_ui.clone()
    }

    fn get_current_selection(&self) -> usize {
        self.current_selection
    }

    fn set_current_selection(&mut self, selection: usize) -> () {
        self.current_selection = selection;
    }

    fn execute_update(&mut self) -> () {
        let mut db_lock = self.global_io.db.lock().unwrap();
        db_lock.update_engine_state(self.position.into()).unwrap()
    }

    fn enter_handler(&mut self, _: u8) -> Option<UiPages> {
        match self.current_selection {
            0 => {
                walk_engine(&mut self.global_io.gpio_engine, true, false);
            },
            2 => {
                walk_engine(&mut self.global_io.gpio_engine, false, false);
            },
            _ => (
                // To implement save
            ),
            
        }
        None
    }
    fn home_handler(&mut self, _: u8) -> Option<UiPages> {
        Some(UiPages::Menu1)
    }

}