use super::schema::songs;
use crate::song::{GetRandomSong, NewSong, Song};
use actix::{Actor, Context, Handler, Message};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, Pool, PooledConnection};
use diesel::result::Error as DieselError;
use diesel::sqlite::SqliteConnection;

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
    type Context = Context<Self>;
}

impl DBExecutor {
    pub fn new(conn: SqlPool) -> Self {
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
        get_random_song(&self.get_conn())
    }
}

// atm it's not needed
pub struct CheckSongExistence {
    pub song_name: String,
    pub artists: String,
}

impl Message for CheckSongExistence {
    type Result = Result<Song, DieselError>;
}

impl Handler<CheckSongExistence> for DBExecutor {
    type Result = Result<Song, DieselError>;
    fn handle(&mut self, msg: CheckSongExistence, ctx: &mut Self::Context) -> Self::Result {
        get_song(&self.get_conn(), msg.song_name, msg.artists)
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
        save_song(&self.get_conn(), &msg.song).map_err(|e| {
            println!("{:#?}", e);
            e
        })
    }
}

pub struct GetAllSongs;

impl Message for GetAllSongs {
    type Result = Result<Vec<Song>, DieselError>;
}

impl Handler<GetAllSongs> for DBExecutor {
    type Result = Result<Vec<Song>, DieselError>;

    fn handle(&mut self, msg: GetAllSongs, ctx: &mut Self::Context) -> Self::Result {
        get_all_songs(&self.get_conn())
    }
}

// in case of problems during save return random song to user via Err()

fn get_random_song(conn: &PooledConn) -> Result<Song, DieselError> {
    use super::schema::songs::dsl::nsfw;

    no_arg_sql_function!(RANDOM, (), "Represents the sql RANDOM() function");
    songs::table
        .filter(nsfw.eq(false))
        .order(RANDOM)
        .limit(1)
        .first::<Song>(conn)
}

fn save_song(conn: &PooledConn, song: &NewSong) -> Result<Song, DieselError> {
    diesel::insert_into(songs::table).values(song).execute(conn);
    get_song(conn, song.name.clone(), song.artists.clone())
}

fn get_song(
    conn: &PooledConn,
    song_name: String,
    song_artists: String,
) -> Result<Song, DieselError> {
    use super::schema::songs::dsl::{artists, name};
    songs::table
        .filter(name.eq(song_name).and(artists.eq(song_artists)))
        .first::<Song>(conn)
}

fn get_all_songs(conn: &PooledConn) -> Result<Vec<Song>, DieselError> {
    songs::table.load::<Song>(conn)
}
