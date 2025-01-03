use crate::light_strip;
use crate::walk_engine;
use crate::GlobalIoHandlers;
use crate::GpioUi;
use std::sync::Mutex;
use super::MenuPage;
use super::ReactivePage;
use colors_transform::Color;
use lcd_driver::LCDdriver;
use std::sync::Arc;
use std::collections::HashMap;
use colors_transform::Rgb;

use crate::UiPages;

pub (crate) struct MoveToTarget {
    pub global_io: GlobalIoHandlers,
    pub current_selection: usize,
    pub target: i32,
    pub enter_pressed: bool,
}

impl MenuPage for MoveToTarget {
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


    fn enter_handler(&mut self, opt_len: u8) -> Option<UiPages> {
        self.enter_pressed = true;
        None    
    }

    fn get_termination(&self) -> Option<UiPages> {
        if let Ok(signal) = self.global_io.terminate.try_lock() {
            if let Some(page) = *signal {
                return Some(page);
            }
        }
        None
    }
}

impl MoveToTarget {
    fn loade_handler(&mut self, called_from: u32) -> Option<UiPages> {
        let db_bindig = self.global_io.db.clone();
        let mut db_lock = db_bindig.lock().unwrap();
        if called_from != 0 {
            // Meaning it was NOT called by the pre_loop_hook
            self.target = self.current_selection as i32 + 1;
        }

        if self.target != 0 {
            let resolved_target = db_lock.get_engine_preset(self.target);
            let new_pos = match resolved_target {
                Ok(preset) => {
                    let lcd_bindig = self.get_lcd();
                    let mut lcd_lock = lcd_bindig.lock().unwrap();
                    let _ = lcd_lock.exec(lcd_driver::LCDCommand { cmd: lcd_driver::LCDProgramm::Clear, args: None });
                    let _ = lcd_lock.exec(lcd_driver::LCDCommand { cmd: lcd_driver::LCDProgramm::Home , args: None});
                    let _ = lcd_lock.exec(lcd_driver::LCDCommand { cmd: lcd_driver::LCDProgramm::Write,
                        args: Some({
                            let mut map = HashMap::new();
                            map.insert("text".to_string(), lcd_driver::LCDArg::String(format!("Moving to:      {}", preset.position)));
                            map
                        })
                    });

                    let needed_sterps = |go_right: bool| -> i32 {
                        let mut steps = 0;
                        let mut current_pos = db_lock.get_application_state().unwrap().current_engine_pos;
                        let steps_per_round = self.global_io.gpio_engine.lock().unwrap().stepps_per_round;
                        let u = match go_right {
                            true => 1,
                            false => -1
                        };
                        loop {
                            if current_pos > steps_per_round  as i32{
                                current_pos = 0;
                            } else if current_pos < 0 {
                                current_pos = steps_per_round as i32;
                            }
                            if current_pos == preset.position {
                                break
                            }
                            current_pos = current_pos + u;
                            steps = steps + 1;
                        }
                        steps
                    };

                    let (right, left) = (needed_sterps(true), needed_sterps(false));
                    self.global_io.gpio_engine.lock().unwrap().sleep.set_high();
                    if right < left {
                        walk_engine(&mut self.global_io.gpio_engine, true, Some(right as u64));
                    } else {
                        walk_engine(&mut self.global_io.gpio_engine, false, Some(left as u64));
                    }
                    self.global_io.gpio_engine.lock().unwrap().sleep.set_low();
                    
                    Some(preset.position)
                },
                _ => {
                    let _ = db_lock.copy_engine_to_preset(self.target);
                    None
                }
            };
            let leds = db_lock.get_associated_led(self.target).unwrap_or(Vec::new());
            match leds.len() {
                0 => {
                    let _ = db_lock.copy_led_to_preset(self.target);
                },
                _ => {
                    let color = Rgb::from_hex_str(&leds[0].color).unwrap();
                    light_strip(&mut self.global_io.rgb_strip, &leds[0].mode,
                    Some([color.get_red() as u8, color.get_green() as u8, color.get_blue() as u8]), Some(leds[0].brightness as u8));    
                }
            }
            // Wee commit every time, to change the active preset
            db_lock.update_application_state(
                new_pos,
                Some(self.target),
                None,
                None,
                None,
            ).unwrap();
            *self.global_io.active_preset.lock().unwrap() = self.target;
            return Some(UiPages::Menu1);
        }
            
        None

        }
    }

impl ReactivePage for MoveToTarget {
    fn pree_loop_hook(&mut self) -> Option<UiPages> {
        if self.target != 0 {
            self.enter_handler(0);
            return Some(UiPages::Menu1);
        }
        None
    }
    fn change_hook(&mut self) -> Option<UiPages> {
        if self.enter_pressed {
            self.enter_pressed = false;
            return self.loade_handler(1)
        }
        None
    }
}
