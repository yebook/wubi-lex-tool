use std::num::NonZeroUsize;

use crate::{
    CodecError, CodecErrorKind, DecodeLimits, DetectedTextEncoding, FieldValue, LexCode,
    LexiconDocument, LexiconEntry, SourceLocation, Weight, escape::unescape_whitespace,
};

use super::{
    DecodedLexiconText, LexiconTextWarning, LexiconTextWarningKind, encoding::decode_bytes,
};

const MAX_WARNING_PREVIEW_CHARS: usize = 160;
const DESCENDING_WEIGHT_BASELINE: u16 = 5_000;

/// Decodes encoded community lexicon text using deterministic encoding and dialect detection.
pub fn decode(input: &[u8], limits: DecodeLimits) -> Result<DecodedLexiconText, CodecError> {
    let (text, detected_encoding) = decode_bytes(input, limits)?;
    parse_text(&text, detected_encoding, limits)
}

#[derive(Clone, Copy)]
struct LineRecord<'a> {
    number: usize,
    content: &'a str,
}

#[derive(Clone, Copy)]
struct Token<'a> {
    value: &'a str,
    column: usize,
    byte_start: usize,
    byte_end: usize,
}

#[derive(Clone)]
struct AsciiTokens<'a> {
    content: &'a str,
    byte_cursor: usize,
    column_cursor: usize,
}

impl<'a> AsciiTokens<'a> {
    const fn new(content: &'a str) -> Self {
        Self {
            content,
            byte_cursor: 0,
            column_cursor: 1,
        }
    }
}

impl<'a> Iterator for AsciiTokens<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.byte_cursor < self.content.len() {
            let character = self.content[self.byte_cursor..].chars().next()?;
            if !is_ascii_whitespace(character) {
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
            if is_ascii_whitespace(character) {
                break;
            }
            self.byte_cursor += character.len_utf8();
            self.column_cursor += 1;
        }

        Some(Token {
            value: &self.content[byte_start..self.byte_cursor],
            column,
            byte_start,
            byte_end: self.byte_cursor,
        })
    }
}

struct PendingEntry {
    entry: LexiconEntry,
    line: usize,
    text_column: usize,
    descending_weight: bool,
}

enum ParseOutcome {
    NoMatch,
    Matched(Vec<PendingEntry>),
    Invalid(CodecError),
}

struct EntryFields<'a> {
    code: Token<'a>,
    text: Token<'a>,
    weight: Option<(Token<'a>, WeightMode)>,
    descending_weight: bool,
}

fn parse_text(
    text: &str,
    detected_encoding: DetectedTextEncoding,
    limits: DecodeLimits,
) -> Result<DecodedLexiconText, CodecError> {
    let lines = line_records(text);
    let after_yaml = strip_yaml_front_matter(&lines)?;
    let uncommented: Vec<_> = after_yaml
        .iter()
        .copied()
        .filter(|line| !line.content.starts_with('#'))
        .collect();

    let text_header = uncommented
        .iter()
        .position(|line| trim_ascii_whitespace(line.content) == "[Text]");
    let (description, body) = match text_header {
        Some(index) => (&uncommented[..index], &uncommented[index + 1..]),
        None => (&[][..], uncommented.as_slice()),
    };
    let jidian = description_contains(description, "~:生僻字词")
        && description_contains(description, "^:用户词组");
    let first_nonempty = body
        .iter()
        .copied()
        .find(|line| !trim_ascii_whitespace(line.content).is_empty());

    let Some(first_body_line) = first_nonempty else {
        return Ok(DecodedLexiconText::new(
            LexiconDocument::default(),
            detected_encoding,
            Vec::new(),
        ));
    };

    let microsoft = text_header.is_some() && is_microsoft_signature(first_body_line.content);
    let mut pending = Vec::new();
    let mut warnings = Vec::new();
    let mut produced = 0usize;

    for line in body {
        if trim_ascii_whitespace(line.content).is_empty() {
            continue;
        }

        let outcome = if microsoft {
            parse_microsoft_line(*line, produced, limits)
        } else {
            parse_dialect_line(*line, produced, limits)
        };

        match outcome {
            ParseOutcome::Matched(mut entries) => {
                produced = checked_produced(produced, entries.len(), *line)?;
                pending.append(&mut entries);
            }
            ParseOutcome::Invalid(error) => return Err(error),
            ParseOutcome::NoMatch => {
                check_line_budget(produced, 1, *line, limits)?;
                produced = checked_produced(produced, 1, *line)?;
                warnings.push(warning_for_line(*line)?);
            }
        }
    }

    normalize_descending_weights(&mut pending)?;
    if jidian {
        pending = clean_jidian_entries(pending)?;
    }

    if pending.is_empty() {
        return Err(malformed(
            "text.body",
            "at least one supported lexicon entry",
            first_body_line.content,
        )
        .at_text(
            nonzero(first_body_line.number)?,
            Some(first_nonblank_column(first_body_line.content)?),
        ));
    }

    Ok(DecodedLexiconText::new(
        LexiconDocument::new(pending.into_iter().map(|pending| pending.entry).collect()),
        detected_encoding,
        warnings,
    ))
}

