pub enum CommandAction {
    SendIrc(String),
    Join(String),
    Local(String),
    Quit,
}

pub fn handle_input(input: &str, current_channel: Option<&str>, ) -> Result<CommandAction, String> {
    let input = input.trim();

    if input.is_empty() {
        return Ok(CommandAction::Local(String::new()));
    }

    if !input.starts_with('/') {
        return match current_channel {
            Some(channel) => Ok(CommandAction::SendIrc(format!("PRIVMSG {channel} :{input}"))),
            None => Err("Joing a channel before chatting.".to_string()),
        };
    }

    let (command, args) = input[1..]
        .split_once(' ')
        .unwrap_or((&input[1..], ""));

    let command = command.to_ascii_lowercase();
    let args = args.trim();

    match command.as_str() {
        "join" => {
            if args.is_empty() {
                return Err("Usage: /join #channel".to_string());
            }

            Ok(CommandAction::Join(args.to_string()))
        }

        "msg" => {
            let (target, message) = args
                .split_once(' ')
                .ok_or("Usage: /msg nick message")?;

            let message = message.trim();

            if target.is_empty() || message.is_empty() {
                return Err("Usage: /msg nick message".to_string());
            }

            Ok(CommandAction::SendIrc(format!("PRIVMSG: {target} :{message}")))
        }

        "nick" => {
            if args.is_empty() {
                return Err("Usage: /nick nick".to_string());
            }

            Ok(CommandAction::SendIrc(format!("NICK {args}")))
        }

        "quit" => {
            if !args.is_empty() {
                return Err("Usage: /quit".to_string());
            }

            Ok(CommandAction::Quit)
        }

        "help" => {
            if !args.is_empty() {
                return Err("Usage: /help".to_string());
            }

            Ok(CommandAction::Local("Commands: /join, /msg, /nick, /quit, /help".to_string()))
        }

        _ => Err(format!("Unknown command: /{command}, Use /help for a list of commands")),
    }
}
