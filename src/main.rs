
use db::DbConn;
use lcd_driver::{LCDdriver, LCDCommand, LCDProgramm, LCDArg};
use std::{path::Path, ptr::null, str, thread::{self, JoinHandle}};
use crossbeam::channel::unbounded;
use std::collections::HashMap;
use rppal::gpio::{Gpio, InputPin, Level, OutputPin};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

const USER_INPUT_DELAY: u64 = 200;

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

fn walk_engine(engine: &mut Arc<Mutex<GpioEngine>>, go_right: bool, use_pwm: bool) -> i16 {
    let mut lock = engine.lock().unwrap();
    let mut delta_pos = 1;
    if go_right {
        lock.dir.write(Level::Low);
        delta_pos *= -1;
    } else {
        lock.dir.write(Level::High);
    }
    if use_pwm {
        
        // to do
    } else {
        lock.step.write(Level::High);
        thread::sleep(std::time::Duration::from_millis(1));
        lock.step.write(Level::Low);
        thread::sleep(std::time::Duration::from_millis(1));
    }
    delta_pos
}
struct GpioUi {
    home: InputPin,
    left: InputPin,
    right: InputPin,
    enter: InputPin,
}

struct GpioEngine {
    dir: OutputPin,
    step: OutputPin,
    ena: OutputPin
}

#[derive( Clone)]
struct GlobalIoHandlers {
    lcd: Arc<Mutex<LCDdriver>>,
    gpio_ui: Arc<Mutex<GpioUi>>,
    gpio_engine: Arc<Mutex<GpioEngine>>,

    db: Arc<Mutex<DbConn>>,
    broadcast_receiver: crossbeam::channel::Receiver<String>,
}

