
pub (crate) mod man_ctrl;
pub (crate) mod menu;
pub (crate) mod settings;
pub(crate) mod led_ctrl;


use crate::Duration;
use crate::thread;
use crate::GlobalIoHandlers;
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


pub (crate) trait MenuPage<'a> {
    fn main_handler(
        &'a mut self,
        text: &str,
        option: Vec<(u8, u8)>,
        pre_loop_hook: Option<Box<dyn Fn(&mut Self) + 'a>>,
        loop_hook: Option<Box<dyn Fn(&mut Self) + 'a>>,
        change_hook: Option<Box<dyn Fn(&mut Self) + 'a>>,
    ) -> (UiPages, GlobalIoHandlers)
    {
        thread::sleep(Duration::from_millis(USER_INPUT_DELAY));

        // Clear and prepare the LCD
        let _ = self.get_lcd().exec(LCDCommand {
            cmd: LCDProgramm::Clear,
            args: None,
        });

        let _ = self.get_lcd().exec(LCDCommand {
            cmd: LCDProgramm::Home,
            args: None,
        });

        let _ = self.get_lcd().exec(LCDCommand {
            cmd: LCDProgramm::Write,
            args: Some({
                let mut map = HashMap::new();
                map.insert("text".to_string(), LCDArg::String(text.to_string()));
                map
            }),
        });

        let mut last_selection: i16 = -2;

        loop {
            let loop_start_time = std::time::Instant::now();
            let mut change = false;
            {
                if let Some(ref func) = loop_hook {
                    func(self);
            }
            }
            // Define action handlers
            let actions: [(_, Box<dyn Fn(&mut Self, u8) -> Option<UiPages>>); 4] = [
                (self.get_gpio_ui().home.read(), Box::new(Self::home_handler)),
                (self.get_gpio_ui().left.read(), Box::new(Self::left_handler)),
                (self.get_gpio_ui().right.read(), Box::new(Self::right_handler)),
                (self.get_gpio_ui().enter.read(), Box::new(Self::enter_handler)),
            ];

           

            // Check for actions and execute handlers
            for (level, handler) in actions.iter() {
                if *level == Level::Low {
                    if let Some(page) = handler(self, option.len() as u8) {
                        self.teardown();
                        return (page, self.get_global_io());
                    }
                    change = true;
                }
            }

            // Handle selection changes
            let current_selection = self.get_current_selection() as i16;
            if current_selection != last_selection {
                let _ = self.get_lcd().exec(LCDCommand {
                    cmd: LCDProgramm::Move,
                    args: Some({
                        let mut map = HashMap::new();
                        map.insert("y".to_string(), LCDArg::Int(1));
                        map.insert("x".to_string(), LCDArg::Int(0));
                        map
                    }),
                });

                let _ = self.get_lcd().exec(LCDCommand {
                    cmd: LCDProgramm::Write,
                    args: Some({
                        let mut map = HashMap::new();
                        map.insert(
                            "text".to_string(),
                            LCDArg::String(format!(
                                "{}{}{}",
                                " ".repeat(option[current_selection as usize].0 as usize),
                                "_".repeat(
                                    (option[current_selection as usize].1 - option[current_selection as usize].0)
                                        as usize
                                ),
                                " ".repeat((16 - option[current_selection as usize].1) as usize)
                            )),
                        );
                        map
                    }),
                });

                if last_selection == -2 {
                    if let Some(ref func) = pre_loop_hook {
                        func(self);
                    }
                }
                last_selection = current_selection;
            }

            // Call the change hook if needed
            if change {
                if let Some(ref func) = change_hook {
                    func(self);
                }
            }

            // Sleep to maintain consistent loop timing
            if let Some(remaining) = Duration::from_millis(USER_INPUT_DELAY).checked_sub(loop_start_time.elapsed()) {
                thread::sleep(remaining);
            }
        }
    }
    fn watch_loop(&mut self, text: &str, option: Vec<(u8, u8)>) -> (UiPages, GlobalIoHandlers) 
     {
        self.main_handler(text, option, None, None, None)
    }
    
    fn get_lcd(&'a mut self) -> &'a mut LCDdriver;
    fn get_gpio_ui(&'a mut self) -> &'a mut GpioUi;
    fn get_global_io(&'a mut self) -> GlobalIoHandlers;

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

pub (crate) trait ReactivePage<'b, 'a>: MenuPage<'a>
where 'a: 'b {
    fn loop_hook(&mut self) -> (){}
    fn change_hook(&mut self) -> (){}
    fn pree_loop_hook(&mut self) -> (){}

    fn reactive_watch(&mut self, text: &str, option: Vec<(u8, u8)>) -> (UiPages, GlobalIoHandlers) {
        self.main_handler(text, option, Some(Box::new(|s: &mut Self| Self::pree_loop_hook(s))), Some(Box::new(|s: &mut Self| Self::loop_hook(s))), Some(Box::new(|s: &mut Self| Self::change_hook(s))))
    }
    
}