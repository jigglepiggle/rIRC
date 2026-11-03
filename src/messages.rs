// [:prefix] COMMAND [param param ...] [:trailing text with spaces]

pub struct RawMessage<'line> {
    prefix: Option<&'line str>,
    command: &'line str,
    params: Vec<&'line str>,
}

pub fn parse_line(line: &str) -> Option<RawMessage<'_>> {
    let remaining = line.trim_end_matches(['\r', '\n']);

    let (prefix, remaining) = match remaining.strip_prefix(':') {
        Some(after_colon) => {
            let (prefix_text, after_prefix) = after_colon.split_once(' ')?;
            (Some(prefix_text), after_prefix.trim_start())
        }
        None => (None, remaining),
    };

    let (command, remaining) = remaining.split_once(' ').unwrap_or((remaining, ""));

    let is_word_command = !command.is_empty()
        && command
            .chars()
            .all(|character| character.is_ascii_uppercase());
    let is_numeric_reply =
        command.len() == 3 && command.chars().all(|character| character.is_ascii_digit());

    if !(is_word_command || is_numeric_reply) {
        return None; // Not a real IRC command, or not implemented
    }

    let mut params = Vec::new();
    let mut remaining = remaining;

    while !remaining.is_empty() {
        if let Some(trailing) = remaining.strip_prefix(':') {
            params.push(trailing);
            break;
        }
        match remaining.split_once(' ') {
            Some((param, after_param)) => {
                params.push(param);
                remaining = after_param;
            }
            None => {
                params.push(remaining);
                break;
            }
        }
    }

    Some(RawMessage {
        prefix,
        command,
        params,
    })
}

pub enum Source<'line> {
    Server(&'line str),
    User {
        nickname: &'line str,
        username: Option<&'line str>,
        host: Option<&'line str>,
    },
}

pub fn parse_source(prefix: &str) -> Source<'_> {
    match prefix.split_once('!') {
        Some((nickname, user_and_host)) => {
            let (username, host) = match user_and_host.split_once('@') {
                Some((username, host)) => (Some(username), Some(host)),
                None => (Some(user_and_host), None),
            };
            Source::User {
                nickname,
                username,
                host,
            }
        }
        None if prefix.contains('.') => Source::Server(prefix),
        None => Source::User {
            nickname: prefix,
            username: None,
            host: None,
        },
    }
}

pub enum Event<'line> {
    ServerReply {
        code: u16,
        params: Vec<&'line str>,
    },
    ChatMessage {
        sender: &'line str,
        target: &'line str,
        text: &'line str,
    },
    Join {
        nickname: &'line str,
        channel: &'line str,
    },
    Quit {
        nickname: &'line str,
        reason: &'line str,
    },
    Notice {
        target: &'line str,
        text: &'line str,
    },
    ServerError(&'line str),
    Unhandled(RawMessage<'line>),
}

pub fn classify(message: RawMessage<'_>) -> Event<'_> {
    if let Ok(code) = message.command.parse::<u16>() {
        return Event::ServerReply {
            code,
            params: message.params,
        };
    }

    let sender_nickname = match message.prefix.map(parse_source) {
        Some(Source::User { nickname, .. }) => Some(nickname),
        _ => None,
    };

    match (message.command, sender_nickname, message.params.as_slice()) {
        ("PRIVMSG", Some(sender), [target, text]) => Event::ChatMessage {
            sender,
            target,
            text,
        },
        ("JOIN", Some(nickname), [channel]) => Event::Join { nickname, channel },
        ("QUIT", Some(nickname), [reason]) => Event::Quit { nickname, reason },
        ("NOTICE", _, [target, text]) => Event::Notice { target, text },
        ("ERROR", _, [text]) => Event::ServerError(text),
        _ => Event::Unhandled(message),
    }
}
