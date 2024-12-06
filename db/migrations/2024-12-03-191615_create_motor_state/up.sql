-- Your SQL goes here
CREATE TABLE EngineState (
    id VARCHAR(255) PRIMARY KEY NOT NULL UNIQUE,
    position INTEGER NOT NULL DEFAULT 0,
    steps_per_revolution INTEGER NOT NULL DEFAULT 100
);