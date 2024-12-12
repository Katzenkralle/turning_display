
use db::DbConn;
use db::models::Led as LedDb;
use lcd_driver::{LCDdriver, LCDCommand, LCDProgramm, LCDArg};
use std::{path::Path, str, thread::{self, JoinHandle}};
use std::collections::HashMap;
use rppal::gpio::{Gpio, InputPin, Level, OutputPin};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use sk6812_rpi::strip::{Bus, Strip};
use sk6812_rpi::led::Led as Led;

mod ui_pages;
use ui_pages::{man_ctrl::ManualControllPage, menu::MainMenu, settings::SettingsMenu, led_ctrl::LedCtrlPage, UiPages, MenuPage, ReactivePage};

const USER_INPUT_DELAY: u64 = 200;
const STEPS_PER_ROUND: i32 = 6000;
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

fn light_strip(strip: &mut Strip,  mode: &str, color: Option<[u8; 3]>, _brightness: Option<u8>) -> () {
    match mode {
        "solid" => {
            if let Some(_color) = color  {
                let mut base_color = _color;
                if let Some(_brightness) = _brightness {
                    for i in 0..3 {
                        base_color[i] = (base_color[i] as f32*(_brightness as f32 / 100.0)).round() as u8;
                    }
                }
                strip.fill(Led::from_rgb_array(base_color));
            }
        },
        _ => (),
    }

    let _ = strip.update();
}


fn walk_engine(gpio_engine: &mut GpioEngine, go_right: bool, delta_distance: Option<u64>) -> i32 {
    const STEPS_PER_WALK: i32 = 500;
    const DELAY: u64 = 200;
    let mut delta_pos = 1 * STEPS_PER_WALK;   
    if !go_right {
        gpio_engine.dir.write(Level::Low);
        delta_pos *= -1;
    } else {
        gpio_engine.dir.write(Level::High);
    }
    
    if let Some(delta) = delta_distance {
        delta_pos = delta as i32;
        for _ in 0..delta {
            gpio_engine.step.write(Level::High);
            thread::sleep(std::time::Duration::from_micros(DELAY));
            gpio_engine.step.write(Level::Low);
            thread::sleep(std::time::Duration::from_micros(DELAY));
        }
    } else {
    for _ in 0..STEPS_PER_WALK {
        gpio_engine.step.write(Level::High);
        thread::sleep(std::time::Duration::from_micros(DELAY));
        gpio_engine.step.write(Level::Low);
        thread::sleep(std::time::Duration::from_micros(DELAY));
        }
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
    sleep: OutputPin
}

struct GlobalIoHandlers {
    lcd: LCDdriver,
    rgb_strip: Strip,
    gpio_ui: GpioUi,
    gpio_engine: GpioEngine,

    db: DbConn,
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
        let mut gpio_engine = GpioEngine {
            dir: Gpio::new().unwrap().get(20).unwrap().into_output(),
            step: Gpio::new().unwrap().get(21).unwrap().into_output(),
            sleep: Gpio::new().unwrap().get(26).unwrap().into_output(),
        };
        
        gpio_engine.sleep.write(Level::Low);
        let strip = Strip::new(Bus::Spi0, 69).unwrap();
       let db = DbConn::establish_connection();
       let active_preset = db.get_application_state().unwrap().active_preset;
        
        GlobalIoHandlers {  
            lcd: LCDdriver::new(Path::new("lcd_driver/lcd.sock"), true).unwrap(),
            gpio_ui: goip_ui,
            gpio_engine: gpio_engine,
            rgb_strip: strip,

            db: db,
            active_preset: active_preset,
        }
    }
}

