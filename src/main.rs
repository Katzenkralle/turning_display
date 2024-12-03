
use db::DbConn;

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

fn main() {
    rebuild_test_db();
    println!("Hello, world!");
}
