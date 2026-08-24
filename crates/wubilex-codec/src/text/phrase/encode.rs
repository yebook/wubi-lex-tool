use std::cmp::Ordering;

use crate::{
    CodecError, CodecErrorKind, PhraseDocument, PhraseEntry, escape::try_escape_whitespace,
};

/// Formats a phrase document using deterministic candidate detail or array lines.
pub fn format(document: &PhraseDocument) -> Result<String, CodecError> {
    let mut entries = Vec::<(usize, &PhraseEntry)>::new();
    entries
        .try_reserve_exact(document.len())
        .map_err(|_| overflow("phrase canonical projection allocation"))?;
    entries.extend(document.entries().iter().enumerate());
    entries.sort_unstable_by(|(left_index, left), (right_index, right)| {
        let code_order = left.code().as_str().cmp(right.code().as_str());
        if code_order == Ordering::Equal {
            left.candidate()
                .get()
                .cmp(&right.candidate().get())
                .then_with(|| left_index.cmp(right_index))
        } else {
            code_order
        }
    });

    let mut output = String::new();
    let mut start = 0usize;
    while start < entries.len() {
        let code = entries[start].1.code().as_str();
        let mut end = start + 1;
        while end < entries.len() && entries[end].1.code().as_str() == code {
            end += 1;
        }
        let group = &entries[start..end];
        if can_compress(group) {
            append(&mut output, code)?;
            append(&mut output, "\t$[")?;
            for (index, (_, entry)) in group.iter().enumerate() {
                if index != 0 {
                    append(&mut output, " ")?;
                }
                let escaped = try_escape_whitespace(entry.text())
                    .map_err(|_| overflow("phrase whitespace escape allocation"))?;
                append(&mut output, &escaped)?;
            }
            append(&mut output, "]\r\n")?;
        } else {
            for (_, entry) in group {
                append(&mut output, entry.code().as_str())?;
                append(&mut output, "\t")?;
                let escaped = try_escape_whitespace(entry.text())
                    .map_err(|_| overflow("phrase whitespace escape allocation"))?;
                append(&mut output, &escaped)?;
                append(&mut output, "\t")?;
                append_candidate(&mut output, entry.candidate().get())?;
                append(&mut output, "\r\n")?;
            }
        }
        start = end;
    }
    Ok(output)
}

fn can_compress(entries: &[(usize, &PhraseEntry)]) -> bool {
    entries.len() > 1
        && entries.iter().enumerate().all(|(index, (_, entry))| {
            index.checked_add(1).is_some_and(|candidate| {
                usize::from(entry.candidate().get()) == candidate && entry.utf16_len() <= 2
            })
        })
}

fn append_candidate(output: &mut String, candidate: u8) -> Result<(), CodecError> {
    let digits = if candidate >= 100 {
        3
    } else if candidate >= 10 {
        2
    } else {
        1
    };
    output
        .try_reserve(digits)
        .map_err(|_| overflow("phrase text output allocation"))?;
    if candidate >= 100 {
        output.push(char::from(b'0' + candidate / 100));
    }
    if candidate >= 10 {
        output.push(char::from(b'0' + (candidate / 10) % 10));
    }
    output.push(char::from(b'0' + candidate % 10));
    Ok(())
}

fn append(output: &mut String, value: &str) -> Result<(), CodecError> {
    output
        .try_reserve(value.len())
        .map_err(|_| overflow("phrase text output allocation"))?;
    output.push_str(value);
    Ok(())
}

fn overflow(operation: &'static str) -> CodecError {
    CodecError::new(CodecErrorKind::IntegerOverflow { operation })
}
