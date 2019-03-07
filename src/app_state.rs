use actix_web::{HttpRequest, HttpResponse, Json, Result};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
	pub current_song: Arc<Mutex<String>>,
	pub something: String,
}

impl Serialize for AppState {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut state = serializer.serialize_struct("AppState", 1)?;
		let current_song = self.current_song.as_ref().lock().unwrap().clone();
		state.serialize_field("current_song", &current_song)?;
		state.end()
	}
}

pub fn get_app_state(req: &HttpRequest<AppState>) -> Result<Json<AppState>> {
	Ok(Json(req.state().clone()))
}
