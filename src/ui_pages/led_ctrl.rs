use std::sync::{Arc, Mutex};

use crate::{LCDdriver, GpioUi};
use crate::HashMap;
use crate::GlobalIoHandlers;
use crate::ui_pages::{ReactivePage, MenuPage, UiPages};
use crate::{LCDCommand, LCDArg, LCDProgramm};
use colors_transform::{Color, Hsl, Rgb};
use crate::light_strip;

pub (crate) struct LedCtrlPage {
    pub (crate) global_io: GlobalIoHandlers,
    pub (crate) current_selection: usize,
    
    pub (crate) return_to: Vec<(usize, UiPages)>, // (selection, page)
    pub (crate) color: String,
    pub (crate) brightness: u8,
    pub (crate) mode: String,
    pub (crate) setting: UiPages
}

impl MenuPage for LedCtrlPage {
    
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
        let db_lock = self.global_io.db.lock().unwrap();
        db_lock.update_led(*self.global_io.active_preset.lock().unwrap(),
            Some(&self.color),
            Some(self.brightness),
            Some(&self.mode)).unwrap();
    }

    fn enter_handler(&mut self, _: u8) -> Option<UiPages> {
        
        if let Some(page) = self.return_to.iter().filter(|(sel, _)| *sel == self.current_selection).map(|(_, page)| *page).nth(0){
            return Some(page);
        }
        
        let brightness_modifyer = |_color: &str, change_by: f32| -> (String, [u8; 3]) {
            let hsl_color = Rgb::from_hex_str(_color).unwrap().to_hsl();
            let hsl_color = Hsl::from((hsl_color.get_hue() + change_by).min(359.0), hsl_color.get_saturation(), hsl_color.get_lightness());
            (format!("{:02x}{:02x}{:02x}", hsl_color.get_red() as u8, hsl_color.get_green() as u8, hsl_color.get_blue() as u8), 
            [hsl_color.get_red() as u8, hsl_color.get_green() as u8, hsl_color.get_blue() as u8])
        };
        let mut new_color = None;
        let mut new_brightness = None;
        match self.setting {
            UiPages::LedColor => {
            new_color = match self.current_selection {
                1 => Some(brightness_modifyer(&self.color, 10.0)),
                2 => Some(brightness_modifyer(&self.color, -10.0)),
                _ => None
            }.and_then(|v| {
                self.color = v.0;
                Some(v.1)
            });
            },
            UiPages::LedBrightness => {
            new_brightness = match self.current_selection{
                1 => Some((self.brightness + 10).min(100)),
                2 => Some((self.brightness - 10).max(0)),
                _ => None
            }.and_then(|v| {
                self.brightness = v;
                Some(v)
            });
            },
            _ => {}
        }
        if new_color.is_some() || new_brightness.is_some() {
            light_strip(&mut self.global_io.rgb_strip, &self.mode, new_color.or_else(||Some(brightness_modifyer(&self.color, 0.0).1)), new_brightness);
        }
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

impl LedCtrlPage {
    fn print_user_info(&mut self) -> (){
        let mut lcd_lock = self.global_io.lcd.lock().unwrap();
        let _ = lcd_lock.exec(LCDCommand{
            cmd: LCDProgramm::Move,
            args: Some({
                let mut map = HashMap::new();
                map.insert("y".to_string(), LCDArg::Int(1));
                map.insert("x".to_string(), LCDArg::Int(4));
                map
            })
        });

        let info = match self.setting {
            UiPages::LedColor => {
            format!("{:03}/360", Rgb::from_hex_str(&self.color).unwrap().to_hsl().get_hue() as u16)
            },
            UiPages::LedBrightness => {
            format!("{:03}%", self.brightness)
            },
            _ => {
            format!("NA")
            }
        };
        
        let _ = lcd_lock.exec(LCDCommand{
            cmd: LCDProgramm::Write,
            args: Some({
                let mut map = HashMap::new();
                map.insert("text".to_string(), LCDArg::String(info));
                map
            })
        });
    }
    
}

impl ReactivePage for LedCtrlPage {
    fn pree_loop_hook(&mut self) -> Option<UiPages> {
        self.print_user_info();
        None
        // |<^ xxxxxxxxxx v>
    }
    fn change_hook(&mut self) -> Option<UiPages> {
        self.print_user_info();
        None
        // |<^ xxxxxxxxxx v>
    }
    
}