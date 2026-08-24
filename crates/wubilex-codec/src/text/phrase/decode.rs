use std::{collections::HashMap, num::NonZeroUsize};

use crate::{
    Candidate, CodecError, CodecErrorKind, DecodeLimits, FieldValue, InvalidInputReason,
    PhraseCode, PhraseDocument, PhraseEntry, SourceLocation, escape::try_unescape_whitespace,
    text::encoding::decode_bytes,
};

use super::{DecodedPhraseText, PhraseTextWarning, PhraseTextWarningKind};

const MAX_WARNING_PREVIEW_CHARS: usize = 160;

/// Decodes encoded community phrase text using deterministic encoding and dialect detection.
pub fn decode(input: &[u8], limits: DecodeLimits) -> Result<DecodedPhraseText, CodecError> {
    let (text, detected_encoding) = decode_bytes(input, limits)?;
    let cleaned = strip_comments(&text)?;
    let mut entries = Vec::new();
    let mut warnings = Vec::new();
    let mut max_candidates = HashMap::<String, u8>::new();
    let mut pending: Option<PendingRecord> = None;
    let mut produced = 0usize;
    let mut first_body = None;

    for line in line_records(&cleaned, &text) {
        let line = line?;
        if line.cleaned.trim().is_empty() {
            continue;
        }
        if first_body.is_none() {
            first_body = Some(line);
        }

        let parsed = parse_line(line);
        if let Some(current) = pending.as_mut() {
            match &parsed {
                ParseOutcome::NoMatch => {
                    let continuation = line.cleaned.trim();
                    let additional = continuation
                        .len()
                        .checked_add(usize::from(!current.record.text.is_empty()))
                        .ok_or_else(|| overflow("phrase multiline text size"))?;
                    current.record.text.try_reserve(additional).map_err(|_| {
                        overflow("phrase multiline text allocation").at_text(
                            nonzero(line.number),
                            Some(first_nonblank_column(line.cleaned)),
                        )
                    })?;
                    if !current.record.text.is_empty() {
                        current.record.text.push('\n');
                    }
                    current.record.text.push_str(continuation);
                    continue;
                }
                ParseOutcome::Invalid(_) | ParseOutcome::Matched(_) => {
                    let completed = pending
                        .take()
                        .ok_or_else(|| overflow("phrase multiline pending state"))?;
                    process_record(
                        completed.record,
                        &mut entries,
                        &mut max_candidates,
                        &mut produced,
                        limits,
                    )?;
                }
            }
        }

        match parsed {
            ParseOutcome::NoMatch => {
                check_budget(produced, line, limits)?;
                warnings.try_reserve(1).map_err(|_| {
                    overflow("phrase warning allocation").at_text(
                        nonzero(line.number),
                        Some(first_nonblank_column(line.cleaned)),
                    )
                })?;
                warnings.push(warning_for_line(line)?);
                produced = checked_increment(produced, "phrase text output count")?;
            }
            ParseOutcome::Invalid(error) => return Err(error),
            ParseOutcome::Matched(record) => {
                if record.text.is_empty() && record.multiline_capable {
                    pending = Some(PendingRecord { record });
                } else {
                    process_record(
                        record,
                        &mut entries,
                        &mut max_candidates,
                        &mut produced,
                        limits,
                    )?;
                }
            }
        }
    }

    if let Some(completed) = pending {
        process_record(
            completed.record,
            &mut entries,
            &mut max_candidates,
            &mut produced,
            limits,
        )?;
    }

    let Some(first_body) = first_body else {
        return Ok(DecodedPhraseText::new(
            PhraseDocument::default(),
            detected_encoding,
            warnings,
        ));
    };

    if entries.is_empty() {
        return Err(malformed(
            "phrase_text.body",
            "at least one supported phrase entry",
            first_body.original,
        )
        .at_text(
            nonzero(first_body.number),
            Some(first_nonblank_column(first_body.original)),
        ));
    }

    Ok(DecodedPhraseText::new(
        PhraseDocument::new(entries),
        detected_encoding,
        warnings,
    ))
}

