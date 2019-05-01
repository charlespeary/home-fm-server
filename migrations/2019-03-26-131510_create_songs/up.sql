-- Your SQL goes here
CREATE TABLE songs (
                     id INTEGER PRIMARY KEY NOT NULL ,
                     name VARCHAR NOT NULL,
                     path VARCHAR NOT NULL,
                     duration INTEGER NOT NULL,
                     thumbnail_url VARCHAR NOT NULL,
                     artists VARCHAR NOT NULL,
                     nsfw INTEGER NOT NULL DEFAULT 1
)