fn line_records(text: &str) -> Vec<LineRecord<'_>> {
    text.split('\n')
        .enumerate()
        .map(|(index, content)| LineRecord {
            number: index + 1,
            content: content.strip_suffix('\r').unwrap_or(content),
        })
        .collect()
}

fn strip_yaml_front_matter<'a>(
    lines: &'a [LineRecord<'a>],
) -> Result<&'a [LineRecord<'a>], CodecError> {
    let Some(first) = lines.first() else {
        return Ok(lines);
    };
    if trim_ascii_whitespace(first.content) != "---" {
        return Ok(lines);
    }

    let Some(end) = lines
        .iter()
        .skip(1)
        .position(|line| trim_ascii_whitespace(line.content) == "...")
    else {
        return Err(malformed(
            "text.yaml_front_matter",
            "a closing ... line",
            "end of input",
        )
        .at_text(nonzero(first.number)?, Some(nonzero(1)?)));
    };

    Ok(&lines[end + 2..])
}

fn description_contains(lines: &[LineRecord<'_>], marker: &str) -> bool {
    lines.iter().any(|line| line.content.contains(marker))
}

fn parse_dialect_line(line: LineRecord<'_>, produced: usize, limits: DecodeLimits) -> ParseOutcome {
    let parsers = [
        parse_a as fn(LineRecord<'_>, usize, DecodeLimits) -> ParseOutcome,
        parse_b,
        parse_c,
        parse_d,
        parse_e,
        parse_f,
    ];

    for parser in parsers {
        match parser(line, produced, limits) {
            ParseOutcome::NoMatch => {}
            outcome => return outcome,
        }
    }

    ParseOutcome::NoMatch
}

fn parse_a(line: LineRecord<'_>, produced: usize, limits: DecodeLimits) -> ParseOutcome {
    let mut tokens = tokens(line.content);
    let (Some(code), Some(second)) = (tokens.next(), tokens.next()) else {
        return ParseOutcome::NoMatch;
    };
    let third = tokens.next();
    if tokens.next().is_some() || !looks_like_code_field(code.value) {
        return ParseOutcome::NoMatch;
    }

    let (text, weight) = if let Some(weight) = third {
        if !is_ascii_decimal(weight.value) && !is_signed_ascii_decimal(weight.value) {
            return ParseOutcome::NoMatch;
        }
        (second, weight)
    } else {
        let separator = &line.content[code.byte_end..second.byte_start];
        if (!is_ascii_decimal(second.value) && !is_signed_ascii_decimal(second.value))
            || separator.chars().count() < 2
        {
            return ParseOutcome::NoMatch;
        }
        let empty_offset = code.byte_end + 1;
        (
            Token {
                value: "",
                column: code.column + code.value.chars().count() + 1,
                byte_start: empty_offset,
                byte_end: empty_offset,
            },
            second,
        )
    };

    one_entry(
        line,
        EntryFields {
            code,
            text,
            weight: Some((weight, WeightMode::Ascending)),
            descending_weight: false,
        },
        produced,
        limits,
    )
}

fn parse_b(line: LineRecord<'_>, produced: usize, limits: DecodeLimits) -> ParseOutcome {
    let trimmed = trim_ascii_whitespace(line.content);
    let leading = line.content.len() - line.content.trim_start_matches(is_ascii_whitespace).len();
    let Some(comma) = trimmed.find(',') else {
        return ParseOutcome::NoMatch;
    };
    let Some(equals) = trimmed[..comma].find('=') else {
        return ParseOutcome::NoMatch;
    };

    let (code_value, code_offset) = trim_with_offset(&trimmed[..equals], leading);
    let (weight_value, weight_offset) =
        trim_with_offset(&trimmed[equals + 1..comma], leading + equals + 1);
    let (text_value, text_offset) = trim_with_offset(&trimmed[comma + 1..], leading + comma + 1);
    let code = Token {
        value: code_value,
        column: column_at(line.content, code_offset),
        byte_start: code_offset,
        byte_end: code_offset + code_value.len(),
    };
    let weight = Token {
        value: weight_value,
        column: column_at(line.content, weight_offset),
        byte_start: weight_offset,
        byte_end: weight_offset + weight_value.len(),
    };
    let text = Token {
        value: text_value,
        column: column_at(line.content, text_offset),
        byte_start: text_offset,
        byte_end: text_offset + text_value.len(),
    };

    one_entry(
        line,
        EntryFields {
            code,
            text,
            weight: Some((weight, WeightMode::Ascending)),
            descending_weight: false,
        },
        produced,
        limits,
    )
}

fn parse_c(line: LineRecord<'_>, produced: usize, limits: DecodeLimits) -> ParseOutcome {
    let mut tokens = tokens(line.content);
    let (Some(code), Some(first_text)) = (tokens.next(), tokens.next()) else {
        return ParseOutcome::NoMatch;
    };
    if !looks_like_code_field(code.value) {
        return ParseOutcome::NoMatch;
    }

    many_texts_for_code(
        line,
        code,
        std::iter::once(first_text).chain(tokens),
        produced,
        limits,
    )
}

fn parse_d(line: LineRecord<'_>, produced: usize, limits: DecodeLimits) -> ParseOutcome {
    let mut tokens = tokens(line.content);
    let (Some(text), Some(code), Some(weight_and_suffix)) =
        (tokens.next(), tokens.next(), tokens.next())
    else {
        return ParseOutcome::NoMatch;
    };
    if !looks_like_code_field(code.value) {
        return ParseOutcome::NoMatch;
    }

    let Some(weight) = descending_weight_token(weight_and_suffix) else {
        return ParseOutcome::NoMatch;
    };
    let trailing_suffix = tokens.next();
    if tokens.next().is_some()
        || trailing_suffix.is_some_and(|token| {
            !token
                .value
                .chars()
                .all(|character| character.is_ascii_lowercase())
        })
    {
        return ParseOutcome::NoMatch;
    }

    one_entry(
        line,
        EntryFields {
            code,
            text,
            weight: Some((weight, WeightMode::Descending)),
            descending_weight: true,
        },
        produced,
        limits,
    )
}

fn descending_weight_token(token: Token<'_>) -> Option<Token<'_>> {
    let decimal_len = token.value.bytes().take_while(u8::is_ascii_digit).count();
    if decimal_len == 0 {
        return is_signed_ascii_decimal(token.value).then_some(token);
    }

    let suffix = &token.value[decimal_len..];
    if !suffix.bytes().all(|byte| byte.is_ascii_lowercase()) {
        return None;
    }

    Some(Token {
        value: &token.value[..decimal_len],
        column: token.column,
        byte_start: token.byte_start,
        byte_end: token.byte_start + decimal_len,
    })
}

fn parse_e(line: LineRecord<'_>, produced: usize, limits: DecodeLimits) -> ParseOutcome {
    let mut tokens = tokens(line.content);
    let (Some(text), Some(code)) = (tokens.next(), tokens.next()) else {
        return ParseOutcome::NoMatch;
    };
    if tokens.next().is_some() || !looks_like_code_field(code.value) {
        return ParseOutcome::NoMatch;
    }

    one_entry(
        line,
        EntryFields {
            code,
            text,
            weight: None,
            descending_weight: false,
        },
        produced,
        limits,
    )
}

fn parse_f(line: LineRecord<'_>, produced: usize, limits: DecodeLimits) -> ParseOutcome {
    let mut tokens = tokens(line.content);
    let (Some(text), Some(first_code)) = (tokens.next(), tokens.next()) else {
        return ParseOutcome::NoMatch;
    };
    let codes = std::iter::once(first_code).chain(tokens);
    if !codes
        .clone()
        .all(|token| looks_like_code_field(token.value))
    {
        return ParseOutcome::NoMatch;
    }

    one_text_for_codes(line, text, codes, produced, limits)
}

fn is_microsoft_signature(content: &str) -> bool {
    let mut tokens = tokens(content);
    let (Some(text), Some(first_code)) = (tokens.next(), tokens.next()) else {
        return false;
    };
    !text
        .value
        .chars()
        .any(|character| character.is_ascii_lowercase())
        && std::iter::once(first_code)
            .chain(tokens)
            .all(|token| looks_like_code_field(token.value))
}

fn parse_microsoft_line(
    line: LineRecord<'_>,
    produced: usize,
    limits: DecodeLimits,
) -> ParseOutcome {
    let mut tokens = tokens(line.content);
    let (Some(text), Some(first_code)) = (tokens.next(), tokens.next()) else {
        return ParseOutcome::NoMatch;
    };
    if text
        .value
        .chars()
        .any(|character| character.is_ascii_lowercase())
    {
        return ParseOutcome::NoMatch;
    }
    let codes = std::iter::once(first_code).chain(tokens);
    if !codes
        .clone()
        .all(|token| looks_like_code_field(token.value))
    {
        return ParseOutcome::NoMatch;
    }

    one_text_for_codes(line, text, codes, produced, limits)
}

fn many_texts_for_code<'a>(
    line: LineRecord<'a>,
    code: Token<'a>,
    texts: impl IntoIterator<Item = Token<'a>>,
    produced: usize,
    limits: DecodeLimits,
) -> ParseOutcome {
    let code = match LexCode::new(code.value) {
        Ok(code) => code,
        Err(error) => return ParseOutcome::Invalid(at_token(error, line, code)),
    };
    let mut entries = Vec::new();
    for text in texts {
        if let Err(error) = check_line_budget(produced, entries.len() + 1, line, limits) {
            return ParseOutcome::Invalid(error);
        }
        let value = unescape_whitespace(text.value);
        let entry = match LexiconEntry::new(code.clone(), value, None) {
            Ok(entry) => entry,
            Err(error) => return ParseOutcome::Invalid(at_token(error, line, text)),
        };
        entries.push(PendingEntry {
            entry,
            line: line.number,
            text_column: text.column,
            descending_weight: false,
        });
    }
    ParseOutcome::Matched(entries)
}

fn one_text_for_codes<'a>(
    line: LineRecord<'a>,
    text: Token<'a>,
    codes: impl IntoIterator<Item = Token<'a>>,
    produced: usize,
    limits: DecodeLimits,
) -> ParseOutcome {
    let value = unescape_whitespace(text.value);
    let mut entries = Vec::new();
    for code in codes {
        if let Err(error) = check_line_budget(produced, entries.len() + 1, line, limits) {
            return ParseOutcome::Invalid(error);
        }
        let code_value = match LexCode::new(code.value) {
            Ok(code_value) => code_value,
            Err(error) => return ParseOutcome::Invalid(at_token(error, line, code)),
        };
        let entry = match LexiconEntry::new(code_value, value.clone(), None) {
            Ok(entry) => entry,
            Err(error) => return ParseOutcome::Invalid(at_token(error, line, text)),
        };
        entries.push(PendingEntry {
            entry,
            line: line.number,
            text_column: text.column,
            descending_weight: false,
        });
    }
    ParseOutcome::Matched(entries)
}

#[derive(Clone, Copy)]
enum WeightMode {
    Ascending,
    Descending,
}

fn one_entry(
    line: LineRecord<'_>,
    fields: EntryFields<'_>,
    produced: usize,
    limits: DecodeLimits,
) -> ParseOutcome {
    if let Err(error) = check_line_budget(produced, 1, line, limits) {
        return ParseOutcome::Invalid(error);
    }
    let code_value = match LexCode::new(fields.code.value) {
        Ok(code_value) => code_value,
        Err(error) => return ParseOutcome::Invalid(at_token(error, line, fields.code)),
    };
    let weight_value = match fields.weight {
        Some((token, mode)) => match parse_weight(token, mode) {
            Ok(weight) => Some(weight),
            Err(error) => return ParseOutcome::Invalid(at_token(error, line, token)),
        },
        None => None,
    };
    let text_value = unescape_whitespace(fields.text.value);
    let entry = match LexiconEntry::new(code_value, text_value, weight_value) {
        Ok(entry) => entry,
        Err(error) => return ParseOutcome::Invalid(at_token(error, line, fields.text)),
    };
    ParseOutcome::Matched(vec![PendingEntry {
        entry,
        line: line.number,
        text_column: fields.text.column,
        descending_weight: fields.descending_weight,
    }])
}

fn parse_weight(token: Token<'_>, mode: WeightMode) -> Result<Weight, CodecError> {
    let source = token.value.parse::<u64>().map_err(|_| {
        malformed(
            "lexicon weight",
            match mode {
                WeightMode::Ascending => "an integer in 1..=65535",
                WeightMode::Descending => "an integer in 0..=65534",
            },
            token.value,
        )
    })?;
    let transformed = match mode {
        WeightMode::Ascending => u16::try_from(source)
            .map_err(|_| malformed("lexicon weight", "an integer in 1..=65535", token.value))?,
        WeightMode::Descending => {
            let source = u16::try_from(source)
                .map_err(|_| malformed("lexicon weight", "an integer in 0..=65534", token.value))?;
            u16::MAX - source
        }
    };
    Weight::new(transformed)
}

fn normalize_descending_weights(entries: &mut [PendingEntry]) -> Result<(), CodecError> {
    let minimum = entries
        .iter()
        .filter(|entry| entry.descending_weight)
        .filter_map(|entry| entry.entry.weight().map(Weight::get))
        .min();
    let Some(minimum) = minimum.filter(|minimum| *minimum > DESCENDING_WEIGHT_BASELINE) else {
        return Ok(());
    };
    let adjustment = minimum - DESCENDING_WEIGHT_BASELINE;

    for pending in entries.iter_mut().filter(|entry| entry.descending_weight) {
        let Some(weight) = pending.entry.weight() else {
            continue;
        };
        let normalized = Weight::new(weight.get() - adjustment)?;
        pending.entry = LexiconEntry::new(
            pending.entry.code().clone(),
            pending.entry.text(),
            Some(normalized),
        )?;
    }
    Ok(())
}

fn clean_jidian_entries(entries: Vec<PendingEntry>) -> Result<Vec<PendingEntry>, CodecError> {
    let mut cleaned = Vec::with_capacity(entries.len());
    for mut pending in entries {
        let text = pending.entry.text();
        if matches!(text.chars().next(), Some('^' | '$' | '!')) {
            continue;
        }
        let text = text.strip_prefix('~').unwrap_or(text);
        let entry =
            LexiconEntry::new(pending.entry.code().clone(), text, None).map_err(|error| {
                error.at_text(
                    NonZeroUsize::new(pending.line).unwrap_or(NonZeroUsize::MIN),
                    NonZeroUsize::new(pending.text_column),
                )
            })?;
        pending.entry = entry;
        pending.descending_weight = false;
        cleaned.push(pending);
    }
    Ok(cleaned)
}

fn warning_for_line(line: LineRecord<'_>) -> Result<LexiconTextWarning, CodecError> {
    let mut characters = line.content.chars();
    let preview: String = characters
        .by_ref()
        .take(MAX_WARNING_PREVIEW_CHARS)
        .collect();
    let truncated = characters.next().is_some();
    Ok(LexiconTextWarning::new(
        LexiconTextWarningKind::UnrecognizedLine,
        SourceLocation::Text {
            line: nonzero(line.number)?,
            column: Some(first_nonblank_column(line.content)?),
        },
        preview,
        truncated,
    ))
}

fn check_line_budget(
    produced: usize,
    additional: usize,
    line: LineRecord<'_>,
    limits: DecodeLimits,
) -> Result<(), CodecError> {
    let actual = produced.checked_add(additional).ok_or_else(|| {
        CodecError::new(CodecErrorKind::IntegerOverflow {
            operation: "lexicon text output count",
        })
    })?;
    limits.check_expanded_entries(actual).map_err(|error| {
        error.at_text(
            NonZeroUsize::new(line.number).unwrap_or(NonZeroUsize::MIN),
            Some(NonZeroUsize::MIN),
        )
    })
}

fn checked_produced(
    produced: usize,
    additional: usize,
    line: LineRecord<'_>,
) -> Result<usize, CodecError> {
    produced.checked_add(additional).ok_or_else(|| {
        CodecError::new(CodecErrorKind::IntegerOverflow {
            operation: "lexicon text output count",
        })
        .at_text(
            NonZeroUsize::new(line.number).unwrap_or(NonZeroUsize::MIN),
            Some(NonZeroUsize::MIN),
        )
    })
}

fn at_token(error: CodecError, line: LineRecord<'_>, token: Token<'_>) -> CodecError {
    error.at_text(
        NonZeroUsize::new(line.number).unwrap_or(NonZeroUsize::MIN),
        NonZeroUsize::new(token.column),
    )
}

const fn tokens(content: &str) -> AsciiTokens<'_> {
    AsciiTokens::new(content)
}

