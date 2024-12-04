
use db::DbConn;
use lcd_driver::{LCDdriver, LCDCommand, LCDProgramm, LCDArg};
use std::{any::{self, Any}, path::Path, str, sync::mpsc::Receiver, thread::{self, JoinHandle}};
use std::sync::mpsc;
use std::collections::HashMap;
use rppal::gpio::{self, Gpio, InputPin, Level, Mode};
use std::sync::Arc;
use std::sync::Mutex;

fn rebuild_test_db() {
    const TEST_LEDS: [(i32, i32, &str); 4] = [
        (10, 10, "#ff0000"),
        (10, -10, "#00ff00"),
        (-10, 10, "#0000ff"),
        (-10, -10, "#ffffff"),
    ];
    
    let mut connection = DbConn::establish_connection();
    let led = connection.get_leds(None).unwrap();
    led.iter().for_each(|l| {
        connection.del_led(l.id).expect("Error deleting led");
    });

    for (x, y, color) in TEST_LEDS.iter() {
        connection.add_led(*x, *y, color.to_string()).expect("Error adding led");
    }
}



// Pinout:
// Home > 17
// Left > 27
// Right > 22
// Enter > 24
// >> Pull-down
// Step Motor:
// PWM: 18
// DIR: 23
// STEP: 25
// ENA: 12

// I2C: 3, 5
// > LCD: 0x27
// r

struct GPIOControllerIo {
    home: InputPin,
    left: InputPin,
    right: InputPin,
    enter: InputPin,
}

trait MenuPage {
    fn watch_loop(&mut self, text: &str, option: Vec<(u8, u8)>) -> MenuPages {
        let _ = self.get_lcd().exec(LCDCommand{
            cmd: LCDProgramm::Write,
            args: Some({
                let mut map = HashMap::new();
                map.insert("text".to_string(), LCDArg::String(text.to_string()));
                map
            })
        });
        loop {
        let actions = [
            (self.get_gpio_controller().home.read(), self.home_handler(option.len() as u8)),
            (self.get_gpio_controller().left.read(), self.left_handler(option.len() as u8)),
            (self.get_gpio_controller().right.read(), self.right_handler(option.len() as u8)),
            (self.get_gpio_controller().enter.read(), self.enter_handler(option.len() as u8)),
        ];

        for (level, handler) in actions.iter() {
            if *level == Level::High {
                if let Some(page) = handler {
                    return *page;
                }
                let c_selection = self.get_current_selection();
                let _ = self.get_lcd().exec(LCDCommand{
                    cmd: LCDProgramm::Move,
                    args: Some({
                        let mut map = HashMap::new();
                        map.insert("x".to_string(), LCDArg::Int(0));
                        map.insert("y".to_string(), LCDArg::Int(option[c_selection].0 as i128));
                        map
                    })
                });
                let _ = self.get_lcd().exec(LCDCommand{
                    cmd: LCDProgramm::Write,
                    args: Some({
                        let mut map = HashMap::new();
                        map.insert("text".to_string(), LCDArg::String("_".repeat(
                            (option[c_selection].1 - option[c_selection].0) as usize)
                            ));
                        map
                    })
                });
            }
        }
        }
    }
    fn get_gpio_controller(&mut self) -> &mut GPIOControllerIo;
    fn get_lcd(&mut self) -> &mut LCDdriver;
    fn get_current_selection(&self) -> usize;


    fn home_handler(&mut self, options_len: u8) -> Option<MenuPages>;
    fn left_handler(&mut self, options_len: u8) -> Option<MenuPages>;
    fn right_handler(&mut self, options_len: u8) -> Option<MenuPages>;
    fn enter_handler(&mut self, options_len: u8) -> Option<MenuPages>;
}

#[derive(Debug, Clone, Copy)]
enum MenuPages {
    MainMenu,
    SettingsMenu,
}


struct MainMenu {
    global_broadcast: mpsc::Sender<String>,
    lcd: LCDdriver,
    gpio_controller: GPIOControllerIo,
    current_selection: usize,
}

impl MenuPage for MainMenu {
    
    fn get_lcd(&mut self) -> &mut LCDdriver {
        &mut self.lcd
    }

