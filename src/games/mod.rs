use crate::Game;
use macroquad::Error;
use std::collections::HashMap;

pub mod apples;
pub mod mouse;

pub async fn all() -> Result<HashMap<String, Box<dyn Game>>, Error> {
    let mut games = HashMap::<String, Box<dyn Game>>::new();
    games.insert(
        "apples".to_owned(),
        Box::new(self::apples::ApplesGame::new().await?),
    );
    games.insert(
        "mouse".to_owned(),
        Box::new(self::mouse::MouseGame::new().await?),
    );
    Ok(games)
}
