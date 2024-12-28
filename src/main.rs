
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
use ui_pages::{man_ctrl::ManualControllPage, menu::MainMenu, select_target::MoveToTarget, led_ctrl::LedCtrlPage, calibrate::CalibrationPage , UiPages, MenuPage, ReactivePage};
use rand::Rng;
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

fn light_strip(strip: &mut Arc<Mutex<Strip>>,  mode: &str, color: Option<[u8; 3]>, _brightness: Option<u8>) -> () {
    let mut lock = strip.lock().unwrap();
    match mode {
        "solid" => {
            if let Some(_color) = color  {
                let mut base_color = _color;
                if let Some(_brightness) = _brightness {
                    for i in 0..3 {
                        base_color[i] = (base_color[i] as f32*(_brightness as f32 / 100.0)).round() as u8;
                    }
                }
                lock.fill(Led::from_rgb_array(base_color));
            }
        },
        _ => (),
    }

    let _ = lock.update();
}


fn walk_engine(gpio_engine: &mut Arc<Mutex<GpioEngine>>, go_right: bool, delta_distance: Option<u64>) -> (i32, bool) {
    let mut lock = gpio_engine.lock().unwrap();
    const STEPS_PER_MOVE: i32 = 200;
    let delay_micros = lock.delay_micros;
    let mut delta_pos: i32;
    let mut hit_calibration = false;

    if let Some(delta) = delta_distance {
        delta_pos = delta as i32;
    } else {
        delta_pos = STEPS_PER_MOVE as i32;   
    }
    if !go_right {
        lock.dir.write(Level::Low);
        delta_pos *= -1;
    } else {
        lock.dir.write(Level::High);
    }
    let mut run_engine = |i: i32| -> () {
            if lock.calibrate.read() == Level::Low {
                delta_pos = delta_pos - i;
                hit_calibration = true;
            }
            lock.step.write(Level::High);
            thread::sleep(std::time::Duration::from_micros(delay_micros));
            lock.step.write(Level::Low);
            thread::sleep(std::time::Duration::from_micros(delay_micros));
    };
    if let Some(delta) = delta_distance {
        for i in 0..delta {
            run_engine(i as i32);
        }
    } else {
    for i in 0..STEPS_PER_MOVE {
        run_engine(i as i32);
        }
    }
    (delta_pos, hit_calibration)
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
    sleep: OutputPin,
    calibrate: InputPin,

    stepps_per_round: u64,
    delay_micros: u64,
}

impl GpioEngine {
    fn update_steps_per_round(&mut self, steps_per_round: u64) -> () {
        self.stepps_per_round = steps_per_round;
    }
}

#[derive( Clone)]
struct GlobalIoHandlers {
    lcd: Arc<Mutex<LCDdriver>>,
    rgb_strip: Arc<Mutex<Strip>>,
    gpio_ui: Arc<Mutex<GpioUi>>,
    gpio_engine: Arc<Mutex<GpioEngine>>,

    db: Arc<Mutex<DbConn>>,
    active_preset: Arc<Mutex<i32>>,
    automatic_mode_delay: Arc<Mutex<i32>>,
    automatic_enabled: Arc<Mutex<bool>>,

    terminate: Arc<Mutex<Option<UiPages>>>,
}

impl GlobalIoHandlers {
    fn new() -> Self {
        let strip = Strip::new(Bus::Spi0, 69).unwrap();
        let db = DbConn::establish_connection();
        let active_preset = db.get_application_state().unwrap().active_preset;

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
            calibrate: Gpio::new().unwrap().get(19).unwrap().into_input_pullup(),
            
            stepps_per_round: db.get_application_state().unwrap().engine_steps_per_rotation as u64,
            delay_micros: db.get_application_state().unwrap().delay_micros as u64,
        };
        
        gpio_engine.sleep.write(Level::Low);

        
        GlobalIoHandlers {  
            lcd: Arc::new(Mutex::new(LCDdriver::new(Path::new("lcd_driver/lcd.sock"), true).unwrap())),
            gpio_ui: Arc::new(Mutex::new(goip_ui)),
            gpio_engine: Arc::new(Mutex::new(gpio_engine)),
            rgb_strip: Arc::new(Mutex::new(strip)),

            automatic_enabled: Arc::new(Mutex::new(db.get_application_state().unwrap().automatic_mode)),
            automatic_mode_delay: Arc::new(Mutex::new(db.get_application_state().unwrap().automatic_mode_delay)),
            db: Arc::new(Mutex::new(db)),
            active_preset: Arc::new(Mutex::new(active_preset)),

            terminate: Arc::new(Mutex::new(None)),
        }
    }
}

