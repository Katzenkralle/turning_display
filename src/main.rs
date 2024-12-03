
use db::DbConn;


fn main() {
    let connection = DbConn::establish_connection();
    connection.add_led(0, 0, "#000000".to_string()).expect("Error adding led");
}
