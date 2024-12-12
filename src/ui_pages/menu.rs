use std::sync::{Arc, Mutex};
use crate::ui_pages::{MenuPage, UiPages};
use crate::GlobalIoHandlers;
use crate::{LCDdriver, GpioUi};



pub (crate) struct MainMenu {
    pub(crate) global_io: GlobalIoHandlers,
    pub(crate) current_selection: usize,
    pub(crate) return_to: Vec<UiPages>,
}

impl <'a> MenuPage<'a> for MainMenu {
    
    fn get_lcd(&'a mut self) -> &'a mut LCDdriver {
        &mut self.global_io.lcd
    }

    fn get_gpio_ui(&'a mut self) -> &'a mut GpioUi {
        &mut self.global_io.gpio_ui
    }

    fn get_global_io(&'a mut self) -> GlobalIoHandlers {
        self.global_io
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
        if self.current_selection < self.return_to.len() {
            return Some(self.return_to[self.current_selection]);
        }
        None
    }
    fn home_handler(&mut self, _: u8) -> Option<UiPages> {
        None
    }

}