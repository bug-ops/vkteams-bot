use clap::Parser;
use proptest::prelude::*;
use vkteams_bot_cli::cli::Cli;

proptest! {
    #[test]
    fn prop_cli_parse_random_args(args in proptest::collection::vec(".*", 0..10)) {
        // Всегда добавляем имя бинарника
        let mut full_args = vec!["vkteams-bot-cli".to_string()];
        full_args.extend(args);
        // Парсер не должен паниковать
        let _ = Cli::try_parse_from(&full_args);
    }

    #[test]
    fn prop_cli_parse_output_format_random(fmt in ".{0,16}") {
        let args = ["vkteams-bot-cli", "--output", &fmt, "config"];
        let _ = Cli::try_parse_from(&args);
    }
}
