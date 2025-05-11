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
