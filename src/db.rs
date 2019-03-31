use super::schema::songs;
use crate::song::{GetRandomSong, NewSong, Song};
use actix::{Actor, Handler, Message, SyncContext};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, Pool, PooledConnection};
use diesel::result::Error as DieselError;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use std::env;

pub type Conn = SqliteConnection;
pub type SqlPool = Pool<ConnectionManager<Conn>>;
pub type PooledConn = PooledConnection<ConnectionManager<Conn>>;

pub fn new_pool<S: Into<String>>(database_url: S) -> Result<SqlPool, ()> {
    let manager = ConnectionManager::<Conn>::new(database_url.into());
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to connect to the db");
    Ok(pool)
}

pub struct DBExecutor {
    conn: SqlPool,
}

impl Actor for DBExecutor {
    type Context = SyncContext<Self>;
}

impl DBExecutor {
    pub fn new() -> Self {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let conn = new_pool(database_url).expect("Failed to create pool");
        DBExecutor { conn }
    }

    fn get_conn(&mut self) -> PooledConn {
        self.conn.get().unwrap()
    }
}

impl Message for GetRandomSong {
    type Result = Result<Song, DieselError>;
}

impl Handler<GetRandomSong> for DBExecutor {
    type Result = Result<Song, DieselError>;

    fn handle(&mut self, msg: GetRandomSong, ctx: &mut Self::Context) -> Self::Result {
        Ok(get_random_song(&self.get_conn()))
    }
}

// atm it's not needed
pub struct CheckSongExistence {
    pub song_name: String,
}

impl Message for CheckSongExistence {
    type Result = Result<Song, DieselError>;
}

impl Handler<CheckSongExistence> for DBExecutor {
    type Result = Result<Song, DieselError>;
    fn handle(&mut self, msg: CheckSongExistence, ctx: &mut Self::Context) -> Self::Result {
        get_song(&self.get_conn(), msg.song_name)
    }
}

pub struct SaveSong {
    pub song: NewSong,
}

impl Message for SaveSong {
    type Result = Result<Song, DieselError>;
}

impl Handler<SaveSong> for DBExecutor {
    type Result = Result<Song, DieselError>;

    fn handle(&mut self, msg: SaveSong, ctx: &mut Self::Context) -> Self::Result {
        save_song(&self.get_conn(), &msg.song)
    }
}

// in case of problems during save return random song to user via Err()

fn get_random_song(conn: &PooledConn) -> Song {
    no_arg_sql_function!(RANDOM, (), "Represents the sql RANDOM() function");
    songs::table
        .order(RANDOM)
        .limit(1)
        .first::<Song>(conn)
        .expect("unable to load songs")
}

fn save_song(conn: &PooledConn, song: &NewSong) -> Result<Song, DieselError> {
    println!("saving song - {:#?}", song);
    let x = diesel::insert_into(songs::table).values(song).execute(conn);
    println!("Song saved - {:#?}", song);
    get_song(conn, song.name.clone())
}

fn get_song(conn: &PooledConn, song_name: String) -> Result<Song, DieselError> {
    use super::schema::songs::dsl::name;
    songs::table.filter(name.eq(song_name)).first::<Song>(conn)
}