#[derive(Clone, Copy)]
struct LineRecord<'a> {
    number: usize,
    cleaned: &'a str,
    original: &'a str,
}

#[derive(Clone, Copy)]
struct Token<'a> {
    value: &'a str,
    column: usize,
    byte_start: usize,
    byte_end: usize,
}

struct ParsedRecord {
    code: PhraseCode,
    text: String,
    candidate: Option<Candidate>,
    line: usize,
    code_column: usize,
    text_column: usize,
    aliases: bool,
    multiline_capable: bool,
}

struct PendingRecord {
    record: ParsedRecord,
}

enum ParseOutcome {
    NoMatch,
    Matched(ParsedRecord),
    Invalid(CodecError),
}

fn line_records<'a>(
    cleaned: &'a str,
    original: &'a str,
) -> impl Iterator<Item = Result<LineRecord<'a>, CodecError>> {
    cleaned
        .split('\n')
        .zip(original.split('\n'))
        .enumerate()
        .map(|(index, (cleaned_line, original_line))| {
            Ok(LineRecord {
                number: index
                    .checked_add(1)
                    .ok_or_else(|| overflow("phrase line number"))?,
                cleaned: cleaned_line.strip_suffix('\r').unwrap_or(cleaned_line),
                original: original_line.strip_suffix('\r').unwrap_or(original_line),
            })
        })
}

fn strip_comments(input: &str) -> Result<String, CodecError> {
    let mut output = String::new();
    output
        .try_reserve_exact(input.len())
        .map_err(|_| overflow("phrase comment output allocation"))?;
    let mut characters = input.chars().peekable();
    let mut in_comment = false;
    let mut comment_start = None;
    let mut line = 1usize;
    let mut column = 1usize;

    while let Some(character) = characters.next() {
        let next = characters.peek().copied();
        if !in_comment && character == '/' && next == Some('*') {
            comment_start = Some((line, column));
            in_comment = true;
            output.push(' ');
            output.push(' ');
            let _ = characters.next();
            column = column
                .checked_add(2)
                .ok_or_else(|| overflow("phrase comment column"))?;
            continue;
        }
        if in_comment && character == '*' && next == Some('/') {
            in_comment = false;
            output.push(' ');
            output.push(' ');
            let _ = characters.next();
            column = column
                .checked_add(2)
                .ok_or_else(|| overflow("phrase comment column"))?;
            continue;
        }

        if in_comment && !matches!(character, '\n' | '\r') {
            output.push(' ');
        } else {
            output.push(character);
        }
        if character == '\n' {
            line = line
                .checked_add(1)
                .ok_or_else(|| overflow("phrase comment line"))?;
            column = 1;
        } else {
            column = column
                .checked_add(1)
                .ok_or_else(|| overflow("phrase comment column"))?;
        }
    }

    if in_comment {
        let (line, column) = comment_start.ok_or_else(|| overflow("phrase comment state"))?;
        return Err(malformed(
            "phrase_text.comment",
            "a closing */ delimiter",
            "end of input",
        )
        .at_text(nonzero(line), Some(nonzero(column))));
    }
    Ok(output)
}

fn parse_line(line: LineRecord<'_>) -> ParseOutcome {
    let (trimmed, leading) = trim_with_offset(line.cleaned, 0);
    if trimmed.is_empty() {
        return ParseOutcome::NoMatch;
    }
    if let Some(equals) = trimmed.find('=') {
        return parse_equals(line, trimmed, leading, equals);
    }
    parse_whitespace(line, trimmed, leading)
}

