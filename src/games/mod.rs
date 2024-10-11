use crate::Game;
use macroquad::Error;

pub mod apples;
pub mod mouse;
pub mod running;

pub async fn all() -> Result<Vec<(String, Box<dyn Game>)>, Error> {
    Ok(vec![
        (
            "apples".to_owned(),
            Box::new(self::apples::Game::new().await?),
        ),
        (
            "mouse".to_owned(),
            Box::new(self::mouse::Game::new().await?),
        ),
        (
            "running".to_owned(),
            Box::new(self::running::Game::new().await?),
        ),
    ])
}
