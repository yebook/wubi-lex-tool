//! Shared strict two-column parsing for auxiliary text formats.

use std::num::NonZeroUsize;

use crate::{CodecError, CodecErrorKind, DecodeLimits, FieldValue};

use super::encoding::decode_bomless_utf8;

#[derive(Clone, Copy)]
pub(crate) struct Token<'a> {
    pub(crate) value: &'a str,
    pub(crate) column: usize,
}

pub(crate) fn decode_two_columns<T>(
    input: &[u8],
    limits: DecodeLimits,
    format: &'static str,
    line_field: &'static str,
    count_operation: &'static str,
    mut parse: impl FnMut(usize, Token<'_>, Token<'_>) -> Result<T, CodecError>,
) -> Result<Vec<T>, CodecError> {
    let text = decode_bomless_utf8(input, limits, format)?;
    let mut entries = Vec::new();

    for (index, raw_line) in text.split('\n').enumerate() {
        let line_number = index
            .checked_add(1)
            .ok_or_else(|| overflow("text line number"))?;
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if line.chars().all(char::is_whitespace) {
            continue;
        }

        let mut tokens = UnicodeTokens::new(line);
        let first = tokens.next().ok_or_else(|| {
            malformed(line_field, "exactly two nonempty columns", line)
                .at_text(nonzero(line_number), Some(NonZeroUsize::MIN))
        })?;
        let second = tokens.next().ok_or_else(|| {
            malformed(line_field, "exactly two nonempty columns", line)
                .at_text(nonzero(line_number), Some(nonzero(first.column)))
        })?;
        if let Some(extra) = tokens.next() {
            return Err(malformed(line_field, "exactly two nonempty columns", line)
                .at_text(nonzero(line_number), Some(nonzero(extra.column))));
        }

        let actual = entries
            .len()
            .checked_add(1)
            .ok_or_else(|| overflow(count_operation))?;
        limits
            .check_expanded_entries(actual)
            .map_err(|error| error.at_text(nonzero(line_number), Some(nonzero(first.column))))?;
        entries.try_reserve(1).map_err(|_| {
            overflow(count_operation).at_text(nonzero(line_number), Some(nonzero(first.column)))
        })?;
        entries.push(parse(line_number, first, second)?);
    }

    Ok(entries)
}

pub(crate) fn at_token(error: CodecError, line: usize, token: Token<'_>) -> CodecError {
    error.at_text(nonzero(line), Some(nonzero(token.column)))
}

pub(crate) fn malformed(field: &'static str, expected: &str, actual: &str) -> CodecError {
    CodecError::new(CodecErrorKind::MalformedField {
        field,
        expected: FieldValue::Text(expected.to_owned()),
        actual: FieldValue::Text(actual.to_owned()),
    })
}

pub(crate) fn append_tab_line(
    output: &mut String,
    first: &str,
    second: &str,
    operation: &'static str,
) -> Result<(), CodecError> {
    let additional = first
        .len()
        .checked_add(second.len())
        .and_then(|value| value.checked_add(2))
        .ok_or_else(|| overflow(operation))?;
    output
        .try_reserve(additional)
        .map_err(|_| overflow(operation))?;
    output.push_str(first);
    output.push('\t');
    output.push_str(second);
    output.push('\n');
    Ok(())
}

pub(crate) fn append_tab_u16_line(
    output: &mut String,
    first: &str,
    second: u16,
    operation: &'static str,
) -> Result<(), CodecError> {
    let digits = if second >= 10_000 {
        5
    } else if second >= 1_000 {
        4
    } else if second >= 100 {
        3
    } else if second >= 10 {
        2
    } else {
        1
    };
    let additional = first
        .len()
        .checked_add(digits)
        .and_then(|value| value.checked_add(2))
        .ok_or_else(|| overflow(operation))?;
    output
        .try_reserve(additional)
        .map_err(|_| overflow(operation))?;
    output.push_str(first);
    output.push('\t');

    let mut remaining = second;
    let mut divisor = 10_000u16;
    while divisor >= 10 && second < divisor {
        divisor /= 10;
    }
    loop {
        let digit = u8::try_from(remaining / divisor).map_err(|_| overflow(operation))?;
        output.push(char::from(b'0' + digit));
        remaining %= divisor;
        if divisor == 1 {
            break;
        }
        divisor /= 10;
    }
    output.push('\n');
    Ok(())
}

struct UnicodeTokens<'a> {
    content: &'a str,
    byte_cursor: usize,
    column_cursor: usize,
}

impl<'a> UnicodeTokens<'a> {
    const fn new(content: &'a str) -> Self {
        Self {
            content,
            byte_cursor: 0,
            column_cursor: 1,
        }
    }
}

impl<'a> Iterator for UnicodeTokens<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.byte_cursor < self.content.len() {
            let character = self.content[self.byte_cursor..].chars().next()?;
            if !character.is_whitespace() {
                break;
            }
            self.byte_cursor += character.len_utf8();
            self.column_cursor += 1;
        }
        if self.byte_cursor == self.content.len() {
            return None;
        }

        let byte_start = self.byte_cursor;
        let column = self.column_cursor;
        while self.byte_cursor < self.content.len() {
            let character = self.content[self.byte_cursor..].chars().next()?;
            if character.is_whitespace() {
                break;
            }
            self.byte_cursor += character.len_utf8();
            self.column_cursor += 1;
        }

        Some(Token {
            value: &self.content[byte_start..self.byte_cursor],
            column,
        })
    }
}

fn nonzero(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).unwrap_or(NonZeroUsize::MIN)
}

fn overflow(operation: &'static str) -> CodecError {
    CodecError::new(CodecErrorKind::IntegerOverflow { operation })
}
