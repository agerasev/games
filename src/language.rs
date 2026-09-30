//! Interface and counting language. The choice lasts for the current session.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    English,
    Russian,
}

impl Language {
    pub const ALL: [Self; 2] = [Self::English, Self::Russian];

    pub fn name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Russian => "Русский",
        }
    }

    pub fn text<'a>(self, english: &'a str, russian: &'a str) -> &'a str {
        match self {
            Self::English => english,
            Self::Russian => russian,
        }
    }
}

/// Names shared by a game's controls and canvas.
pub struct Text(&'static str, &'static str);
impl Text {
    pub const fn new(english: &'static str, russian: &'static str) -> Self {
        Self(english, russian)
    }
    pub fn get(&self, language: Language) -> &'static str {
        language.text(self.0, self.1)
    }
}
