use diesel::prelude::*;
use dotenvy::dotenv;
use schema::EngineState::{direction, position};
use std::{any, env};

pub mod models;
pub mod schema;

use self::models::{NewLed, Led};

use std::fmt;
use any::Any;
use std::sync::{Arc, Mutex};


pub struct DbConn(pub Arc<Mutex<SqliteConnection>>);

impl DbConn {
    pub fn establish_connection() -> Self {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let connection = SqliteConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));
        let conn = Self(Arc::new(Mutex::new(connection)));
        conn.init_engine_state().expect("Failed to initialize engine state");
        conn
    }

    fn init_engine_state(&self) -> Result<(), diesel::result::Error> {
        use self::schema::EngineState::dsl::*;
        let lock = &mut *self.0.lock()
            .map_err(|_| diesel::result::Error::RollbackTransaction)?;

        // we neet to declare models::EngineState here because we have a name conflict with the Table EngineState
        if  EngineState.filter(id.eq("main")).load::<models::EngineState>(lock)?.len() > 0 {
            return Ok(());
        }
        diesel::insert_into(EngineState)
            .values(models::EngineState{
                id: "main".to_string(),
                position: 0,
                speed: 0, 
                direction: "stop".to_string()
            })
            .execute(lock)?;
        Ok(())
    }


    pub fn add_led(&mut self, x: i32, y: i32, _color: String) 
        -> Result<(), diesel::result::Error> {
        use self::schema::Led::dsl::*;

        let new_led = models::NewLed{ px: x, py: y, color: _color};
        let lock = &mut * self.0.lock()
            .map_err(|_| diesel::result::Error::RollbackTransaction)?;
        diesel::insert_into(Led)
            .values(new_led)
            .execute(lock)?;
        Ok(())
    }

    pub fn del_led(&mut self, index: i32) -> Result<(), diesel::result::Error> {
        use self::schema::Led::dsl::*;
        let lock = &mut *self.0.lock()
            .map_err(|_| diesel::result::Error::RollbackTransaction)?;
        diesel::delete(Led.filter(id.eq(index)))
            .execute(lock)?;
        Ok(())
    }

    pub fn get_leds(&self, index: Option<i32>) -> Result<Vec<models::Led>, diesel::result::Error> {
        use self::schema::Led::dsl::*;
        let lock = &mut *self.0.lock()
            .map_err(|_| diesel::result::Error::RollbackTransaction)?;
        let results = match index {
            Some(index) => Led
                .filter(id.eq(index))
                .load(lock)?,
            None => Led
                .load(lock)?,
        };
        Ok(results)
    }


    pub fn update_engine_state(&mut self, _position: i32, _direction: String) -> Result<(), diesel::result::Error> {
        use self::schema::EngineState::dsl::*;
        let lock = &mut *self.0.lock()
            .map_err(|_| diesel::result::Error::RollbackTransaction)?;
        diesel::update(EngineState.filter(id.eq("main")))
            .set((position.eq(position), direction.eq(direction)))
            .execute(lock)?;
        Ok(())
    }

    pub fn update_engine_prefered_speed(&mut self, _speed: i32) -> Result<(), diesel::result::Error> {
        use self::schema::EngineState::dsl::*;
        let lock = &mut *self.0.lock()
            .map_err(|_| diesel::result::Error::RollbackTransaction)?;
        diesel::update(EngineState.filter(id.eq("main")))
            .set(speed.eq(speed))
            .execute(lock)?;
        Ok(())
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
