use std::sync::{Arc, Mutex};
use crate::{LCDdriver, GpioUi};
use crate::GlobalIoHandlers;
use crate::Level;
use crate::ui_pages::{MenuPage, UiPages};
use crate::walk_engine;
use crate::STEPS_PER_ROUND;

pub (crate) struct ManualControllPage {
    pub (crate)  global_io: GlobalIoHandlers,
    pub (crate)  current_selection: usize,
    pub (crate)  position: u8,
}
impl <'a> MenuPage<'a> for ManualControllPage {
    
    fn get_lcd(&mut self) -> &'a mut LCDdriver {
        &'a mut self.global_io.lcd
    }


    fn get_gpio_ui(&'a mut self) -> &'a mut GpioUi {
        &mut self.global_io.gpio_ui
    }

    fn get_global_io(&mut self) -> GlobalIoHandlers {
        self.global_io
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
            let mut acumulated_distance = 0;
            _global_io.gpio_engine.lock().unwrap().sleep.set_high();
            loop {
                acumulated_distance = acumulated_distance + walk_engine(&mut self.global_io.gpio_engine, go_right, None);
                if input_lock.enter.read() != Level::Low {
                    break
                }
            }
            acumulated_distance = self.global_io.db.lock().unwrap().get_application_state().unwrap().current_engine_state + acumulated_distance;
            if acumulated_distance > STEPS_PER_ROUND {
                acumulated_distance = acumulated_distance - STEPS_PER_ROUND;
            } else if acumulated_distance < 0 {
                acumulated_distance = STEPS_PER_ROUND + acumulated_distance;
            }
            self.global_io.db.lock().unwrap().update_application_state(
                Some(acumulated_distance),
                None)
                .unwrap();
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