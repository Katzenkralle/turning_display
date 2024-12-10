
use db::{models::Led, DbConn};
use lcd_driver::{LCDdriver, LCDCommand, LCDProgramm, LCDArg};
use std::{path::Path, str, thread::{self, JoinHandle}};
use crossbeam::channel::unbounded;
use std::collections::HashMap;
use rppal::gpio::{Gpio, InputPin, Level, OutputPin};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;


mod ui_pages;
use ui_pages::{man_ctrl::ManualControllPage, menu::MainMenu, settings::SettingsMenu, led_ctrl::LedCtrlPage, UiPages, MenuPage, ReactivePage};

const USER_INPUT_DELAY: u64 = 200;
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

fn walk_engine(global_ui: &mut GlobalIoHandlers, go_right: bool, use_pwm: bool) -> i32 {
    let mut lock = global_ui.gpio_engine.lock().unwrap();
    let mut delta_pos = 1;
    let mut db_lock = global_ui.db.lock().unwrap();
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
    let c_pos = db_lock.get_application_state().unwrap().current_engine_state;
    db_lock.update_application_state(Some(c_pos+delta_pos), None).unwrap();
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
    active_preset: i32,
}

impl GlobalIoHandlers {
    fn new() -> Self {
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
        
       let db = DbConn::establish_connection();
       let active_preset = db.get_application_state().unwrap().active_preset;
        
        GlobalIoHandlers {  
            lcd: Arc::new(Mutex::new(LCDdriver::new(Path::new("lcd_driver/lcd.sock"), true).unwrap())),
            gpio_ui: Arc::new(Mutex::new(goip_ui)),
            gpio_engine: Arc::new(Mutex::new(gpio_engine)),

            db: Arc::new(Mutex::new(db)),
            active_preset: active_preset,
        }
    }
}

fn main_prosessing_loop() -> () {
        //let (tx, rx) = unbounded::<String>();   

        let mut menu_page_thread: Option<JoinHandle<UiPages>> = None;
        let mut requested_menu = UiPages::Menu1;
        
        let global_io = GlobalIoHandlers::new();
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
                        return_to: vec![UiPages::Menu2, UiPages::ManualControll, UiPages::LedColor],
                    }.watch_loop("< mPos.   Led. >", vec![(0,1), (2, 7), (10, 14)])}),  
                UiPages::Menu2 =>
                    thread::spawn(move || {
                        MainMenu {
                            global_io: _global_io,
                            current_selection: 0,
                            return_to: vec![UiPages::ManualControll, UiPages::SettingsMenu, UiPages::Menu1],
                        }.watch_loop("  Sav.   Set.  >", vec![(2, 6), (9, 13), (15, 16)])}),       
                UiPages::SettingsMenu => 
                    thread::spawn(move || {
                        SettingsMenu {
                            global_io: _global_io,
                            current_selection: 0,
                        }.watch_loop("<  Automatic ON?", vec![(0, 1), (13, 16)])
                    }),
                UiPages::ManualControll => 
                    thread::spawn(move || {
                        let app_state = _global_io.db.lock().unwrap().get_application_state().unwrap();
                        ManualControllPage {
                            global_io: _global_io.clone(),
                            current_selection: 0,
                            position: app_state.current_engine_state as u8,
                        }.watch_loop("<UP  SAVE  DOWN>", vec![(0, 3), (5, 9), (11, 16)])
                    }),
                UiPages::LedColor =>
                    thread::spawn(move || {
                        let associates = _global_io.active_preset;
                        let led_state: Led = _global_io.db
                            .lock()
                            .expect("DB lock could not be aquired")
                            .get_associated_led(associates)
                            .unwrap_or(vec![])
                            .get(0)
                            .cloned()
                            .or_else(|| Some(Led {
                                id: 0,
                                color: "ff0000".to_string(),
                                brightness: 100,
                                mode: "solid".to_string(),
                                associated_preset: Some(associates),
                            }))
                            .unwrap();
                        LedCtrlPage {
                            global_io: _global_io,
                            current_selection: 0,
                            return_to: vec![(0, UiPages::LedBrightness), (3, UiPages::LedMode)],
                            color: led_state.color.clone(),
                            brightness: led_state.brightness as u8,
                            mode: led_state.mode.clone(),
                            setting: UiPages::LedColor
                        }.reactive_watch("<^   Color    v>", vec![(0, 1), (1, 2), (14, 15), (15, 16)])
                    }),
                UiPages::LedBrightness =>
                    thread::spawn(move || {
                        let associates = _global_io.active_preset;
                        let led_state = _global_io.db
                            .lock()
                            .expect("DB lock could not be aquired")
                            .get_associated_led(associates)
                            .unwrap_or(vec![])
                            .get(0)
                            .cloned()
                            .or_else(|| Some(Led {
                                id: 0,
                                color: "ff0000".to_string(),
                                brightness: 100,
                                mode: "solid".to_string(),
                                associated_preset: Some(associates),
                            }))
                            .unwrap();
                        LedCtrlPage {
                            global_io: _global_io,
                            current_selection: 0,
                            return_to: vec![(0, UiPages::LedMode), (3, UiPages::LedColor)],
                            color: led_state.color.clone(),
                            brightness: led_state.brightness as u8,
                            mode: led_state.mode.clone(),
                            setting: UiPages::LedBrightness,
                        }.reactive_watch("<^ Brightness v>", vec![(0, 1), (1, 2), (14, 15), (15, 16)])
                    }),
                UiPages::LedMode =>
                    thread::spawn(move || {
                        let associates = _global_io.active_preset;
                        let led_state = _global_io.db
                            .lock()
                            .expect("DB lock could not be aquired")
                            .get_associated_led(associates)
                            .unwrap_or(vec![])
                            .get(0)
                            .cloned()
                            .or_else(|| Some(Led {
                                id: 0,
                                color: "ff0000".to_string(),
                                brightness: 100,
                                mode: "solid".to_string(),
                                associated_preset: Some(associates),
                            }))
                            .unwrap();
                        LedCtrlPage {
                            global_io: _global_io,
                            current_selection: 0,
                            return_to: vec![(0, UiPages::LedColor), (3, UiPages::LedBrightness)],
                            color: led_state.color.clone(),
                            brightness: led_state.brightness as u8,
                            mode: led_state.mode.clone(),
                            setting: UiPages::LedMode,
                        }.reactive_watch("<^    Mode    v>", vec![(0, 1), (1, 2), (14, 15), (15, 16)])
                    }),
            });
        }
    }

    


fn main() {
    main_prosessing_loop();
}