trait MenuPage {
    fn watch_loop(&mut self, text: &str, option: Vec<(u8, u8)>) -> MenuPages {
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
       
        //let _ = lcd_lock.exec(LCDCommand{
        //    cmd: LCDProgramm::Move,
        //    args: Some({
        //        let mut map = HashMap::new();
        //        map.insert("y".to_string(), LCDArg::Int(1));
        //        map.insert("x".to_string(), LCDArg::Int(option[self.get_current_selection()].0 as i128));
        //        map
        //    })
        //});
        //let _ = lcd_lock.exec(LCDCommand{
        //    cmd: LCDProgramm::Write,
        //    args: Some({
        //        let mut map = HashMap::new();
        //        map.insert("text".to_string(), LCDArg::String("_".repeat(
        //            (option[self.get_current_selection()].1 - option[self.get_current_selection()].0) as usize)
        //            ));
        //        map
        //    })
        //});
        let mut last_selection: i16 = -2;
        loop {
            let actions: [(_, Box<dyn Fn(&mut Self, u8) -> Option<MenuPages>>); 4] = [
                (gpio_lock.home.read(), Box::new(|s, o| MenuPage::home_handler(s, o))),
                (gpio_lock.left.read(), Box::new(|s, o| MenuPage::left_handler(s, o))),
                (gpio_lock.right.read(), Box::new(|s, o| MenuPage::right_handler(s, o))),
                (gpio_lock.enter.read(), Box::new(|s, o| MenuPage::enter_handler(s, o))),
            ];
            

            for (i, (level, handler)) in actions.iter().enumerate() {
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
    fn execute_update(&mut self) -> ();

    fn home_handler(&mut self, options_len: u8) -> Option<MenuPages>;
    fn left_handler(&mut self, options_len: u8) -> Option<MenuPages>;
    fn right_handler(&mut self, options_len: u8) -> Option<MenuPages>;
    fn enter_handler(&mut self, options_len: u8) -> Option<MenuPages>;
}

#[derive(Debug, Clone, Copy)]
enum MenuPages {
    MainMenu,
    SettingsMenu,
    ManualControll,
}


struct MainMenu {
    global_io: GlobalIoHandlers,
    current_selection: usize,
}

impl MenuPage for MainMenu {
    
    fn get_lcd(&mut self) -> Arc<Mutex<LCDdriver>> {
        self.global_io.lcd.clone()
    }

    fn get_gpio_controller(&mut self) -> Arc<Mutex<GpioUi>> {
        self.global_io.gpio_ui.clone()
    }

    fn execute_update(&mut self) -> () {
        
    }

    fn get_current_selection(&self) -> usize {
        self.current_selection
    }

    fn enter_handler(&mut self, _: u8) -> Option<MenuPages> {
        match self.current_selection {
            0 => Some(MenuPages::ManualControll),
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
    global_io: GlobalIoHandlers,
    current_selection: usize,
}

impl MenuPage for SettingsMenu {
    
    fn get_lcd(&mut self) -> Arc<Mutex<LCDdriver>> {
        self.global_io.lcd.clone()
    }

    fn get_gpio_controller(&mut self) -> Arc<Mutex<GpioUi>> {
        self.global_io.gpio_ui.clone()
    }

    fn execute_update(&mut self) -> () {
        
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


struct ManualControllPage {
    global_io: GlobalIoHandlers,
    current_selection: usize,

    position: u8,
}
impl MenuPage for ManualControllPage {
    
    fn get_lcd(&mut self) -> Arc<Mutex<LCDdriver>> {
        self.global_io.lcd.clone()
    }

    fn get_gpio_controller(&mut self) -> Arc<Mutex<GpioUi>> {
        self.global_io.gpio_ui.clone()
    }

    fn get_current_selection(&self) -> usize {
        self.current_selection
    }

    fn execute_update(&mut self) -> () {
        let mut db_lock = self.global_io.db.lock().unwrap();
        db_lock.update_engine_state(self.position.into()).unwrap()
    }

    fn enter_handler(&mut self, _: u8) -> Option<MenuPages> {
        match self.current_selection {
            0 => {
                walk_engine(&mut self.global_io.gpio_engine, true, false);
            },
            2 => {
                walk_engine(&mut self.global_io.gpio_engine, false, false);
            },
            _ => (
                // To implement save
            ),
            
        }
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
        let (tx, rx) = unbounded::<String>();   

        let goip_ui = GpioUi {
            home: Gpio::new().unwrap().get(23).unwrap().into_input_pullup(),
            left: Gpio::new().unwrap().get(25).unwrap().into_input_pullup(),
            right: Gpio::new().unwrap().get(22).unwrap().into_input_pullup(),
            enter: Gpio::new().unwrap().get(24).unwrap().into_input_pullup(),
        };
        let gpio_engine = GpioEngine {
            dir: Gpio::new().unwrap().get(20).unwrap().into_output(),
            step: Gpio::new().unwrap().get(21).unwrap().into_output(),
            ena: Gpio::new().unwrap().get(16).unwrap().into_output(),
        };

        let mut menu_page_thread: Option<JoinHandle<MenuPages>> = None;
        let mut requested_menu = MenuPages::MainMenu;
        
        let global_io = GlobalIoHandlers {  
            lcd: Arc::new(Mutex::new(LCDdriver::new(Path::new("lcd_driver/lcd.sock"), true).unwrap())),
            gpio_ui: Arc::new(Mutex::new(goip_ui)),
            gpio_engine: Arc::new(Mutex::new(gpio_engine)),

            db: Arc::new(Mutex::new(DbConn::establish_connection())),
            broadcast_receiver: rx,
        };
        println!("Entering main loop");
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
            println!("Spawning new {:?}", &requested_menu);
            let _global_io = global_io.clone();
            menu_page_thread = Some(match requested_menu {
                MenuPages::MainMenu => 
                    thread::spawn(move || {
                        MainMenu {
                        global_io: _global_io,
                        current_selection: 0,
                    }.watch_loop("<Ctrl.     Set.>", vec![(0, 6), (11, 16)])}),         
                MenuPages::SettingsMenu => 
                    thread::spawn(move || {
                        SettingsMenu {
                            global_io: _global_io,
                            current_selection: 0,
                        }.watch_loop("<  Automatic ON?", vec![(0, 1), (13, 16)])
                    }),
                MenuPages::ManualControll => 
                    thread::spawn(move || {
                        let engine_state = _global_io.db.lock().unwrap().get_engine_state().unwrap();
                        ManualControllPage {
                            global_io: _global_io.clone(),
                            current_selection: 0,
                            position: engine_state.position as u8,
                        }.watch_loop("<UP  SAVE  DOWN>", vec![(0, 3), (5, 9), (11, 16)])
                    }),
            });
        }
    }

    


fn main() {
    rebuild_test_db();
    main_prosessing_loop();
}
