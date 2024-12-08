
use db::DbConn;
use lcd_driver::{LCDdriver, LCDCommand, LCDProgramm, LCDArg};
use std::{path::Path, str, thread::{self, JoinHandle}};
use crossbeam::channel::unbounded;
use std::collections::HashMap;
use rppal::gpio::{Gpio, InputPin, Level, OutputPin};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;


mod ui_pages;
use ui_pages::{man_ctrl::ManualControllPage, menu::MainMenu, settings::SettingsMenu, UiPages, MenuPage};

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

        let mut menu_page_thread: Option<JoinHandle<UiPages>> = None;
        let mut requested_menu = UiPages::Menu1;
        
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
                UiPages::Menu1 => 
                    thread::spawn(move || {
                        MainMenu {
                        global_io: _global_io,
                        current_selection: 0,
                        return_to: vec![UiPages::Menu2, UiPages::ManualControll, UiPages::SettingsMenu],
                    }.watch_loop("< mPos.   Led. >", vec![(0,1), (2, 8), (9, 14)])}),  
                UiPages::Menu2 =>
                    thread::spawn(move || {
                        MainMenu {
                            global_io: _global_io,
                            current_selection: 0,
                            return_to: vec![UiPages::ManualControll, UiPages::SettingsMenu, UiPages::Menu1],
                        }.watch_loop("  Sav.   Set.  >", vec![(2, 8), (9, 14), (15, 16)])}),       
                UiPages::SettingsMenu => 
                    thread::spawn(move || {
                        SettingsMenu {
                            global_io: _global_io,
                            current_selection: 0,
                        }.watch_loop("<  Automatic ON?", vec![(0, 1), (13, 16)])
                    }),
                UiPages::ManualControll => 
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
