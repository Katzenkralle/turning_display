use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

pub mod models;
pub mod schema;


use std::sync::{Arc, Mutex};


pub struct DbConn(pub Arc<Mutex<SqliteConnection>>);

impl DbConn {
    pub fn establish_connection() -> Self {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let connection = SqliteConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));
        let conn = Self(Arc::new(Mutex::new(connection)));
        conn.init_engine_state().expect("Failed to initialize engine state in db");
        conn.init_led_state().expect("Failed to initialize led state in db");
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
                steps_per_revolution: 200,
            })
            .execute(lock)?;
        Ok(())
    }

    fn init_led_state(&self) -> Result<(), diesel::result::Error> {
        use self::schema::Led::dsl::*;
        let lock = &mut *self.0.lock()
            .map_err(|_| diesel::result::Error::RollbackTransaction)?;

        // we neet to declare models::Led here because we have a name conflict with the Table Led
        if  Led.load::<models::Led>(lock)?.len() > 0 {
            return Ok(());
        }
        for i in 0..69 {
            diesel::insert_into(Led)
                .values(models::Led{
                    id: i,
                    color: "ff0000".to_string(),
                    brightness: 10,
                    mode: "solid".to_string(),
                })
                .execute(lock)?;
        }
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

    pub fn update_led(&self, _id: Vec<i32>, _color: Option<&String>, _brightness: Option<u8>, _mode: Option<&String>) -> Result<(), diesel::result::Error> {
        use self::schema::Led::dsl::*;
        // todo: use dynamic query builder
        let lock = &mut *self.0.lock()
            .map_err(|_| diesel::result::Error::RollbackTransaction)?;
        if let Some(_color) = _color {
            diesel::update(Led.filter(id.eq_any(_id.clone())))
                .set(color.eq(_color))
                .execute(lock)?;
        }
        if let Some(_brightness) = _brightness {
            diesel::update(Led.filter(id.eq_any(_id.clone())))
                .set(brightness.eq(_brightness as i32))
                .execute(lock)?;
        }
        if let Some(_mode) = _mode {
            diesel::update(Led.filter(id.eq_any(_id)))
                .set(mode.eq(_mode))
                .execute(lock)?;
        }
        Ok(())
    }


    pub fn update_engine_state(&mut self, _position: i32) -> Result<(), diesel::result::Error> {
        use self::schema::EngineState::dsl::*;
        let lock = &mut *self.0.lock()
            .map_err(|_| diesel::result::Error::RollbackTransaction)?;
        diesel::update(EngineState.filter(id.eq("main")))
            .set(position.eq(_position))
            .execute(lock)?;
        Ok(())
    }

    
    pub fn get_engine_state(&self) -> Result<models::EngineState, diesel::result::Error> {
        use self::schema::EngineState::dsl::*;
        let lock = &mut *self.0.lock()
            .map_err(|_| diesel::result::Error::RollbackTransaction)?;
        let result = EngineState
            .filter(id.eq("main"))
            .first(lock);
        result
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