fn main_prosessing_loop() -> () {
        //let (tx, rx) = unbounded::<String>();   

        let get_led_state = |_global_io: &GlobalIoHandlers| -> LedDb {
            let associates = *_global_io.active_preset.lock().unwrap();
            _global_io.db
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
                .unwrap()
        };

        let mut menu_page_thread: Option<JoinHandle<UiPages>> = None;
        let mut requested_menu = UiPages::Menu1;
        

        let global_io = GlobalIoHandlers::new();
        println!("Entering main loop");
        let mut last_move = std::time::Instant::now();
        let mut move_to_target = 0; 
        loop {
            // Handle completed threads
            if let Some(thread) = menu_page_thread.take() {
                if !thread.is_finished() {
                    menu_page_thread = Some(thread);
                    continue;
                }
                requested_menu = thread.join().unwrap();
                *global_io.terminate.lock().unwrap() = None;
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
                    }.watch_loop("< mPos.   Led.  ", vec![(0,1), (2, 7), (10, 14)])}),  
                UiPages::Menu2 =>
                    thread::spawn(move || {
                        MainMenu {
                            global_io: _global_io,
                            current_selection: 2,
                            return_to: vec![UiPages::CalibrationPage, UiPages::MoveToTarget, UiPages::Menu1],
                        }.watch_loop("Calib. Preset. >", vec![(0, 6), (7, 14), (15, 16)])}),       
                UiPages::Menu3 => 
                    thread::spawn(move || {
                        MainMenu {
                            global_io: _global_io,
                            return_to: vec![UiPages::Menu1, UiPages::CalibrationPage],
                            current_selection: 0,
                        }.watch_loop("<    Calibration", vec![(0, 1), (6, 16)])
                    }),
                UiPages::ManualControll => 
                    thread::spawn(move || {
                        let app_state = _global_io.db.lock().unwrap().get_application_state().unwrap();
                        ManualControllPage {
                            global_io: _global_io.clone(),
                            current_selection: 0,
                            position: app_state.current_engine_pos as u8,
                        }.watch_loop("<UP  SAVE  DOWN>", vec![(0, 3), (5, 9), (11, 16)])
                    }),
                UiPages::LedColor =>
                    thread::spawn(move || {
                        let led_state = get_led_state(&_global_io);
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
                        let led_state = get_led_state(&_global_io);
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
                        let led_state = get_led_state(&_global_io);
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
                UiPages::CalibrationPage =>
                    thread::spawn(move || {
                        CalibrationPage {
                            global_io: _global_io,
                            current_selection: 0,
                        }.reactive_watch("Calibrating STOP", vec![(12, 16)])
                    }),
                UiPages::MoveToTarget =>{
                    let _move_target = move_to_target.clone();
                    move_to_target = 0;
                    thread::spawn(move || {
                        MoveToTarget {
                            global_io: _global_io,
                            current_selection: 0,
                            target: _move_target,
                            enter_pressed: false
                        }.reactive_watch("1 2 3 4 5 6 7 8 ", vec![ (0, 1), (2, 3), (4, 5), (6, 7), (8, 9), (10, 11), (12, 13), (14, 15)])
                    })
                },
            });
            if *global_io.automatic_enabled.lock().unwrap()  {
                if (last_move.elapsed().as_secs() / 60) as i32 >  global_io.automatic_mode_delay.try_lock().and_then(|a| Ok(*a)).unwrap_or(core::i32::MAX) {
                    // signal termination and nxt menu 
                    *global_io.terminate.lock().unwrap() = Some(UiPages::MoveToTarget);
                    last_move = std::time::Instant::now();
                    let existing_presets = global_io.db.lock().unwrap().get_all_presets().unwrap();
                    let mut rng = rand::thread_rng();
                    loop {
                        let new_target = existing_presets[rng.gen_range(0..existing_presets.len())];
                        if new_target != *global_io.active_preset.lock().unwrap() {
                            move_to_target = new_target;
                            break;
                        }
                    }
                    }
                }

        }
    }

    


fn main() {
    main_prosessing_loop();
}
