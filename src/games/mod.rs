use crate::Game;
use macroquad::Error;

pub mod apples;
pub mod mouse;

pub async fn all() -> Result<Vec<(String, Box<dyn Game>)>, Error> {
    Ok(vec![
        (
            "Считаем яблоки".to_owned(),
            Box::new(self::apples::ApplesGame::new().await?),
        ),
        (
            "Мышь и сыр".to_owned(),
            Box::new(self::mouse::MouseGame::new().await?),
        ),
    ])
}
