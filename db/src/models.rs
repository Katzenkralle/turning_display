use diesel::prelude::*;

#[derive(Debug)]
#[derive(Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::Led)]
pub struct Led {
    pub id: i32,
    pub px: i32,
    pub py: i32,
    pub color: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::Led)]
pub struct NewLed {
    pub px: i32,
    pub py: i32,
    pub color: String,
}

#[derive(Debug)]
#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::EngineState)]
pub struct EngineState {
    pub id: String,
    pub position: i32,
    pub steps_per_revolution: i32,
}