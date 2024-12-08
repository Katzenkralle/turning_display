use std::sync::{Arc, Mutex};
use db::schema::Led::{brightness, color};

use crate::{LCDdriver, GpioUi};
use crate::HashMap;
use crate::GlobalIoHandlers;
use crate::ui_pages::{MenuPage, UiPages};
use crate::{LCDCommand, LCDArg, LCDProgramm};



fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

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
    pub (crate) color: Option<String>,
    pub (crate) brightness: Option<u8>,
    pub (crate) mode: Option<String>,
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

    fn execute_update(&mut self) -> () {
        let db_lock = self.global_io.db.lock().unwrap();
        let led = db_lock.get_leds(None).unwrap().iter().map(|x| x.id).collect();
        db_lock.update_led(led, self.color.as_ref(), self.brightness, self.mode.as_ref()).unwrap();
    }

    fn enter_handler(&mut self, _: u8) -> Option<UiPages> {
        let mut userinfo = "".to_string();
        match self.current_selection {
           1 => {
               if let Some(rgb_coler) = &self.color {
                    let mut hsl_color = rgb_to_hsl(rgb_coler[..1].parse().unwrap(),
                        rgb_coler[2..3].parse().unwrap(),
                        rgb_coler[4..5].parse().unwrap());
                    hsl_color.0 = (hsl_color.0 + 10.0).rem_euclid(360.0);
                    userinfo = hsl_color.0.to_string();
                    self.color = Some(hsl_to_rgb_string(hsl_color.0, hsl_color.1, hsl_color.2));
                }
                if let Some(bright) = self.brightness {
                    self.brightness = Some((bright + 10).min(100));
                    userinfo = bright.to_string();
                }
              },
            2 => {
            if let Some(rgb_coler) = &self.color {
                    let mut hsl_color = rgb_to_hsl(rgb_coler[..1].parse().unwrap(),
                        rgb_coler[2..3].parse().unwrap(),
                        rgb_coler[4..5].parse().unwrap());
                    hsl_color.0 = (hsl_color.0 - 10.0).rem_euclid(360.0);
                    userinfo = hsl_color.0.to_string();
                    self.color = Some(hsl_to_rgb_string(hsl_color.0, hsl_color.1, hsl_color.2));
                }
                if let Some(bright) = self.brightness {
                    self.brightness = Some((bright - 10).max(0));
                    userinfo = bright.to_string();
                }
            },
           _ => {
                return self.return_to.iter().find(|x| x.0 == self.current_selection).map(|x| x.1);
           }
        }
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
        let _ = lcd_lock.exec(LCDCommand{
            cmd: LCDProgramm::Write,
            args: Some({
                let mut map = HashMap::new();
                map.insert("text".to_string(), LCDArg::String(format!("{}",
                    userinfo
                )
                    ));
                map
            })
        });
        // |<^ xxxxxxxxxx v>|
        None
    }
    
}