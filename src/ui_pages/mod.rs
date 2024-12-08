
pub (crate) mod man_ctrl;
pub (crate) mod menu;
pub (crate) mod settings;
pub(crate) mod led_ctrl;

use crate::Duration;
use crate::thread;
use crate::Level;
use crate::HashMap;
use crate::{Arc, Mutex};

use crate::GpioUi;
use crate::{LCDCommand, LCDArg, LCDProgramm, LCDdriver};
use crate::USER_INPUT_DELAY;

#[derive(Debug, Clone, Copy)]
pub (crate) enum UiPages {
    Menu1,
    Menu2,
    LedColor,
    LedBrightness,
    LedMode,
    SettingsMenu,
    ManualControll,
}



pub (crate) trait MenuPage {
    fn watch_loop(&mut self, text: &str, option: Vec<(u8, u8)>) -> UiPages {
        thread::sleep(Duration::from_millis(USER_INPUT_DELAY));
        let lcd_binding = self.get_lcd();
        let gpio_binding = self.get_gpio_controller();
        let mut lcd_lock = lcd_binding.lock().unwrap();
        let gpio_lock = gpio_binding.lock().unwrap();

        let _ = lcd_lock.exec(LCDCommand{
            cmd: LCDProgramm::Clear,
            args: None
        });

        let _ = lcd_lock.exec(LCDCommand{
            cmd: LCDProgramm::Home,
            args: None
        });

        let _ = lcd_lock.exec(LCDCommand{
            cmd: LCDProgramm::Write,
            args: Some({
                let mut map = HashMap::new();
                map.insert("text".to_string(), LCDArg::String(text.to_string()));
                map
            })
        });
       
        let mut last_selection: i16 = -2;
        loop {
            let actions: [(_, Box<dyn Fn(&mut Self, u8) -> Option<UiPages>>); 4] = [
                (gpio_lock.home.read(), Box::new(|s, o| MenuPage::home_handler(s, o))),
                (gpio_lock.left.read(), Box::new(|s, o| MenuPage::left_handler(s, o))),
                (gpio_lock.right.read(), Box::new(|s, o| MenuPage::right_handler(s, o))),
                (gpio_lock.enter.read(), Box::new(|s, o| MenuPage::enter_handler(s, o))),
            ];
            

            for (level, handler) in actions.iter() {
                if *level == Level::Low {
                    if let Some(page) = handler(self, option.len() as u8) {
                        self.execute_update();
                        return page;
                    }    
                }
            }
            if self.get_current_selection() as i16 != last_selection {
                let c_selection = self.get_current_selection();
                let _ = lcd_lock.exec(LCDCommand{
                    cmd: LCDProgramm::Move,
                    args: Some({
                        let mut map = HashMap::new();
                        map.insert("y".to_string(), LCDArg::Int(1));
                        map.insert("x".to_string(), LCDArg::Int(0));
                        map
                    })
                });
                let _ = lcd_lock.exec(LCDCommand{
                    cmd: LCDProgramm::Write,
                    args: Some({
                        let mut map = HashMap::new();
                        map.insert("text".to_string(), LCDArg::String(format!("{}{}{}",
                        " ".repeat(option[c_selection].0 as usize),
                        "_".repeat((option[c_selection].1 - option[c_selection].0) as usize),
                        " ".repeat((16 -  option[c_selection].1) as usize)
                        )
                            ));
                        map
                    })
                });
                last_selection = self.get_current_selection() as i16;
                thread::sleep(Duration::from_millis(USER_INPUT_DELAY));
            }
            
        }
    }
    fn get_gpio_controller(&mut self) -> Arc<Mutex<GpioUi>>;
    fn get_lcd(&mut self) -> Arc<Mutex<LCDdriver>>;
    fn get_current_selection(&self) -> usize;
    fn set_current_selection(&mut self, selection: usize) -> ();
    fn execute_update(&mut self) -> ();

    fn home_handler(&mut self, options_len: u8) -> Option<UiPages> {
        Some(UiPages::Menu1)
    }
    fn left_handler(&mut self, options_len: u8) -> Option<UiPages>{
        if self.get_current_selection() > 0 {
            self.set_current_selection(self.get_current_selection() - 1);
        }
        None
    }
    fn right_handler(&mut self, options_len: u8) -> Option<UiPages> {
        if self.get_current_selection() < options_len as usize - 1 {
            self.set_current_selection(self.get_current_selection() + 1);
        }
        None
    }
    fn enter_handler(&mut self, options_len: u8) -> Option<UiPages>;
}