fn main_prosessing_loop() -> () {
        //let (tx, rx) = unbounded::<String>();   

        let mut menu_page_thread: Option<JoinHandle<(UiPages, GlobalIoHandlers)>>;
        let mut requested_menu;

        let mut global_io = GlobalIoHandlers::new();

        menu_page_thread = Some(thread::spawn(move || {
            MainMenu {
                global_io: global_io,
                current_selection: 0,
                return_to: vec![UiPages::Menu2, UiPages::ManualControll, UiPages::LedColor],
            }.watch_loop("< mPos.   Led. >", vec![(0,1), (2, 7), (10, 14)])
        }));

        println!("Entering main loop");
        loop {
            // Handle completed threads
            if let Some(thread) = menu_page_thread.take() {
                if !thread.is_finished() {
                    menu_page_thread = Some(thread);
                    continue;
                }
                (requested_menu, global_io) = thread.join().unwrap();
        
            
                // Match the requested menu and start a new thread
                println!("Spawning new {:?}", &requested_menu);

                menu_page_thread = Some(match requested_menu {
                    UiPages::Menu1 => 
                        thread::spawn(move || {
                            MainMenu {
                            global_io: global_io,
                            current_selection: 0,
                            return_to: vec![UiPages::Menu2, UiPages::ManualControll, UiPages::LedColor],
                        }.watch_loop("< mPos.   Led. >", vec![(0,1), (2, 7), (10, 14)])}),  
                    UiPages::Menu2 =>
                        thread::spawn(move || {
                            MainMenu {
                                global_io: global_io,
                                current_selection: 0,
                                return_to: vec![UiPages::ManualControll, UiPages::SettingsMenu, UiPages::Menu1],
                            }.watch_loop("  Sav.   Set.  >", vec![(2, 6), (9, 13), (15, 16)])}),       
                    UiPages::SettingsMenu => 
                        thread::spawn(move || {
                            SettingsMenu {
                                global_io: global_io,
                                current_selection: 0,
                            }.watch_loop("<  Automatic ON?", vec![(0, 1), (13, 16)])
                        }),
                    UiPages::ManualControll => 
                        thread::spawn(move || {
                            let app_state = global_io.db.lock().unwrap().get_application_state().unwrap();
                            ManualControllPage {
                                global_io: global_io,
                                current_selection: 0,
                                position: app_state.current_engine_state as u8,
                            }.watch_loop("<UP  SAVE  DOWN>", vec![(0, 3), (5, 9), (11, 16)])
                        }),
                    UiPages::LedColor =>
                        thread::spawn(move || {
                            let associates = global_io.active_preset;
                            let led_state: LedDb = global_io.db
                                .lock()
                                .expect("DB lock could not be aquired")
                                .get_associated_led(associates)
                                .unwrap_or(vec![])
                                .get(0)
                                .cloned()
                                .or_else(|| Some(LedDb {
                                    id: 0,
                                    color: "ff0000".to_string(),
                                    brightness: 100,
                                    mode: "solid".to_string(),
                                    associated_preset: Some(associates),
                                }))
                                .unwrap();
                            LedCtrlPage {
                                global_io: global_io,
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
                            let associates = global_io.active_preset;
                            let led_state = global_io.db
                                .lock()
                                .expect("DB lock could not be aquired")
                                .get_associated_led(associates)
                                .unwrap_or(vec![])
                                .get(0)
                                .cloned()
                                .or_else(|| Some(LedDb {
                                    id: 0,
                                    color: "ff0000".to_string(),
                                    brightness: 100,
                                    mode: "solid".to_string(),
                                    associated_preset: Some(associates),
                                }))
                                .unwrap();
                            LedCtrlPage {
                                global_io: global_io,
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
                            let associates = global_io.active_preset;
                            let led_state = global_io.db
                                .lock()
                                .expect("DB lock could not be aquired")
                                .get_associated_led(associates)
                                .unwrap_or(vec![])
                                .get(0)
                                .cloned()
                                .or_else(|| Some(LedDb {
                                    id: 0,
                                    color: "ff0000".to_string(),
                                    brightness: 100,
                                    mode: "solid".to_string(),
                                    associated_preset: Some(associates),
                                }))
                                .unwrap();
                            LedCtrlPage {
                                global_io: global_io,
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
    }

    


fn main() {
    main_prosessing_loop();
}
