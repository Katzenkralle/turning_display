
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
    fn main_handler(&mut self, text: &str, option: Vec<(u8, u8)>, pree_loop_hook: Option<Box<dyn Fn(&mut Self) -> ()>>, loop_hook: Option<Box<dyn Fn(&mut Self) -> ()>>, change_hook: Option<Box<dyn Fn(&mut Self) -> ()>>) -> UiPages
    {
        thread::sleep(Duration::from_millis(USER_INPUT_DELAY));
        let lcd_binding = self.get_lcd();
        let gpio_binding = self.get_gpio_controller();
        let mut lcd_lock = Some(lcd_binding.lock().unwrap());
        let mut gpio_lock = Some(gpio_binding.lock().unwrap());

        let _ = lcd_lock.as_mut().unwrap().exec(LCDCommand{
            cmd: LCDProgramm::Clear,
            args: None
        });

        let _ = lcd_lock.as_mut().unwrap().exec(LCDCommand{
            cmd: LCDProgramm::Home,
            args: None
        });

        let _ = lcd_lock.as_mut().unwrap().exec(LCDCommand{
            cmd: LCDProgramm::Write,
            args: Some({
                let mut map = HashMap::new();
                map.insert("text".to_string(), LCDArg::String(text.to_string()));
                map
            })
        });
        let mut last_selection: i16 = -2;
        loop {
            let loop_start_time = std::time::Instant::now();
            let mut change = false;
            let actions: [(_, Box<dyn Fn(&mut Self, u8) -> Option<UiPages>>); 4] = [
                (gpio_lock.as_mut().unwrap().home.read(), Box::new(|s, o| MenuPage::home_handler(s, o))),
                (gpio_lock.as_mut().unwrap().left.read(), Box::new(|s, o| MenuPage::left_handler(s, o))),
                (gpio_lock.as_mut().unwrap().right.read(), Box::new(|s, o| MenuPage::right_handler(s, o))),
                (gpio_lock.as_mut().unwrap().enter.read(), Box::new(|s, o| MenuPage::enter_handler(s, o))),
            ];
            
            if let Some(ref func) = loop_hook {
                lcd_lock = None;
                func(self);
                lcd_lock = Some(lcd_binding.lock().unwrap());
            }
            for (level, handler) in actions.iter() {
                if *level == Level::Low {
                    gpio_lock = None;
                    if let Some(page) = handler(self, option.len() as u8) {
                        self.teardown();
                        return page;
                    }
                    gpio_lock = Some(gpio_binding.lock().unwrap());
                    change = true;
                }
            }
            if self.get_current_selection() as i16 != last_selection {
                let c_selection = self.get_current_selection();
                let _ = lcd_lock.as_mut().unwrap().exec(LCDCommand{
                    cmd: LCDProgramm::Move,
                    args: Some({
                        let mut map = HashMap::new();
                        map.insert("y".to_string(), LCDArg::Int(1));
                        map.insert("x".to_string(), LCDArg::Int(0));
                        map
                    })
                });
                let _ = lcd_lock.as_mut().unwrap().exec(LCDCommand{
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
                if last_selection == -2 {
                    if let Some(ref func) = pree_loop_hook {
                        lcd_lock = None;
                        func(self);
                        lcd_lock = Some(lcd_binding.lock().unwrap());
                    }
                }
                last_selection = self.get_current_selection() as i16;
            }
            if change {
                if let Some(ref func) = change_hook {
                    lcd_lock = None;
                    func(self);
                    lcd_lock = Some(lcd_binding.lock().unwrap());
                }
                if loop_start_time.elapsed() < Duration::from_millis(USER_INPUT_DELAY) {
                    thread::sleep(Duration::from_millis(USER_INPUT_DELAY) - loop_start_time.elapsed());
                }
                //thread::sleep(Duration::from_millis(USER_INPUT_DELAY));

            }
            
        }
    }
    fn watch_loop(&mut self, text: &str, option: Vec<(u8, u8)>) -> UiPages {
        self.main_handler(text, option, None, None, None)
    }
    fn get_gpio_controller(&mut self) -> Arc<Mutex<GpioUi>>;
    fn get_lcd(&mut self) -> Arc<Mutex<LCDdriver>>;
    fn get_current_selection(&self) -> usize;
    fn set_current_selection(&mut self, selection: usize) -> ();
    fn teardown(&mut self) -> ();

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


pub (crate) trait ReactivePage: MenuPage + 'static {
    fn loop_hook(&mut self) -> (){}
    fn change_hook(&mut self) -> (){}
    fn pree_loop_hook(&mut self) -> (){}

    fn reactive_watch(&mut self, text: &str, option: Vec<(u8, u8)>) -> UiPages {
        self.main_handler(text, option, Some(Box::new(Self::pree_loop_hook)), Some(Box::new(Self::loop_hook)), Some(Box::new(Self::change_hook)))
    }
    
}