fn parse_equals(
    line: LineRecord<'_>,
    trimmed: &str,
    leading: usize,
    equals: usize,
) -> ParseOutcome {
    let left = &trimmed[..equals];
    let right = &trimmed[equals + 1..];
    if let Some(comma) = left.find(',') {
        let (code, code_offset) = trim_with_offset(&left[..comma], leading);
        let (candidate, candidate_offset) =
            trim_with_offset(&left[comma + 1..], leading + comma + 1);
        let (mut text, mut text_offset) = trim_with_offset(right, leading + equals + 1);
        if let Some(stripped) = text.strip_prefix('#') {
            text = stripped;
            text_offset += 1;
        }
        return build_record(
            line,
            token(line.cleaned, code, code_offset),
            token(line.cleaned, text, text_offset),
            Some(token(line.cleaned, candidate, candidate_offset)),
            true,
            true,
        );
    }

    if let Some(comma) = right.find(',') {
        let candidate_raw = &right[..comma];
        let (candidate, candidate_offset) = trim_with_offset(candidate_raw, leading + equals + 1);
        if is_numeric_shape(candidate) {
            let (code, code_offset) = trim_with_offset(left, leading);
            let (text, text_offset) =
                trim_with_offset(&right[comma + 1..], leading + equals + comma + 2);
            return build_record(
                line,
                token(line.cleaned, code, code_offset),
                token(line.cleaned, text, text_offset),
                Some(token(line.cleaned, candidate, candidate_offset)),
                false,
                false,
            );
        }
    }

    let (code, code_offset) = trim_with_offset(left, leading);
    let (text, text_offset) = trim_with_offset(right, leading + equals + 1);
    build_record(
        line,
        token(line.cleaned, code, code_offset),
        token(line.cleaned, text, text_offset),
        None,
        false,
        true,
    )
}

fn parse_whitespace(line: LineRecord<'_>, trimmed: &str, leading: usize) -> ParseOutcome {
    let mut tokens = UnicodeTokens::new(line.cleaned, trimmed, leading);
    let Some(first) = tokens.next() else {
        return ParseOutcome::NoMatch;
    };

    let Some(second) = tokens.next() else {
        return ParseOutcome::NoMatch;
    };
    if looks_like_code_field(first.value) {
        let mut count = 2usize;
        let mut last = second;
        for next in tokens {
            count = match count.checked_add(1) {
                Some(value) => value,
                None => return ParseOutcome::Invalid(overflow("phrase line token count")),
            };
            last = next;
        }
        if count >= 3 && is_numeric_shape(last.value) {
            let (text, text_offset) = trim_with_offset(
                &line.cleaned[first.byte_end..last.byte_start],
                first.byte_end,
            );
            return build_record(
                line,
                first,
                token(line.cleaned, text, text_offset),
                Some(last),
                false,
                false,
            );
        }

        let (text, text_offset) = trim_with_offset(&line.cleaned[first.byte_end..], first.byte_end);
        return build_record(
            line,
            first,
            token(line.cleaned, text, text_offset),
            None,
            false,
            false,
        );
    }

    if let Some(code) = p6_code_token(second)
        && tokens.all(|token| is_ascii_decimal(token.value))
    {
        return build_record(line, code, first, None, false, false);
    }

    ParseOutcome::NoMatch
}

fn build_record(
    line: LineRecord<'_>,
    code: Token<'_>,
    text: Token<'_>,
    candidate: Option<Token<'_>>,
    aliases: bool,
    multiline_capable: bool,
) -> ParseOutcome {
    let code_text = match copy_text(code.value, "phrase code allocation") {
        Ok(value) => value,
        Err(error) => return ParseOutcome::Invalid(at_token(error, line, code)),
    };
    let code_value = match PhraseCode::new(code_text) {
        Ok(value) => value,
        Err(error) => return ParseOutcome::Invalid(at_token(error, line, code)),
    };
    let candidate_value = match candidate {
        Some(token) => match parse_candidate(token) {
            Ok(value) => Some(value),
            Err(error) => return ParseOutcome::Invalid(at_token(error, line, token)),
        },
        None => None,
    };
    if text.value.is_empty() && !multiline_capable {
        return ParseOutcome::Invalid(at_token(
            CodecError::new(CodecErrorKind::InvalidInput {
                field: "phrase text",
                reason: InvalidInputReason::Empty,
            }),
            line,
            text,
        ));
    }

    let text_value = match copy_text(text.value, "phrase record text allocation") {
        Ok(value) => value,
        Err(error) => return ParseOutcome::Invalid(at_token(error, line, text)),
    };

    ParseOutcome::Matched(ParsedRecord {
        code: code_value,
        text: text_value,
        candidate: candidate_value,
        line: line.number,
        code_column: code.column,
        text_column: text.column,
        aliases,
        multiline_capable,
    })
}

