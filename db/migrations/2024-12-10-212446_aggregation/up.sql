-- Your SQL goes here
CREATE TABLE Engine (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0,
    is_target BOOLEAN NOT NULL DEFAULT FALSE,
    associated_preset INTEGER
);

-- Your SQL goes here
CREATE TABLE Led (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  color TEXT NOT NULL DEFAULT "000000",
  brightness INTEGER NOT NULL DEFAULT 10,
  mode TEXT NOT NULL DEFAULT "solid",
  associated_preset INTEGER
);

CREATE TABLE ApplicationState (
    id INTEGER PRIMARY KEY NOT NULL,
    active_preset INTEGER NOT NULL DEFAULT 0,
    current_engine_pos INTEGER NOT NULL DEFAULT 0,
    engine_steps_per_rotation INTEGER NOT NULL DEFAULT 100,
    delay_micros INTEGER NOT NULL DEFAULT 200,
    automatic_mode BOOLEAN NOT NULL DEFAULT FALSE,
    automatic_mode_delay INTEGER NOT NULL DEFAULT 60
);