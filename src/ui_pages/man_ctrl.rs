use std::sync::{Arc, Mutex};
use crate::{LCDdriver, GpioUi};
use crate::GlobalIoHandlers;
use crate::Level;
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

    fn teardown(&mut self) -> () { 
    }

    fn enter_handler(&mut self, _: u8) -> Option<UiPages> {
        let mut repeater= |go_right: bool| {
            let _global_io = self.global_io.clone();
            let input_lock = _global_io.gpio_ui.lock().unwrap();
            _global_io.gpio_engine.lock().unwrap().sleep.set_high();
            loop {
                walk_engine(&mut self.global_io.gpio_engine, go_right, None);
                if input_lock.enter.read() != Level::Low {
                    break
                }
            }
            _global_io.gpio_engine.lock().unwrap().sleep.set_low();
        };
        match self.current_selection {
            0 => {
                repeater(true);
            },
            1 => {
                let db_lock = self.global_io.db.lock().unwrap();
                db_lock.update_engin(self.global_io.active_preset,
                    Some(db_lock.get_application_state().unwrap().current_engine_state),
                    Some(true))
                .unwrap();
                return Some(UiPages::Menu1);
            },
            2 => {
                repeater(false);
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