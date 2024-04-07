use crate::api::types::*;
use std::convert::From;

impl From<Keyboard> for String {
    /// Convert [`Keyboard`] to JSON string
    fn from(val: Keyboard) -> Self {
        val.get_keyboard()
    }
}
/// Create new [`Keyboard`] and check params
impl Keyboard {
    /// Append row with buttons to [`Keyboard`]
    pub fn add_row(&mut self) -> &mut Self {
        self.buttons.push(vec![]);
        self
    }
    /// Get index of last row of [`Keyboard`]
    pub fn get_row_index(&self) -> usize {
        self.buttons.len() - 1
    }
    /// Append button to last row of [`Keyboard`]
    /// Maximum buttons in row is 8. If row is full, add new row
    pub fn add_button(&mut self, button: &ButtonKeyboard) -> &mut Self {
        // IF row is full, add new row
        if self.buttons[self.get_row_index()].len() >= 8 {
            self.add_row();
        }
        let row_index = self.get_row_index();
        self.buttons[row_index].push(button.clone());
        self
    }
    /// Get keyboard as JSON string
    fn get_keyboard(&self) -> String {
        serde_json::to_string(&self.buttons).unwrap()
    }
}
/// Create new [`ButtonKeyboard`] and check params
impl ButtonKeyboard {
    /// Create new [`ButtonKeyboard`] with URL
    pub fn url(text: String, url: String, style: ButtonStyle) -> Self {
        ButtonKeyboard {
            text,
            style: Some(style),
            url: Some(url),
            callback_data: None,
        }
    }
    /// Create new [`ButtonKeyboard`] with callback data
    pub fn cb(text: String, cb: String, style: ButtonStyle) -> Self {
        ButtonKeyboard {
            text,
            style: Some(style),
            url: None,
            callback_data: Some(cb),
        }
    }
}
/// Trait [`MessageTextHTMLParser`]
pub trait MessageTextHTMLParser {
    fn add(&mut self, text: MessageTextFormat) -> &mut Self;
    fn next_line(&mut self) -> &mut Self;
    fn space(&mut self) -> &mut Self;
    fn parse(&self) -> (String, ParseMode);
}
/// Implement [`MessageTextParser`]
impl MessageTextParser {
    /// Parse [`MessageTextFormat`] types to HTML string
    fn parse_html(&self, text: &MessageTextFormat) -> String {
        match text {
            MessageTextFormat::Plain(text) => text.to_string(),
            MessageTextFormat::Link(url, text) => {
                format!("<a href=\"{}\">{}</a>", url, text)
            }
            MessageTextFormat::Bold(text) => {
                format!("<b>{}</b>", text)
            }
            MessageTextFormat::Italic(text) => {
                format!("<i>{}</i>", text)
            }
            MessageTextFormat::Code(text) => {
                format!("<code>{}</code>", text)
            }
            MessageTextFormat::Pre(text, class) => match class {
                //TODO add class enum
                Some(class) => {
                    format!("<pre class=\"{}\">{}</pre>", class, text)
                }
                None => {
                    format!("<pre>{}</pre>", text)
                }
            },
            MessageTextFormat::Mention(text) => {
                format!("<a>@[{}]</a>", text)
            }
            MessageTextFormat::Strikethrough(text) => {
                format!("<s>{}</s>", text)
            }
            MessageTextFormat::Underline(text) => {
                format!("<u>{}</u>", text)
            }
            MessageTextFormat::Quote(text) => {
                format!("<blockquote>{}</blockquote>", text)
            }
            MessageTextFormat::OrderedList(list) => {
                let mut result = String::new();
                for item in list {
                    result.push_str(&format!("<li>{}</li>", item));
                }
                format!("<ol>{}</ol>", result)
            }
            MessageTextFormat::UnOrdereredList(list) => {
                let mut result = String::new();
                for item in list {
                    result.push_str(&format!("<li>{}</li>", item));
                }
                format!("<ul>{}</ul>", result)
            }
            MessageTextFormat::None => String::new(),
        }
    }
}
/// Implement [`MessageTextHTMLParser`] for [`MessageTextParser`]
impl MessageTextHTMLParser for MessageTextParser {
    /// Add plain text to [`MessageTextFormat`]
    fn add(&mut self, text: MessageTextFormat) -> &mut Self {
        self.text.push(text);
        self
    }
    /// Line feed
    fn next_line(&mut self) -> &mut Self {
        self.text.push(MessageTextFormat::Plain(String::from("\n")));
        self
    }
    /// Space
    fn space(&mut self) -> &mut Self {
        self.text.push(MessageTextFormat::Plain(String::from(" ")));
        self
    }
    /// Parse [`MessageTextFormat`] to string
    fn parse(&self) -> (String, ParseMode) {
        let mut result = String::new();
        for txt in &self.text {
            match self.parse_mode {
                ParseMode::HTML => {
                    result.push_str(&self.parse_html(txt));
                }
                _ => todo!("Not implemented parse mode: {:?}", self.parse_mode),
            }
        }
        (result, self.parse_mode)
    }
}
/// Setters
pub trait MessageTextSetters {
    fn set_text(&mut self, parser: Option<MessageTextParser>) -> &mut Self;
    fn set_reply_msg_id(&mut self, msg_id: Option<MsgId>) -> &mut Self;
    fn set_forward_msg_id(&mut self, chat_id: Option<ChatId>, msg_id: Option<MsgId>) -> &mut Self;
    fn set_keyboard(&mut self, keyboard: Option<Keyboard>) -> &mut Self;
}