fn parse_candidate(token: Token<'_>) -> Result<Candidate, CodecError> {
    if !is_ascii_decimal(token.value) {
        return Err(malformed(
            "phrase candidate",
            "an integer in 1..=255",
            token.value,
        ));
    }
    let value = token
        .value
        .parse::<u64>()
        .map_err(|_| malformed("phrase candidate", "an integer in 1..=255", token.value))?;
    let value = u8::try_from(value)
        .map_err(|_| malformed("phrase candidate", "an integer in 1..=255", token.value))?;
    Candidate::new(value)
}

fn process_record(
    mut record: ParsedRecord,
    entries: &mut Vec<PhraseEntry>,
    max_candidates: &mut HashMap<String, u8>,
    produced: &mut usize,
    limits: DecodeLimits,
) -> Result<(), CodecError> {
    if record.text.is_empty() {
        return Err(CodecError::new(CodecErrorKind::InvalidInput {
            field: "phrase text",
            reason: InvalidInputReason::Empty,
        })
        .at_text(nonzero(record.line), Some(nonzero(record.text_column))));
    }
    if record.aliases {
        record.text = replace_time_aliases(&record.text)?;
    }

    if record.candidate.is_none()
        && let Some(content) = record
            .text
            .strip_prefix("$[")
            .and_then(|value| value.strip_suffix(']'))
    {
        return expand_array(&record, content, entries, max_candidates, produced, limits);
    }

    let candidate = match record.candidate {
        Some(candidate) => candidate,
        None => {
            let current = max_candidates
                .get(record.code.as_str())
                .copied()
                .unwrap_or(0);
            let next = current.checked_add(1).ok_or_else(|| {
                overflow("phrase automatic candidate")
                    .at_text(nonzero(record.line), Some(nonzero(record.code_column)))
            })?;
            Candidate::new(next).map_err(|error| {
                error.at_text(nonzero(record.line), Some(nonzero(record.code_column)))
            })?
        }
    };
    check_record_budget(*produced, &record, limits)?;
    let candidate_key = prepare_candidate_key(
        max_candidates,
        record.code.as_str(),
        record.line,
        record.code_column,
    )?;
    entries.try_reserve(1).map_err(|_| {
        overflow("phrase entry allocation")
            .at_text(nonzero(record.line), Some(nonzero(record.code_column)))
    })?;
    let text = try_unescape_whitespace(&record.text).map_err(|_| {
        overflow("phrase whitespace unescape allocation")
            .at_text(nonzero(record.line), Some(nonzero(record.text_column)))
    })?;
    let entry = PhraseEntry::new(record.code, text, candidate)
        .map_err(|error| error.at_text(nonzero(record.line), Some(nonzero(record.text_column))))?;
    apply_candidate_update(
        max_candidates,
        candidate_key,
        entry.code().as_str(),
        candidate.get(),
    )?;
    entries.push(entry);
    *produced = checked_increment(*produced, "phrase text output count")?;
    Ok(())
}