fn trim_with_offset(value: &str, base_offset: usize) -> (&str, usize) {
    let trimmed_start = value.trim_start_matches(is_ascii_whitespace);
    let removed = value.len() - trimmed_start.len();
    (
        trimmed_start.trim_end_matches(is_ascii_whitespace),
        base_offset + removed,
    )
}

fn trim_ascii_whitespace(value: &str) -> &str {
    value.trim_matches(is_ascii_whitespace)
}

const fn is_ascii_whitespace(character: char) -> bool {
    matches!(
        character,
        ' ' | '\t' | '\n' | '\r' | '\u{000b}' | '\u{000c}'
    )
}

fn looks_like_code_field(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
        && value
            .chars()
            .any(|character| character.is_ascii_alphabetic())
}

fn is_ascii_decimal(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn is_signed_ascii_decimal(value: &str) -> bool {
    value.strip_prefix(['-', '+']).is_some_and(is_ascii_decimal)
}

fn column_at(content: &str, byte_offset: usize) -> usize {
    content[..byte_offset].chars().count() + 1
}

fn first_nonblank_column(content: &str) -> Result<NonZeroUsize, CodecError> {
    let column = content
        .chars()
        .take_while(|character| is_ascii_whitespace(*character))
        .count()
        + 1;
    nonzero(column)
}

fn nonzero(value: usize) -> Result<NonZeroUsize, CodecError> {
    NonZeroUsize::new(value).ok_or_else(|| {
        CodecError::new(CodecErrorKind::IntegerOverflow {
            operation: "one-based text location",
        })
    })
}

fn malformed(field: &'static str, expected: &str, actual: &str) -> CodecError {
    CodecError::new(CodecErrorKind::MalformedField {
        field,
        expected: FieldValue::Text(expected.to_owned()),
        actual: FieldValue::Text(actual.to_owned()),
    })
}
