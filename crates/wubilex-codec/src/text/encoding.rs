use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use encoding_rs::{DecoderResult, Encoding, GBK, UTF_8, UTF_16BE, UTF_16LE};

use crate::{CodecError, CodecErrorKind, DecodeLimits, DetectedTextEncoding, TextEncoding};

pub(super) fn decode_bytes(
    input: &[u8],
    limits: DecodeLimits,
) -> Result<(String, DetectedTextEncoding), CodecError> {
    limits.check_input_bytes(input.len())?;

    let (encoding, has_bom, bom_len) = select_encoding(input);
    let body = &input[bom_len..];
    let text = if encoding == TextEncoding::Utf8 {
        decode_utf8(body, bom_len)?
    } else {
        decode_legacy(body, encoding, bom_len)?
    };

    Ok((text, DetectedTextEncoding::new(encoding, has_bom)))
}

fn select_encoding(input: &[u8]) -> (TextEncoding, bool, usize) {
    if input.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return (TextEncoding::Utf8, true, 3);
    }
    if input.starts_with(&[0xFF, 0xFE]) {
        return (TextEncoding::Utf16Le, true, 2);
    }
    if input.starts_with(&[0xFE, 0xFF]) {
        return (TextEncoding::Utf16Be, true, 2);
    }
    if std::str::from_utf8(input).is_ok() {
        return (TextEncoding::Utf8, false, 0);
    }

    let mut detector = EncodingDetector::new(Iso2022JpDetection::Deny);
    detector.feed(input, true);
    let detected = detector.guess(Some(b"cn"), Utf8Detection::Allow);
    let encoding = if detected == UTF_8 {
        TextEncoding::Utf8
    } else {
        TextEncoding::Gbk
    };
    (encoding, false, 0)
}

fn decode_utf8(input: &[u8], prefix_len: usize) -> Result<String, CodecError> {
    std::str::from_utf8(input)
        .map(str::to_owned)
        .map_err(|error| invalid_encoding(TextEncoding::Utf8, prefix_len + error.valid_up_to()))
}

fn decode_legacy(
    input: &[u8],
    encoding: TextEncoding,
    prefix_len: usize,
) -> Result<String, CodecError> {
    let selected = encoding_rs_encoding(encoding);
    let mut decoder = selected.new_decoder_without_bom_handling();
    let capacity = decoder
        .max_utf8_buffer_length_without_replacement(input.len())
        .ok_or_else(|| overflow("text decode UTF-8 capacity"))?;
    let mut output = String::new();
    output
        .try_reserve_exact(capacity)
        .map_err(|_| overflow("text decode UTF-8 allocation"))?;

    let (result, read) = decoder.decode_to_string_without_replacement(input, &mut output, true);
    match result {
        DecoderResult::InputEmpty => Ok(output),
        DecoderResult::OutputFull => Err(overflow("text decode output capacity")),
        DecoderResult::Malformed(malformed_len, after_len) => {
            let consumed_after_start = usize::from(malformed_len) + usize::from(after_len);
            let local_offset = read
                .checked_sub(consumed_after_start)
                .ok_or_else(|| overflow("malformed text byte offset"))?;
            Err(invalid_encoding(encoding, prefix_len + local_offset))
        }
    }
}

fn encoding_rs_encoding(encoding: TextEncoding) -> &'static Encoding {
    match encoding {
        TextEncoding::Utf8 => UTF_8,
        TextEncoding::Utf16Le => UTF_16LE,
        TextEncoding::Utf16Be => UTF_16BE,
        TextEncoding::Gbk => GBK,
    }
}

fn invalid_encoding(encoding: TextEncoding, offset: usize) -> CodecError {
    CodecError::new(CodecErrorKind::InvalidTextEncoding { encoding }).at_byte_offset(offset as u64)
}

fn overflow(operation: &'static str) -> CodecError {
    CodecError::new(CodecErrorKind::IntegerOverflow { operation })
}