fn expand_array(
    record: &ParsedRecord,
    content: &str,
    entries: &mut Vec<PhraseEntry>,
    max_candidates: &mut HashMap<String, u8>,
    produced: &mut usize,
    limits: DecodeLimits,
) -> Result<(), CodecError> {
    let item_count = if content.contains(' ') {
        content.split(' ').filter(|item| !item.is_empty()).count()
    } else {
        content.chars().count()
    };
    if item_count == 0 {
        return Err(CodecError::new(CodecErrorKind::InvalidInput {
            field: "phrase text",
            reason: InvalidInputReason::Empty,
        })
        .at_text(nonzero(record.line), Some(nonzero(record.text_column))));
    }
    let count = u8::try_from(item_count).map_err(|_| {
        overflow("phrase array candidate count")
            .at_text(nonzero(record.line), Some(nonzero(record.text_column)))
    })?;
    let candidate_key = prepare_candidate_key(
        max_candidates,
        record.code.as_str(),
        record.line,
        record.code_column,
    )?;

    if content.contains(' ') {
        for (index, item) in content
            .split(' ')
            .filter(|item| !item.is_empty())
            .enumerate()
        {
            append_array_entry(record, item, index, entries, produced, limits)?;
        }
    } else {
        for (index, (start, character)) in content.char_indices().enumerate() {
            let end = start
                .checked_add(character.len_utf8())
                .ok_or_else(|| overflow("phrase array item boundary"))?;
            append_array_entry(
                record,
                &content[start..end],
                index,
                entries,
                produced,
                limits,
            )?;
        }
    }
    apply_candidate_update(max_candidates, candidate_key, record.code.as_str(), count)?;
    Ok(())
}

fn append_array_entry(
    record: &ParsedRecord,
    item: &str,
    index: usize,
    entries: &mut Vec<PhraseEntry>,
    produced: &mut usize,
    limits: DecodeLimits,
) -> Result<(), CodecError> {
    check_record_budget(*produced, record, limits)?;
    let candidate_value = index
        .checked_add(1)
        .and_then(|value| u8::try_from(value).ok())
        .ok_or_else(|| {
            overflow("phrase array candidate")
                .at_text(nonzero(record.line), Some(nonzero(record.text_column)))
        })?;
    let candidate = Candidate::new(candidate_value)
        .map_err(|error| error.at_text(nonzero(record.line), Some(nonzero(record.text_column))))?;
    let code = copy_text(record.code.as_str(), "phrase array code allocation")
        .map_err(|error| error.at_text(nonzero(record.line), Some(nonzero(record.code_column))))?;
    let code = PhraseCode::new(code)
        .map_err(|error| error.at_text(nonzero(record.line), Some(nonzero(record.code_column))))?;
    entries.try_reserve(1).map_err(|_| {
        overflow("phrase array entry allocation")
            .at_text(nonzero(record.line), Some(nonzero(record.code_column)))
    })?;
    let text = try_unescape_whitespace(item).map_err(|_| {
        overflow("phrase array whitespace unescape allocation")
            .at_text(nonzero(record.line), Some(nonzero(record.text_column)))
    })?;
    let entry = PhraseEntry::new(code, text, candidate)
        .map_err(|error| error.at_text(nonzero(record.line), Some(nonzero(record.text_column))))?;
    entries.push(entry);
    *produced = checked_increment(*produced, "phrase text output count")?;
    Ok(())
}

