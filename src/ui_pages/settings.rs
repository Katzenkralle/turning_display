use std::sync::{Arc, Mutex};
use crate::{LCDdriver, GpioUi};
use crate::GlobalIoHandlers;
use crate::ui_pages::{MenuPage, UiPages};


pub (crate) struct SettingsMenu {
    pub(crate) global_io: GlobalIoHandlers,
    pub(crate) current_selection: usize,
}

impl <'a> MenuPage<'a> for SettingsMenu {
    
    fn get_lcd(&'a mut self) -> &'a mut LCDdriver {
        &mut self.global_io.lcd
    }

    fn get_gpio_ui(&'a mut self) -> &'a mut GpioUi {
        &mut self.global_io.gpio_ui
    }

    fn teardown(&mut self) -> () {  
    }

    fn set_current_selection(&mut self, selection: usize) -> () {
        self.current_selection = selection;
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

}