    fn get_gpio_controller(&mut self) -> &mut GPIOControllerIo {
        &mut  self.gpio_controller
    }

    fn get_current_selection(&self) -> usize {
        self.current_selection
    }

    fn enter_handler(&mut self, _: u8) -> Option<MenuPages> {
        match self.current_selection {
            0 => None,
            1 => Some(MenuPages::SettingsMenu),
            _ => None,
        }
    }
    fn home_handler(&mut self, _: u8) -> Option<MenuPages> {
        None
    }

    fn left_handler(&mut self, _: u8) -> Option<MenuPages> {
        if self.current_selection > 0 {
            self.current_selection -= 1;
        }
        None
    }
    fn right_handler(&mut self, options_len: u8) -> Option<MenuPages> {
        if self.current_selection < options_len as usize - 1 {
            self.current_selection += 1;
        }
        None
    }
}


struct SettingsMenu {
    global_broadcast: mpsc::Sender<String>,
    lcd: LCDdriver,
    gpio_controller: GPIOControllerIo,
    current_selection: usize,
}

impl MenuPage for SettingsMenu {
    
    fn get_lcd(&mut self) -> &mut LCDdriver {
        &mut self.lcd
    }

    fn get_gpio_controller(&mut self) -> &mut GPIOControllerIo {
        &mut  self.gpio_controller
    }

    fn get_current_selection(&self) -> usize {
        self.current_selection
    }

    fn enter_handler(&mut self, _: u8) -> Option<MenuPages> {
        None
    }
    fn home_handler(&mut self, _: u8) -> Option<MenuPages> {
        Some(MenuPages::MainMenu)
    }

    fn left_handler(&mut self, _: u8) -> Option<MenuPages> {
        if self.current_selection > 0 {
            self.current_selection -= 1;
        }
        None
    }
    fn right_handler(&mut self, options_len: u8) -> Option<MenuPages> {
        if self.current_selection < options_len as usize - 1 {
            self.current_selection += 1;
        }
        None
    }
}


fn main_prosessing_loop() -> () {
        let (tx, rx) = mpsc::channel::<String>();
    
        let mut requested_menu = MenuPages::MainMenu;
        let mut menu_page_thread: Option<JoinHandle<MenuPages>> = None;
    
        loop {
            // Handle completed threads
            if let Some(thread) = menu_page_thread.take() {
                if !thread.is_finished() {
                    menu_page_thread = Some(thread);
                    continue;
                }
                requested_menu = thread.join().unwrap();
            }
            
            // Match the requested menu and start a new thread
            let _tx = tx.clone();
            menu_page_thread = Some(match requested_menu {
                MenuPages::MainMenu => 
                    thread::spawn(move || {
                        let gpio = Gpio::new().unwrap();

                        SettingsMenu {
                        global_broadcast: _tx,
                        lcd:LCDdriver::new(Path::new("lcd_driver/lcd.sock"), true).unwrap(),
                        gpio_controller: GPIOControllerIo {
                            home: gpio.get(17).unwrap().into_input(),
                            left: gpio.get(27).unwrap().into_input(),
                            right: gpio.get(22).unwrap().into_input(),
                            enter: gpio.get(24).unwrap().into_input(),
                        },
                        current_selection: 0,
                    }.watch_loop("<Ctrl.     Set.>", vec![(0, 6), (12, 15)])}),         
                MenuPages::SettingsMenu => 
                    thread::spawn(move || {
                        let gpio = Gpio::new().unwrap();

                        SettingsMenu {
                            global_broadcast: _tx,
                            lcd: LCDdriver::new(Path::new("lcd_driver/lcd.sock"), true).unwrap(),
                            gpio_controller: GPIOControllerIo {
                                home: gpio.get(17).unwrap().into_input(),
                                left: gpio.get(27).unwrap().into_input(),
                                right: gpio.get(22).unwrap().into_input(),
                                enter: gpio.get(24).unwrap().into_input(),
                            },
                            current_selection: 0,
                        }.watch_loop("<  Automatic ON?", vec![(0, 1), (12, 15)])
                    })
            });
        }
    }

    


fn main() {
    rebuild_test_db();
    main_prosessing_loop();
}
