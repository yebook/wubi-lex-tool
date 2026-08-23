//! Symmetric whitespace escaping shared by text codecs.

/// Replaces the six ASCII whitespace characters used by text formats with
/// their uppercase percent escapes.
#[must_use]
pub fn escape_whitespace(input: &str) -> String {
    let mut output = String::with_capacity(input.len());

    for character in input.chars() {
        match character {
            ' ' => output.push_str("%20"),
            '\t' => output.push_str("%09"),
            '\n' => output.push_str("%0A"),
            '\r' => output.push_str("%0D"),
            '\u{000b}' => output.push_str("%0B"),
            '\u{000c}' => output.push_str("%0C"),
            _ => output.push(character),
        }
    }

    output
}

/// Decodes only the six uppercase whitespace escapes supported by the text
/// formats. Unknown, lowercase, and incomplete percent sequences stay literal.
#[must_use]
pub fn unescape_whitespace(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut index = 0;

    while index < bytes.len() {
        let replacement = bytes
            .get(index..index.saturating_add(3))
            .and_then(decode_escape);

        if let Some(character) = replacement {
            output.push(character);
            index += 3;
            continue;
        }

        let Some(character) = input[index..].chars().next() else {
            break;
        };
        output.push(character);
        index += character.len_utf8();
    }

    output
}

fn decode_escape(candidate: &[u8]) -> Option<char> {
    match candidate {
        b"%20" => Some(' '),
        b"%09" => Some('\t'),
        b"%0A" => Some('\n'),
        b"%0D" => Some('\r'),
        b"%0B" => Some('\u{000b}'),
        b"%0C" => Some('\u{000c}'),
        _ => None,
    }
}
