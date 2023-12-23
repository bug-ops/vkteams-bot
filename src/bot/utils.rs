use crate::types::*;
use core::ops::Fn;
use std::convert::From;
use std::sync::Arc; // Import the crate containing the `execute` macro
/// Keyboard with simple logic
/// Create new [`Keyboard`] with empty rows
/// ## Example
/// ```rust
/// use vkteams_bot::{Keyboard, ButtonKeyboard, ButtonStyle};
/// // Make keyboard with two buttons
/// let mut kb:Keyboard = Default::default();
/// kb.add_button(&ButtonKeyboard::cb(  
///     String::from("test"),
///     String::from("test_callback_data"),
///     ButtonStyle::Primary,
/// ))
/// .add_button(&ButtonKeyboard::url(
///     String::from("Example"),
///     String::from("https://example.com"),
///     ButtonStyle::Attention,
/// ));
/// ```
impl Bot {
    pub async fn event_listener<F>(&self, func: F)
    where
        F: Fn(ResponseEventsGet),
    {
        loop {
            let events = self.get_events().await;
            match events {
                Ok(res) => {
                    if !res.events.is_empty() {
                        // Get last event id
                        let counter = Arc::clone(&self.event_id);
                        let mut event = counter.lock().unwrap();
                        *event = res.events[res.events.len() - 1].event_id;
                        // Execute callback function
                        func(res);
                    }
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        Self {
            buttons: vec![vec![]],
        }
    }
}

impl Keyboard {
    /// Append row with with buttons to [`Keyboard`]
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
    pub fn get_keyboard(&self) -> String {
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
impl RequestMessagesSendText {
    /// Create new [`RequestMessagesSendTExt`] with required params
    pub fn new(chat_id: ChatId) -> Self {
        Self {
            chat_id,
            text: String::new(),
            reply_msg_id: None,
            forward_chat_id: None,
            forward_msg_id: None,
            inline_keyboard_markup: None,
            format: None,
            parse_mode: None,
        }
    }
}
impl RequestMessagesSendText {
    /// Set reply message id
    pub fn set_reply_msg_id(&mut self, msg_id: MsgId) -> &mut Self {
        self.reply_msg_id = Some(msg_id);
        self
    }
    /// Forward message
    pub fn set_forward_msg(&mut self, msg_id: MsgId, chat_id: ChatId) -> &mut Self {
        self.forward_msg_id = Some(msg_id);
        self.forward_chat_id = Some(chat_id);
        self
    }
    /// Set inline keyboard markup
    pub fn set_keyboard(&mut self, kb: Keyboard) -> &mut Self {
        self.inline_keyboard_markup = Some(kb.get_keyboard());
        self
    }
    /// Set format
    pub fn set_format(&mut self) -> &mut Self {
        self.format = None; //TODO: impl format
        self
    }
    /// Set parse mode
    pub fn set_parse_mode(&mut self, pm: ParseMode) -> &mut Self {
        self.parse_mode = Some(pm);
        self
    }
    /// Set text as HTML
    pub fn set_text(&mut self, (text, parse_mode): (String, ParseMode)) -> &mut Self {
        self.text = text;
        self.parse_mode = Some(parse_mode);
        self
    }
    /// Build [`RequestMessagesSendTextWithDeepLink`]
    pub fn build(&mut self) -> Self {
        self.clone()
    }
}
impl RequestMessagesSendTextWithDeepLink {
    /// Create new [`RequestMessagesSendTextWithDeepLink`] with required params
    pub fn new(chat_id: ChatId, deep_link: String) -> Self {
        Self {
            chat_id,
            deep_link,
            text: String::new(),
            reply_msg_id: None,
            forward_chat_id: None,
            forward_msg_id: None,
            inline_keyboard_markup: None,
            format: None,
            parse_mode: None,
        }
    }
}
impl RequestMessagesSendTextWithDeepLink {
    /// Set reply message id
    pub fn set_reply_msg_id(&mut self, msg_id: MsgId) -> &mut Self {
        self.reply_msg_id = Some(msg_id);
        self
    }
    /// Forward message
    pub fn set_forward_msg(&mut self, msg_id: MsgId, chat_id: ChatId) -> &mut Self {
        self.forward_msg_id = Some(msg_id);
        self.forward_chat_id = Some(chat_id);
        self
    }
    /// Set inline keyboard markup
    pub fn set_keyboard(&mut self, kb: Keyboard) -> &mut Self {
        self.inline_keyboard_markup = Some(kb.get_keyboard());
        self
    }
    /// Set format
    pub fn set_format(&mut self) -> &mut Self {
        self.format = None; //TODO: impl format
        self
    }
    /// Set parse mode
    pub fn set_parse_mode(&mut self, pm: ParseMode) -> &mut Self {
        self.parse_mode = Some(pm);
        self
    }
    /// Set text as HTML
    pub fn set_text(&mut self, (text, parse_mode): (String, ParseMode)) -> &mut Self {
        self.text = text;
        self.parse_mode = Some(parse_mode);
        self
    }
    /// Build [`RequestMessagesSendTextWithDeepLink`]
    pub fn build(&mut self) -> Self {
        self.clone()
    }
}
impl RequestMessagesEditText {
    /// Create new [`RequestMessagesEditText`] with required params
    pub fn new(chat_id: ChatId, msg_id: MsgId) -> Self {
        Self {
            chat_id,
            msg_id,
            text: String::new(),
            inline_keyboard_markup: None,
            format: None,
            parse_mode: None,
        }
    }
}
impl RequestMessagesEditText {
    /// Set inline keyboard markup
    pub fn set_keyboard(&mut self, kb: Keyboard) -> &mut Self {
        self.inline_keyboard_markup = Some(kb.get_keyboard());
        self
    }
    /// Set parse mode
    pub fn set_parse_mode(&mut self, pm: ParseMode) -> &mut Self {
        self.parse_mode = Some(pm);
        self
    }
    /// Set text as HTML
    pub fn set_text(&mut self, (text, parse_mode): (String, ParseMode)) -> &mut Self {
        self.text = text;
        self.parse_mode = Some(parse_mode);
        self
    }
    /// Build [`RequestMessagesEditText`]
    pub fn build(&mut self) -> Self {
        self.clone()
    }
}
impl RequestMessagesSendFile {
    /// Create new [`RequestMessagesSendFile`] with required params
    pub fn new(chat_id: ChatId) -> Self {
        Self {
            chat_id,
            caption: None,
            reply_msg_id: None,
            forward_chat_id: None,
            forward_msg_id: None,
            inline_keyboard_markup: None,
            format: None,
            parse_mode: None,
        }
    }
    /// Set caption
    pub fn set_caption(&mut self, caption: String) -> &mut Self {
        self.caption = Some(caption);
        self
    }
    /// Set reply message id
    pub fn set_reply_msg_id(&mut self, msg_id: MsgId) -> &mut Self {
        self.reply_msg_id = Some(msg_id);
        self
    }
    /// Forward message
    pub fn set_forward_msg(&mut self, msg_id: MsgId, chat_id: ChatId) -> &mut Self {
        self.forward_msg_id = Some(msg_id);
        self.forward_chat_id = Some(chat_id);
        self
    }
    /// Set inline keyboard markup
    pub fn set_keyboard(&mut self, kb: Keyboard) -> &mut Self {
        self.inline_keyboard_markup = Some(kb.get_keyboard());
        self
    }
    /// Build [`RequestMessagesSendFile`]
    pub fn build(&mut self) -> Self {
        self.clone()
    }
}
impl Default for MessageTextParser {
    /// Create new [`MessageText`] with required params
    fn default() -> Self {
        Self {
            text: vec![MessageTextFormat::None],
            parse_mode: ParseMode::HTML,
        }
    }
}
impl MessageTextParser {
    /// Add plain text to [`MessageText`]
    pub fn add(&mut self, text: MessageTextFormat) -> &mut Self {
        self.text.push(text);
        self
    }
    /// Line feed
    pub fn next_line(&mut self) -> &mut Self {
        self.text.push(MessageTextFormat::Plain(String::from("\n")));
        self
    }
    /// Space
    pub fn space(&mut self) -> &mut Self {
        self.text.push(MessageTextFormat::Plain(String::from(" ")));
        self
    }
    /// Parse [`MessageText`] to string
    pub fn parse(&self) -> (String, ParseMode) {
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
