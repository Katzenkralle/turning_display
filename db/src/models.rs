use diesel::prelude::*;

#[derive(Debug)]
#[derive(Queryable, Selectable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::Led)]
pub struct Led {
    pub id: i32,
    pub color: String,
    pub brightness: i32,
    pub mode: String,
    pub associated_preset: Option<i32>,
}

#[derive(Debug)]
#[derive(Insertable)]
#[diesel(table_name = crate::schema::Led)]
pub struct NewLed {
    pub color: String,
    pub brightness: i32,
    pub mode: String,
    pub associated_preset: Option<i32>,
}


#[derive(Debug)]
#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::Engine)]
pub struct Engine {
    pub id: i32,
    pub position: i32,
    pub is_target: bool,
    pub associated_preset: Option<i32>,
}

#[derive(Debug)]
#[derive(Insertable)]
#[diesel(table_name = crate::schema::Engine)]
pub struct NewEngine {
    pub position: i32,
    pub is_target: bool,
    pub associated_preset: Option<i32>,
}

#[derive(Debug)]
#[derive(Insertable, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::ApplicationState)]
pub struct ApplicationState {
    pub id: i32,
    pub active_preset: i32,
    pub current_engine_pos: i32,
    pub engine_steps_per_rotation: i32,
    pub delay_micros: i32,
    pub automatic_mode: bool,
    pub automatic_mode_delay: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::ApplicationState)]
pub struct NewApplicationState {
    pub id: i32,
}