use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

pub mod models;
pub mod schema;

use self::models::*;

use std::fmt::Result;
use std::sync::{Arc, Mutex};


pub struct DbConn(pub Arc<Mutex<SqliteConnection>>);

impl DbConn {
    pub fn establish_connection() -> Self {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let connection = SqliteConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));
        Self(Arc::new(Mutex::new(connection)))
    }

    pub fn add_led(&mut self, x: i32, y: i32, color: String) 
        -> Result<((), diesel::result::InsertResult<models::Led>)> {
        let new_led = models::NewLed { px: x, py: y, color: color };
        let lock = self.0.lock()
            .map_err(|_| diesel::result::Error::ConnectionError)?;
        diesel::insert_into(schema::led::table)
            .values(&new_led)
            .execute(&mut *lock)?;
        Ok(())
    }
    pub fn get_leds(&self, index: Option<i32>) -> Result<Vec<models::Led>, diesel::result::Error> {
        use self::schema::posts::dsl::*;
        let lock = self.0.lock()
            .map_err(|_| diesel::result::Error::ConnectionError)?;
        let results = match index {
            Some(index) => Led
                .filter(id.eq(index))
                .load(&mut *lock)?,
            None => Led
                .load(&mut *lock)?,
        };
        Ok(results)
    }
}



pub fn add(left: usize, right: usize) -> usize {
    left + right
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
