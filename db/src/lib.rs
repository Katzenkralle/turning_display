use diesel::prelude::*;
use dotenvy::dotenv;
use schema::ApplicationState::{automatic_mode, engine_steps_per_rotation};
use std::env;

pub mod models;
pub mod schema;


use std::sync::{Arc, Mutex};

const MAX_LED: usize = 69;
pub struct DbConn(pub Arc<Mutex<SqliteConnection>>);

impl DbConn {
    pub fn establish_connection() -> Self {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let mut connection = SqliteConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));

        use self::schema::ApplicationState::dsl::*;
        if ApplicationState.filter(id.eq(0)).load::<models::ApplicationState>(&mut connection).unwrap().len() == 0 {
            diesel::insert_into(ApplicationState)
                .values(models::NewApplicationState{
                    id: 0,
                })
                .execute(&mut connection).unwrap();
        }
        let conn = Self(Arc::new(Mutex::new(connection)));
        conn
    }

    

    pub fn get_associated_led(&self, associates: i32) -> Result<Vec<models::Led>, diesel::result::Error> {
        use self::schema::Led::dsl::*;
        let lock = &mut *self.0.lock()
            .map_err(|_| diesel::result::Error::RollbackTransaction)?;
        Led
        .filter(associated_preset.eq(associates))
        .load(lock)
    }

    pub fn update_led(&self, target_associates: i32, _color: Option<&String>, _brightness: Option<u8>, _mode: Option<&String>) -> Result<(), diesel::result::Error> {
        use self::schema::Led::dsl::*;
        // todo: use dynamic query builder
        let lock = &mut *self.0.lock()
            .map_err(|_| diesel::result::Error::RollbackTransaction)?;

        if Led.filter(associated_preset.eq(target_associates)).load::<models::Led>(lock)?.len() == 0 {
            for _ in 0..MAX_LED {
            diesel::insert_into(Led)
                .values(models::NewLed{
                    color: "ff0000".to_string(),
                    brightness: 10,
                    mode: "solid".to_string(),
                    associated_preset: Some(target_associates),
                })
                .execute(lock)?;
            }
        }

        if let Some(_color) = _color {
            diesel::update(Led.filter(associated_preset.eq(target_associates as i32)))
                .set(color.eq(_color))
                .execute(lock)?;
        }
        if let Some(_brightness) = _brightness {
            diesel::update(Led.filter(associated_preset.eq(target_associates as i32)))
                .set(brightness.eq(_brightness as i32))
                .execute(lock)?;
        }
        if let Some(_mode) = _mode {
            diesel::update(Led.filter(associated_preset.eq(target_associates as i32)))
                .set(mode.eq(_mode))
                .execute(lock)?;
        }
        Ok(())
    }

    pub fn update_engin(&self,  _associated_preset: i32,  _position: Option<i32>, _is_target: Option<bool>) -> Result<(), diesel::result::Error> {
        use self::schema::Engine::dsl::*;
        let lock = &mut *self.0.lock()
            .map_err(|_| diesel::result::Error::RollbackTransaction)?;
        
        if Engine.filter(associated_preset.eq(_associated_preset)).load::<models::Engine>(lock)?.len() == 0 {
            diesel::insert_into(Engine)
                .values(models::NewEngine{
                    position: 0,
                    is_target: false,
                    associated_preset: Some(_associated_preset),
                })
                .execute(lock)?;
        }
        
        if let Some(_position) = _position {
            diesel::update(Engine.filter(is_target.eq(true)))
                .set(position.eq(_position))
                .execute(lock)?;
        }
        if let Some(_is_target) = _is_target {
            diesel::update(Engine.filter(is_target.eq(true)))
                .set(is_target.eq(_is_target))
                .execute(lock)?;
        }
        Ok(())
    }

    pub fn get_engine_preset(&self, _associated_preset: i32) -> Result<models::Engine, diesel::result::Error> {
        use self::schema::Engine::dsl::*;
        let lock = &mut *self.0.lock()
            .map_err(|_| diesel::result::Error::RollbackTransaction)?;
        Engine
            .filter(associated_preset.eq(_associated_preset))
            .first(lock)
    }

    pub fn get_all_presets(&self) -> Result<Vec<i32>, diesel::result::Error> {
        use crate::schema::Engine::dsl as engine_dsl;
        use crate::schema::Led::dsl as led_dsl;
        let lock = &mut *self.0.lock()
        .map_err(|_| diesel::result::Error::RollbackTransaction)?;
        // Query for Engine table
        let engine_presets: Vec<i32> = engine_dsl::Engine
            .select(engine_dsl::associated_preset)
            .filter(engine_dsl::associated_preset.is_not_null())
            .load::<Option<i32>>(lock)?
            .into_iter()
            .filter_map(|p| p) // Remove `None` values
            .collect();
    
        // Query for Led table
        let led_presets: Vec<i32> = led_dsl::Led
            .select(led_dsl::associated_preset)
            .filter(led_dsl::associated_preset.is_not_null())
            .load::<Option<i32>>(lock)?
            .into_iter()
            .filter_map(|p| p) // Remove `None` values
            .collect();
    
        // Combine both vectors and deduplicate
        let mut all_presets = engine_presets;
        all_presets.extend(led_presets);
        all_presets.sort_unstable();
        all_presets.dedup();
    
        Ok(all_presets)
    }

    pub fn update_application_state(&mut self, current_engine_possition: Option<i32>, _active_preset: Option<i32>, _engine_steps_per_rotation: Option<u64>, _automatic_mode: Option<bool>, _automatic_mode_delay: Option<i32> ) -> Result<(), diesel::result::Error> {
        use self::schema::ApplicationState::dsl::*;
        let lock = &mut *self.0.lock()
            .map_err(|_| diesel::result::Error::RollbackTransaction)?;
        if let Some(current_engine_possition) = current_engine_possition {
            diesel::update(ApplicationState.filter(id.eq(0)))
            .set(engine_steps_per_rotation.eq(current_engine_possition))
            .execute(lock)?;
        }
        if let Some(_active_preset) = _active_preset {
            diesel::update(ApplicationState.filter(id.eq(0)))
            .set(active_preset.eq(active_preset))
            .execute(lock)?;
        }
        if let Some(_engine_steps_per_rotation) = _engine_steps_per_rotation {
            diesel::update(ApplicationState.filter(id.eq(0)))
            .set(engine_steps_per_rotation.eq(_engine_steps_per_rotation as i32))
            .execute(lock)?;
        }
        if let Some(_automatic_mode) = _automatic_mode {
            diesel::update(ApplicationState.filter(id.eq(0)))
            .set(automatic_mode.eq(_automatic_mode))
            .execute(lock)?;
        }

        if let Some(_automatic_mode_delay) = _automatic_mode_delay {
            diesel::update(ApplicationState.filter(id.eq(0)))
            .set(automatic_mode_delay.eq(_automatic_mode_delay))
            .execute(lock)?;
        }
        Ok(())
    }

    
    pub fn get_application_state(&self) -> Result<models::ApplicationState, diesel::result::Error> {
        use self::schema::ApplicationState::dsl::*;
        let lock = &mut *self.0.lock()
            .map_err(|_| diesel::result::Error::RollbackTransaction)?;
        ApplicationState
            .filter(id.eq(0))
            .first(lock)
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
