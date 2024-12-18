use crate::Game;
use anyhow::Error;

pub mod apples;
pub mod balls;
pub mod drive;
pub mod letters;
pub mod mouse;
pub mod running;

pub async fn all() -> Result<Vec<(String, Box<dyn Game>)>, Error> {
    Ok(vec![
        (
            "apples".to_owned(),
            Box::new(self::apples::Game::new().await?),
        ),
        (
            "letters".to_owned(),
            Box::new(self::letters::Game::new().await?),
        ),
        (
            "mouse".to_owned(),
            Box::new(self::mouse::Game::new().await?),
        ),
        (
            "balls".to_owned(),
            Box::new(self::balls::Game::new().await?),
        ),
        (
            "running".to_owned(),
            Box::new(self::running::Game::new().await?),
        ),
        (
            "drive".to_owned(),
            Box::new(self::drive::Game::new().await?),
        ),
    ])
}
