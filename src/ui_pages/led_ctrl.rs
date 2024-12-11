use std::sync::{Arc, Mutex};
use db::schema::Led::{brightness, color};

use crate::{LCDdriver, GpioUi};
use crate::HashMap;
use crate::GlobalIoHandlers;
use crate::ui_pages::{ReactivePage, MenuPage, UiPages};
use crate::{LCDCommand, LCDArg, LCDProgramm};
use colors_transform::{Color, Hsl, Rgb};

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
        db_lock.update_led(self.global_io.active_preset,
            Some(&self.color),
            Some(self.brightness),
            Some(&self.mode)).unwrap();
    }

    fn enter_handler(&mut self, _: u8) -> Option<UiPages> {
        match self.current_selection {
           1 => {
            match self.setting {
                UiPages::LedColor => {
                
                        let hsl_color = Rgb::from_hex_str(&self.color).unwrap().to_hsl();
                        let hsl_color = Hsl::from((hsl_color.get_hue() + 10.0).min(359.0), hsl_color.get_saturation(), hsl_color.get_lightness());
                        self.color = format!("{:02x}{:02x}{:02x}", hsl_color.get_red() as u8, hsl_color.get_green() as u8, hsl_color.get_blue() as u8);
                    
                },
                UiPages::LedBrightness => {
                    self.brightness = (self.brightness + 10).min(100);
                },
                _ => {}
            }
        },
        2 => {
            match self.setting {
                UiPages::LedColor => {
                        let hsl_color = Rgb::from_hex_str(&self.color).unwrap().to_hsl();
                        let hsl_color = Hsl::from((hsl_color.get_hue() - 10.0).min(359.0), hsl_color.get_saturation(), hsl_color.get_lightness());
                        self.color = format!("{:02x}{:02x}{:02x}", hsl_color.get_red() as u8, hsl_color.get_green() as u8, hsl_color.get_blue() as u8);
                    
                },
                UiPages::LedBrightness => {
                    self.brightness = (self.brightness - 10).max(0);
                },
                _ => {}
            }
            },
           _ => {
                return self.return_to.iter().find(|x| x.0 == self.current_selection).map(|x| x.1);
           }
        }
        // |<^ xxxxxxxxxx v>|
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
            format!("{:03}/360", Rgb::from_hex_str(&self.color).unwrap().to_hsl().get_hue() as u8)
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
    fn pree_loop_hook(&mut self) -> () {
        self.print_user_info()
        // |<^ xxxxxxxxxx v>
    }
    fn change_hook(&mut self) -> () {
        self.print_user_info()
        // |<^ xxxxxxxxxx v>
    }
    
}