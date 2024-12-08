use std::sync::{Arc, Mutex};
use crate::{LCDdriver, GpioUi};
use crate::GlobalIoHandlers;
use crate::ui_pages::{MenuPage, UiPages};


pub (crate) struct SettingsMenu {
    pub(crate) global_io: GlobalIoHandlers,
    pub(crate) current_selection: usize,
}

impl MenuPage for SettingsMenu {
    
    fn get_lcd(&mut self) -> Arc<Mutex<LCDdriver>> {
        self.global_io.lcd.clone()
    }

    fn get_gpio_controller(&mut self) -> Arc<Mutex<GpioUi>> {
        self.global_io.gpio_ui.clone()
    }

    fn execute_update(&mut self) -> () {
        
    }
    fn get_current_selection(&self) -> usize {
        self.current_selection
    }

    fn enter_handler(&mut self, _: u8) -> Option<UiPages> {
        None
    }
    fn home_handler(&mut self, _: u8) -> Option<UiPages> {
        Some(UiPages::Menu1)
    }

    fn left_handler(&mut self, _: u8) -> Option<UiPages> {
        if self.current_selection > 0 {
            self.current_selection -= 1;
        }
        None
    }
    fn right_handler(&mut self, options_len: u8) -> Option<UiPages> {
        if self.current_selection < options_len as usize - 1 {
            self.current_selection += 1;
        }
        None
    }
}