fn replace_time_aliases(input: &str) -> Result<String, CodecError> {
    let mut output = String::new();
    output
        .try_reserve_exact(input.len())
        .map_err(|_| overflow("phrase time alias output allocation"))?;
    let mut cursor = 0usize;
    while cursor < input.len() {
        if input.as_bytes()[cursor] == b'$' {
            let name_start = cursor + 1;
            let name_len = input[name_start..]
                .bytes()
                .take_while(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
                .count();
            if name_len != 0 {
                let end = name_start + name_len;
                let name = &input[name_start..end];
                if let Some(replacement) = time_alias(name) {
                    append_text(
                        &mut output,
                        replacement,
                        "phrase time alias output allocation",
                    )?;
                    cursor = end;
                    continue;
                }
                append_text(
                    &mut output,
                    &input[cursor..end],
                    "phrase time alias output allocation",
                )?;
                cursor = end;
                continue;
            }
        }
        let character = input[cursor..]
            .chars()
            .next()
            .ok_or_else(|| overflow("phrase time alias cursor"))?;
        let end = cursor
            .checked_add(character.len_utf8())
            .ok_or_else(|| overflow("phrase time alias cursor"))?;
        append_text(
            &mut output,
            &input[cursor..end],
            "phrase time alias output allocation",
        )?;
        cursor = end;
    }
    Ok(output)
}

const fn time_alias(name: &str) -> Option<&'static str> {
    match name.as_bytes() {
        b"year" => Some("%yyyy%"),
        b"year_yy" => Some("%yy%"),
        b"month_mm" => Some("%MM%"),
        b"month" => Some("%M%"),
        b"day" | b"day_dd" => Some("%dd%"),
        b"fullhour" => Some("%HH%"),
        b"minute" => Some("%mm%"),
        b"second" => Some("%ss%"),
        _ => None,
    }
}

fn warning_for_line(line: LineRecord<'_>) -> Result<PhraseTextWarning, CodecError> {
    let preview_end = line
        .original
        .char_indices()
        .nth(MAX_WARNING_PREVIEW_CHARS)
        .map_or(line.original.len(), |(index, _)| index);
    let preview = copy_text(
        &line.original[..preview_end],
        "phrase warning preview allocation",
    )
    .map_err(|error| {
        error.at_text(
            nonzero(line.number),
            Some(first_nonblank_column(line.original)),
        )
    })?;
    Ok(PhraseTextWarning::new(
        PhraseTextWarningKind::UnrecognizedLine,
        SourceLocation::Text {
            line: nonzero(line.number),
            column: Some(first_nonblank_column(line.original)),
        },
        preview,
        preview_end != line.original.len(),
    ))
}

fn prepare_candidate_key(
    max_candidates: &mut HashMap<String, u8>,
    code: &str,
    line: usize,
    column: usize,
) -> Result<Option<String>, CodecError> {
    if max_candidates.contains_key(code) {
        return Ok(None);
    }
    max_candidates.try_reserve(1).map_err(|_| {
        overflow("phrase candidate map allocation").at_text(nonzero(line), Some(nonzero(column)))
    })?;
    copy_text(code, "phrase candidate map key allocation")
        .map(Some)
        .map_err(|error| error.at_text(nonzero(line), Some(nonzero(column))))
}

fn apply_candidate_update(
    max_candidates: &mut HashMap<String, u8>,
    candidate_key: Option<String>,
    code: &str,
    candidate: u8,
) -> Result<(), CodecError> {
    if let Some(value) = max_candidates.get_mut(code) {
        *value = (*value).max(candidate);
        return Ok(());
    }
    let key = candidate_key.ok_or_else(|| overflow("phrase candidate map state"))?;
    max_candidates.insert(key, candidate);
    Ok(())
}

fn copy_text(value: &str, operation: &'static str) -> Result<String, CodecError> {
    let mut output = String::new();
    output
        .try_reserve_exact(value.len())
        .map_err(|_| overflow(operation))?;
    output.push_str(value);
    Ok(output)
}

fn append_text(
    output: &mut String,
    value: &str,
    operation: &'static str,
) -> Result<(), CodecError> {
    output
        .try_reserve(value.len())
        .map_err(|_| overflow(operation))?;
    output.push_str(value);
    Ok(())
}

fn check_budget(
    produced: usize,
    line: LineRecord<'_>,
    limits: DecodeLimits,
) -> Result<(), CodecError> {
    let actual = checked_increment(produced, "phrase text output count")?;
    limits.check_expanded_entries(actual).map_err(|error| {
        error.at_text(
            nonzero(line.number),
            Some(first_nonblank_column(line.cleaned)),
        )
    })
}

