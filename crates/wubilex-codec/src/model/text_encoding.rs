//! Text encoding detection result contracts.

/// A supported text lexicon encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextEncoding {
    /// UTF-8.
    Utf8,
    /// Little-endian UTF-16.
    Utf16Le,
    /// Big-endian UTF-16.
    Utf16Be,
    /// GBK, used as the legacy simplified-Chinese ANSI encoding.
    Gbk,
}

/// A detected text encoding and whether the input carried a byte-order mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DetectedTextEncoding {
    encoding: TextEncoding,
    has_bom: bool,
}

impl DetectedTextEncoding {
    /// Creates a detection result without performing detection.
    #[must_use]
    pub const fn new(encoding: TextEncoding, has_bom: bool) -> Self {
        Self { encoding, has_bom }
    }

    /// Returns the detected encoding.
    #[must_use]
    pub const fn encoding(self) -> TextEncoding {
        self.encoding
    }

    /// Returns whether the original input carried a byte-order mark.
    #[must_use]
    pub const fn has_bom(self) -> bool {
        self.has_bom
    }
}
