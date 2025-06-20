// Property-based tests for vkteams-bot/src/api/types.rs
use proptest::prelude::*;
use std::convert::From as StdFrom;
use vkteams_bot::{
    ApiResponseWrapper, ButtonKeyboard, ButtonStyle, Chat, ChatId, EventMessage,
    EventPayloadNewMessage, EventType, From, Keyboard, MsgId, Timestamp, UserId,
};

// Newtype-обёртка для ButtonKeyboard
#[derive(Debug, Clone)]
struct ButtonKeyboardGen {
    text: String,
    url: Option<String>,
    callback_data: Option<String>,
    style: ButtonStyle,
}

impl Arbitrary for ButtonKeyboardGen {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            ".{1,16}",
            proptest::option::of(".{0,16}"),
            proptest::option::of(".{0,16}"),
            prop_oneof![
                Just(ButtonStyle::Primary),
                Just(ButtonStyle::Attention),
                Just(ButtonStyle::Base)
            ],
        )
            .prop_map(|(text, url, callback_data, style)| ButtonKeyboardGen {
                text,
                url,
                callback_data,
                style,
            })
            .boxed()
    }
}

impl StdFrom<ButtonKeyboardGen> for ButtonKeyboard {
    fn from(inner: ButtonKeyboardGen) -> Self {
        ButtonKeyboard {
            text: inner.text,
            url: inner.url,
            callback_data: inner.callback_data,
            style: Some(inner.style),
        }
    }
}

// Newtype-обёртка для Keyboard
#[derive(Debug, Clone)]
struct KeyboardGen {
    buttons: Vec<Vec<ButtonKeyboardGen>>,
}

impl Arbitrary for KeyboardGen {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        proptest::collection::vec(
            proptest::collection::vec(any::<ButtonKeyboardGen>(), 0..3),
            0..3,
        )
        .prop_map(|buttons| KeyboardGen { buttons })
        .boxed()
    }
}

impl StdFrom<KeyboardGen> for Keyboard {
    fn from(inner: KeyboardGen) -> Self {
        Keyboard {
            buttons: inner
                .buttons
                .into_iter()
                .map(|row| row.into_iter().map(ButtonKeyboard::from).collect())
                .collect(),
        }
    }
}

proptest! {
    #[test]
    fn prop_roundtrip_event_type(event_id in any::<u32>(), text in ".{0,32}") {
        let msg = EventMessage {
            event_id,
            event_type: EventType::NewMessage(Box::new(EventPayloadNewMessage {
                msg_id: MsgId("mid".to_string()),
                text: text.clone(),
                chat: Chat {
                    chat_id: ChatId::from("cid"),
                    title: None,
                    chat_type: "private".to_string(),
                },
                from: From {
                    first_name: "A".to_string(),
                    last_name: None,
                    user_id: UserId("uid".to_string()),
                },
                format: None,
                parts: vec![],
                timestamp: Timestamp(0),
            })),
        };
        let ser = serde_json::to_string(&msg).unwrap();
        let de: EventMessage = serde_json::from_str(&ser).unwrap();
        assert_eq!(de.event_id, event_id);
        if let EventType::NewMessage(nm) = de.event_type {
            assert_eq!(nm.text, text);
        } else {
            panic!("Expected NewMessage");
        }
    }

    #[test]
    fn prop_roundtrip_button_keyboard(btn in any::<ButtonKeyboardGen>()) {
        let btn: ButtonKeyboard = btn.into();
        let ser = serde_json::to_string(&btn).unwrap();
        let de: ButtonKeyboard = serde_json::from_str(&ser).unwrap();
        assert_eq!(de.text, btn.text);
        assert_eq!(de.url, btn.url);
        assert_eq!(de.callback_data, btn.callback_data);
        assert_eq!(de.style, btn.style);
    }

    #[test]
    fn prop_roundtrip_keyboard(kb in any::<KeyboardGen>()) {
        let kb: Keyboard = kb.into();
        let ser = serde_json::to_string(&kb).unwrap();
        let de: Keyboard = serde_json::from_str(&ser).unwrap();
        // Сравниваем сериализованные строки, чтобы избежать проблем с PartialEq
        let orig = serde_json::to_string(&kb).unwrap();
        let deser = serde_json::to_string(&de).unwrap();
        assert_eq!(orig, deser);
    }

    #[test]
    fn prop_api_response_wrapper_payloadonly(val in ".{0,32}") {
        let wrap = ApiResponseWrapper::PayloadOnly(val.clone());
        let ser = serde_json::to_string(&wrap).unwrap();
        let de: ApiResponseWrapper<String> = serde_json::from_str(&ser).unwrap();
        match de {
            ApiResponseWrapper::PayloadOnly(v) => assert_eq!(v, val),
            _ => panic!("Expected PayloadOnly"),
        }
    }

    #[test]
    fn prop_roundtrip_chat(chat_id in ".{1,16}", title in proptest::option::of(".{0,16}"), chat_type in "private|group|channel") {
        let chat = Chat {
            chat_id: ChatId::from(chat_id.clone()),
            title: title.clone(),
            chat_type: chat_type.to_string(),
        };
        let ser = serde_json::to_string(&chat).unwrap();
        let de: Chat = serde_json::from_str(&ser).unwrap();
        assert_eq!(de.chat_id.as_ref(), chat_id);
        assert_eq!(de.title, title);
        assert_eq!(de.chat_type, chat_type);
    }
}
