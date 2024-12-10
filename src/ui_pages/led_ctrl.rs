use std::sync::{Arc, Mutex};
use db::schema::Led::{brightness, color};

use crate::{LCDdriver, GpioUi};
use crate::HashMap;
use crate::GlobalIoHandlers;
use crate::ui_pages::{ReactivePage, MenuPage, UiPages};
use crate::{LCDCommand, LCDArg, LCDProgramm};
use colors_transform::{Color, Hsl, Rgb};


fn rgb_to_hsl(hex: &String) -> (f32, f32, f32) {
    
    let r = u8::from_str_radix(&hex[0..1], 16).unwrap() as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..3], 16).unwrap() as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..5], 16).unwrap() as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    // Calculate Lightness
    let l = (max + min) / 2.0;

    // Calculate Saturation
    let s = if delta == 0.0 {
        0.0
    } else {
        delta / (1.0 - (max + min - 1.0).abs())
    };

    // Calculate Hue
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        (g - b) / delta + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    let h = (h * 60.0).rem_euclid(360.0); // Ensure hue is in the range [0, 360)

    (h, s, l)
}

fn hsl_to_rgb_string(h: f32, s: f32, l: f32) -> String {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r_prime, g_prime, b_prime) = if h >= 0.0 && h < 60.0 {
        (c, x, 0.0)
    } else if h >= 60.0 && h < 120.0 {
        (x, c, 0.0)
    } else if h >= 120.0 && h < 180.0 {
        (0.0, c, x)
    } else if h >= 180.0 && h < 240.0 {
        (0.0, x, c)
    } else if h >= 240.0 && h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let r = ((r_prime + m) * 255.0).round() as u8;
    let g = ((g_prime + m) * 255.0).round() as u8;
    let b = ((b_prime + m) * 255.0).round() as u8;

    format!("{r:02x}{g:02x}{b:02x}")
}

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