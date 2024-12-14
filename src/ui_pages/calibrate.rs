use std::hash::Hash;
use std::sync::{Arc, Mutex};
use rppal::gpio;

use crate::ui_pages::{MenuPage, UiPages, ReactivePage};
use crate::{walk_engine, GlobalIoHandlers};
use crate::{LCDdriver, GpioUi};
use crate::{LCDCommand, LCDArg, LCDProgramm};
use std::collections::HashMap;


pub (crate) struct CalibrationPage {
    pub(crate) global_io: GlobalIoHandlers,
    pub(crate) current_selection: usize,
}

impl MenuPage for CalibrationPage {
    
    fn get_lcd(&mut self) -> Arc<Mutex<LCDdriver>> {
        self.global_io.lcd.clone()
    }

    fn get_gpio_controller(&mut self) -> Arc<Mutex<GpioUi>> {
        self.global_io.gpio_ui.clone()
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
        return Some(UiPages::Menu1)
    }
    fn home_handler(&mut self, _: u8) -> Option<UiPages> {
        None
    }

    fn get_termination(&self) -> Option<UiPages> {
        None
    }

}

impl ReactivePage for CalibrationPage {
    fn pree_loop_hook(&mut self) -> Option<UiPages> {
        let lcd_binding = self.get_lcd();
        let mut lcd_lock = lcd_binding.lock().unwrap();
        let gpio_binding = self.get_gpio_controller();
        let mut gpio_lock = gpio_binding.lock().unwrap();

        let _ = lcd_lock.exec(LCDCommand { cmd: LCDProgramm::Move,
            args: Some({
                let mut map = HashMap::new();
                map.insert("y".to_string(), LCDArg::Int(1));
                map.insert("x".to_string(), LCDArg::Int(0));
                map
            }) });
            let _ = lcd_lock.exec(LCDCommand { cmd: LCDProgramm::Write,
            args: Some({
                let mut map = HashMap::new();
                map.insert("text".to_string(), LCDArg::String("Finding 0".to_string()));
                map
            }) });

        let mut pos_counnter: u64 = 0;

    

        self.global_io.gpio_engine.lock().unwrap().sleep.set_high();

        
            while walk_engine(&mut self.global_io.gpio_engine, true, None).1 == false {
                // do nothing
                if gpio_lock.enter.read() == gpio::Level::Low {
                    return Some(UiPages::Menu1)
                }            }
            let _ = lcd_lock.exec(LCDCommand { cmd: LCDProgramm::Move,
                args: Some({
                    let mut map = HashMap::new();
                    map.insert("y".to_string(), LCDArg::Int(1));
                    map.insert("x".to_string(), LCDArg::Int(0));
                    map
                }) });

            let _ = lcd_lock.exec(LCDCommand { cmd: LCDProgramm::Write,
                args: Some({
                    let mut map = HashMap::new();
                    map.insert("text".to_string(), LCDArg::String("Counting ESC".to_string()));
                    map
                }) });
            // calibration point found, counting steps
            while walk_engine(&mut self.global_io.gpio_engine, true, None).1 == true {
                pos_counnter = pos_counnter + 1;
                if gpio_lock.enter.read() == gpio::Level::Low {
                    return Some(UiPages::Menu1)
                }            }
            self.global_io.gpio_engine.lock().unwrap().sleep.set_low();
            let _ = lcd_lock.exec(LCDCommand { cmd: LCDProgramm::Move,
                args: Some({
                    let mut map = HashMap::new();
                    map.insert("y".to_string(), LCDArg::Int(0));
                    map.insert("x".to_string(), LCDArg::Int(0));
                    map
                }) });
            let _ = lcd_lock.exec(LCDCommand { cmd: LCDProgramm::Write,
                args: Some({
                    let mut map = HashMap::new();
                    map.insert("text".to_string(), LCDArg::String(format!("Round took steps{}", pos_counnter)));
                    map
                }) });
            
            let mut db_lock = self.global_io.db.lock().unwrap();
            db_lock.update_application_state(Some(0), None, Some(pos_counnter), None, None).unwrap();
            self.global_io.gpio_engine.lock().unwrap().update_steps_per_round(pos_counnter as u64);
            Some(UiPages::Menu1)
            
        }
        
    }    
    
