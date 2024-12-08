use std::sync::{Arc, Mutex};
use crate::ui_pages::{MenuPage, UiPages};
use crate::GlobalIoHandlers;
use crate::{LCDdriver, GpioUi};



pub (crate) struct MainMenu {
    pub(crate) global_io: GlobalIoHandlers,
    pub(crate) current_selection: usize,
    pub(crate) return_to: Vec<UiPages>,
}

impl MenuPage for MainMenu {
    
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
        if self.current_selection < self.return_to.len() {
            return Some(self.return_to[self.current_selection]);
        }
        None
    }
    fn home_handler(&mut self, _: u8) -> Option<UiPages> {
        None
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