fn check_record_budget(
    produced: usize,
    record: &ParsedRecord,
    limits: DecodeLimits,
) -> Result<(), CodecError> {
    let actual = checked_increment(produced, "phrase text output count")?;
    limits
        .check_expanded_entries(actual)
        .map_err(|error| error.at_text(nonzero(record.line), Some(nonzero(record.code_column))))
}

fn checked_increment(value: usize, operation: &'static str) -> Result<usize, CodecError> {
    value.checked_add(1).ok_or_else(|| overflow(operation))
}

struct UnicodeTokens<'a> {
    full_line: &'a str,
    content: &'a str,
    base: usize,
    cursor: usize,
}

impl<'a> UnicodeTokens<'a> {
    const fn new(full_line: &'a str, content: &'a str, base: usize) -> Self {
        Self {
            full_line,
            content,
            base,
            cursor: 0,
        }
    }
}

impl<'a> Iterator for UnicodeTokens<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.cursor < self.content.len() {
            let character = self.content[self.cursor..].chars().next()?;
            if !character.is_whitespace() {
                break;
            }
            self.cursor += character.len_utf8();
        }
        if self.cursor == self.content.len() {
            return None;
        }

        let start = self.cursor;
        while self.cursor < self.content.len() {
            let character = self.content[self.cursor..].chars().next()?;
            if character.is_whitespace() {
                break;
            }
            self.cursor += character.len_utf8();
        }
        let byte_start = self.base + start;
        Some(Token {
            value: &self.content[start..self.cursor],
            column: column_at(self.full_line, byte_start),
            byte_start,
            byte_end: self.base + self.cursor,
        })
    }
}

fn token<'a>(line: &'a str, value: &'a str, byte_start: usize) -> Token<'a> {
    Token {
        value,
        column: column_at(line, byte_start),
        byte_start,
        byte_end: byte_start + value.len(),
    }
}

fn at_token(error: CodecError, line: LineRecord<'_>, token: Token<'_>) -> CodecError {
    error.at_text(nonzero(line.number), Some(nonzero(token.column)))
}

fn trim_with_offset(value: &str, base: usize) -> (&str, usize) {
    let trimmed_start = value.trim_start_matches(char::is_whitespace);
    let removed = value.len() - trimmed_start.len();
    (
        trimmed_start.trim_end_matches(char::is_whitespace),
        base + removed,
    )
}

fn is_numeric_shape(value: &str) -> bool {
    is_ascii_decimal(value) || value.strip_prefix(['-', '+']).is_some_and(is_ascii_decimal)
}

fn is_ascii_decimal(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
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

fn p6_code_token(token: Token<'_>) -> Option<Token<'_>> {
    let code_len = token
        .value
        .bytes()
        .take_while(u8::is_ascii_lowercase)
        .take(4)
        .count();
    if code_len == 0
        || !token.value[code_len..]
            .bytes()
            .all(|byte| byte.is_ascii_digit())
    {
        return None;
    }

    Some(Token {
        value: &token.value[..code_len],
        byte_end: token.byte_start + code_len,
        ..token
    })
}

fn column_at(content: &str, byte_offset: usize) -> usize {
    content[..byte_offset].chars().count() + 1
}

fn first_nonblank_column(content: &str) -> NonZeroUsize {
    nonzero(
        content
            .chars()
            .take_while(|character| character.is_whitespace())
            .count()
            + 1,
    )
}

fn nonzero(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).unwrap_or(NonZeroUsize::MIN)
}

fn malformed(field: &'static str, expected: &str, actual: &str) -> CodecError {
    CodecError::new(CodecErrorKind::MalformedField {
        field,
        expected: FieldValue::Text(expected.to_owned()),
        actual: FieldValue::Text(actual.to_owned()),
    })
}

fn overflow(operation: &'static str) -> CodecError {
    CodecError::new(CodecErrorKind::IntegerOverflow { operation })
}
