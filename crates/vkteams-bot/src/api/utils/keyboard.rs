//! Module with traits for [`Keyboard`], [`MessageTextParser`], etc.
use crate::api::types::*;
use std::convert::From;

impl From<Keyboard> for String {
    /// # Convert [`Keyboard`] to JSON string
    fn from(val: Keyboard) -> Self {
        val.get_keyboard()
    }
}
impl Keyboard {
    /// # Create new [`Keyboard`]
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    /// # Append row with buttons to [`Keyboard`]
    pub fn add_row(&mut self) -> Self {
        self.buttons.push(vec![]);
        self.to_owned()
    }
    /// # Get index of last row of [`Keyboard`]
    pub fn get_row_index(&self) -> usize {
        self.buttons.len() - 1
    }
    /// # Append button to last row of [`Keyboard`]
    /// Maximum buttons in row is 8. If row is full, add new row
    pub fn add_button(&mut self, button: &ButtonKeyboard) -> Self {
        // IF row is full, add new row
        if self.buttons[self.get_row_index()].len() >= 8 {
            self.add_row();
        }
        let row_index = self.get_row_index();
        self.buttons[row_index].push(button.clone());
        self.to_owned()
    }
    /// # Get keyboard as JSON string
    fn get_keyboard(&self) -> String {
        serde_json::to_string(&self.buttons).unwrap()
    }
}
impl ButtonKeyboard {
    /// Create new [`ButtonKeyboard`] with URL
    /// ## Parameters
    /// - `text`: [`String`] - Button text
    /// - `url`: [`String`] - URL
    /// - `style`: [`ButtonStyle`] - Button style
    pub fn url(text: String, url: String, style: ButtonStyle) -> Self {
        ButtonKeyboard {
            text,
            style: Some(style),
            url: Some(url),
            callback_data: None,
        }
    }
    /// Create new [`ButtonKeyboard`] with callback data
    /// ## Parameters
    /// - `text`: [`String`] - Button text
    /// - `cb`: [`String`] - Callback data
    /// - `style`: [`ButtonStyle`] - Button style
    pub fn cb(text: String, cb: String, style: ButtonStyle) -> Self {
        ButtonKeyboard {
            text,
            style: Some(style),
            url: None,
            callback_data: Some(cb),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::ButtonStyle;

    #[test]
    fn test_keyboard_new_and_add_row() {
        let mut kb = Keyboard::new();
        assert_eq!(kb.buttons.len(), 1);
        kb = kb.add_row();
        assert_eq!(kb.buttons.len(), 2);
        kb = kb.add_row();
        assert_eq!(kb.buttons.len(), 3);
    }

    #[test]
    fn test_keyboard_add_button_and_row_limits() {
        let mut kb = Keyboard::new();
        let btn = ButtonKeyboard::url(
            "A".to_string(),
            "http://a".to_string(),
            ButtonStyle::Primary,
        );
        // Добавляем 8 кнопок в одну строку
        for _ in 0..8 {
            kb = kb.add_button(&btn);
        }
        assert_eq!(kb.buttons.len(), 1);
        assert_eq!(kb.buttons[0].len(), 8);
        // 9-я кнопка должна создать новую строку
        kb = kb.add_button(&btn);
        assert_eq!(kb.buttons.len(), 2);
        assert_eq!(kb.buttons[1].len(), 1);
    }

    #[test]
    fn test_keyboard_get_row_index() {
        let mut kb = Keyboard::new();
        assert_eq!(kb.get_row_index(), 0);
        kb = kb.add_row();
        assert_eq!(kb.get_row_index(), 1);
    }

    #[test]
    fn test_keyboard_to_string_json() {
        let mut kb = Keyboard::new();
        let btn = ButtonKeyboard::cb("B".to_string(), "cb".to_string(), ButtonStyle::Attention);
        kb = kb.add_button(&btn);
        let json: String = kb.clone().into();
        // Должен быть валидный JSON массив массивов
        let parsed: Vec<Vec<ButtonKeyboard>> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), kb.buttons.len());
        assert_eq!(parsed[0][0].text, "B");
    }

    #[test]
    fn test_buttonkeyboard_url_and_cb() {
        let btn_url = ButtonKeyboard::url(
            "Link".to_string(),
            "http://link".to_string(),
            ButtonStyle::Base,
        );
        assert_eq!(btn_url.text, "Link");
        assert_eq!(btn_url.url.as_deref(), Some("http://link"));
        assert!(btn_url.callback_data.is_none());
        let btn_cb =
            ButtonKeyboard::cb("CB".to_string(), "data".to_string(), ButtonStyle::Attention);
        assert_eq!(btn_cb.text, "CB");
        assert_eq!(btn_cb.callback_data.as_deref(), Some("data"));
        assert!(btn_cb.url.is_none());
    }

    #[test]
    fn test_keyboard_add_many_buttons_multiple_rows() {
        let mut kb = Keyboard::new();
        let btn = ButtonKeyboard::url(
            "A".to_string(),
            "http://a".to_string(),
            ButtonStyle::Primary,
        );
        for _ in 0..17 {
            kb = kb.add_button(&btn);
        }
        // Должно быть 3 строки: 8 + 8 + 1 (с учётом начальной строки)
        assert_eq!(kb.buttons.len(), 3);
        assert_eq!(kb.buttons[0].len(), 8);
        assert_eq!(kb.buttons[1].len(), 8);
        assert_eq!(kb.buttons[2].len(), 1);
    }
}
