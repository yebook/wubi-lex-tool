use std::{cmp::Ordering, collections::HashMap};

use crate::{CodecError, CodecErrorKind, LexiconDocument, LexiconEntry, escape::escape_whitespace};

use super::LexiconTextFormat;

struct ProjectedEntry<'a> {
    entry: &'a LexiconEntry,
    effective_weight: u16,
}

/// Formats a lexicon document using one deterministic community text layout.
pub fn format(document: &LexiconDocument, format: LexiconTextFormat) -> Result<String, CodecError> {
    let projection = canonical_projection(document)?;
    match format {
        LexiconTextFormat::CodeThenText => format_code_then_text(&projection),
        LexiconTextFormat::CodeThenTexts => format_code_then_texts(&projection),
        LexiconTextFormat::CodeThenTextWeight => format_code_then_text_weight(&projection),
        LexiconTextFormat::TextThenCode => format_text_then_code(&projection),
        LexiconTextFormat::TextThenCodes => format_text_then_codes(&projection),
        LexiconTextFormat::TextThenCodeDescendingWeight => {
            format_text_then_code_descending_weight(&projection)
        }
        LexiconTextFormat::PhraseAscendingCandidate => format_phrase_candidates(&projection),
    }
}

/// Formats a lexicon document as BOM-prefixed UTF-16LE bytes.
pub fn encode_utf16le(
    document: &LexiconDocument,
    format: LexiconTextFormat,
) -> Result<Vec<u8>, CodecError> {
    let formatted = self::format(document, format)?;
    let unit_count = formatted.encode_utf16().count();
    let byte_count = unit_count
        .checked_mul(2)
        .and_then(|value| value.checked_add(2))
        .ok_or_else(|| overflow("UTF-16LE output size"))?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(byte_count)
        .map_err(|_| overflow("UTF-16LE output allocation"))?;
    output.extend_from_slice(&[0xFF, 0xFE]);
    for unit in formatted.encode_utf16() {
        output.extend_from_slice(&unit.to_le_bytes());
    }
    Ok(output)
}

fn canonical_projection(document: &LexiconDocument) -> Result<Vec<ProjectedEntry<'_>>, CodecError> {
    let mut previous_by_code = HashMap::<&str, u16>::new();
    let mut projection = Vec::with_capacity(document.len());

    for entry in document.entries() {
        let previous = previous_by_code.entry(entry.code().as_str()).or_insert(0);
        let effective_weight = match entry.weight() {
            Some(weight) => weight.get(),
            None => previous
                .checked_add(1)
                .ok_or_else(|| overflow("lexicon effective weight increment"))?,
        };
        *previous = effective_weight;
        projection.push(ProjectedEntry {
            entry,
            effective_weight,
        });
    }

    projection.sort_by(|left, right| {
        let code_order = left.entry.code().as_str().cmp(right.entry.code().as_str());
        if code_order == Ordering::Equal {
            left.effective_weight.cmp(&right.effective_weight)
        } else {
            code_order
        }
    });
    Ok(projection)
}

fn format_code_then_text(entries: &[ProjectedEntry<'_>]) -> Result<String, CodecError> {
    let mut output = String::new();
    for projected in entries {
        push_fields(
            &mut output,
            &[
                projected.entry.code().as_str(),
                &escape_whitespace(projected.entry.text()),
            ],
            "\t",
        );
    }
    Ok(output)
}

fn format_code_then_texts(entries: &[ProjectedEntry<'_>]) -> Result<String, CodecError> {
    let mut output = String::new();
    let mut index = 0;
    while index < entries.len() {
        let code = entries[index].entry.code().as_str();
        output.push_str(code);
        let mut previous_text = None;
        while index < entries.len() && entries[index].entry.code().as_str() == code {
            let text = entries[index].entry.text();
            if previous_text != Some(text) {
                output.push('\t');
                output.push_str(&escape_whitespace(text));
                previous_text = Some(text);
            }
            index += 1;
        }
        output.push_str("\r\n");
    }
    Ok(output)
}

fn format_code_then_text_weight(entries: &[ProjectedEntry<'_>]) -> Result<String, CodecError> {
    let mut output = String::new();
    for projected in entries {
        push_fields(
            &mut output,
            &[
                projected.entry.code().as_str(),
                &escape_whitespace(projected.entry.text()),
                &projected.effective_weight.to_string(),
            ],
            "\t",
        );
    }
    Ok(output)
}

fn format_text_then_code(entries: &[ProjectedEntry<'_>]) -> Result<String, CodecError> {
    let mut output = String::new();
    for projected in entries {
        push_fields(
            &mut output,
            &[
                &escape_whitespace(projected.entry.text()),
                projected.entry.code().as_str(),
            ],
            "\t",
        );
    }
    Ok(output)
}

fn format_text_then_codes(entries: &[ProjectedEntry<'_>]) -> Result<String, CodecError> {
    struct TextGroup<'a> {
        text: &'a str,
        codes: Vec<&'a str>,
    }

    let mut group_indexes = HashMap::<&str, usize>::new();
    let mut groups: Vec<TextGroup<'_>> = Vec::new();
    for projected in entries {
        let text = projected.entry.text();
        let code = projected.entry.code().as_str();
        if let Some(index) = group_indexes.get(text).copied() {
            let group = &mut groups[index];
            if group.codes.last().copied() != Some(code) {
                group.codes.push(code);
            }
        } else {
            group_indexes.insert(text, groups.len());
            groups.push(TextGroup {
                text,
                codes: vec![code],
            });
        }
    }

    let mut output = String::new();
    for group in groups {
        output.push_str(&escape_whitespace(group.text));
        output.push('\t');
        for (index, code) in group.codes.iter().enumerate() {
            if index > 0 {
                output.push(' ');
            }
            output.push_str(code);
        }
        output.push_str("\r\n");
    }
    Ok(output)
}

fn format_text_then_code_descending_weight(
    entries: &[ProjectedEntry<'_>],
) -> Result<String, CodecError> {
    let mut output = String::new();
    for projected in entries {
        let descending = u16::MAX - projected.effective_weight;
        push_fields(
            &mut output,
            &[
                &escape_whitespace(projected.entry.text()),
                projected.entry.code().as_str(),
                &descending.to_string(),
            ],
            "\t",
        );
    }
    Ok(output)
}

fn format_phrase_candidates(entries: &[ProjectedEntry<'_>]) -> Result<String, CodecError> {
    let mut output = String::new();
    let mut previous_code = None;
    let mut candidate = 0u16;
    for projected in entries {
        let code = projected.entry.code().as_str();
        if previous_code == Some(code) {
            candidate = candidate
                .checked_add(1)
                .ok_or_else(|| overflow("phrase candidate index"))?;
        } else {
            previous_code = Some(code);
            candidate = 1;
        }
        output.push_str(code);
        output.push('=');
        output.push_str(&candidate.to_string());
        output.push(',');
        output.push_str(&escape_whitespace(projected.entry.text()));
        output.push_str("\r\n");
    }
    Ok(output)
}

fn push_fields(output: &mut String, fields: &[&str], separator: &str) {
    for (index, field) in fields.iter().enumerate() {
        if index > 0 {
            output.push_str(separator);
        }
        output.push_str(field);
    }
    output.push_str("\r\n");
}

fn overflow(operation: &'static str) -> CodecError {
    CodecError::new(CodecErrorKind::IntegerOverflow { operation })
}
