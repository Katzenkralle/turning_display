use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Led {
    pub id: i32,
    pub px: i32,
    pub py: i32,
    pub color: String,
}

#[derive(Insertable)]
pub struct NewLed {
    pub px: i32,
    pub py: i32,
    pub color: String,
}