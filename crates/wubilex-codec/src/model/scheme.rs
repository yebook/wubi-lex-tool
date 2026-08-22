//! Lexicon scheme identities.

/// A supported lexicon scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LexScheme {
    /// Microsoft Wubi 86.
    Wubi86,
    /// Microsoft Wubi 98.
    Wubi98,
    /// Wubi New Century (06).
    Wubi06,
    /// Wubi 091.
    Wubi091,
    /// Wubi 092.
    Wubi092,
    /// Zhengma, optionally identified as its dedicated word-formation table.
    Zhengma {
        /// Whether this is a Zhengma formation table rather than a normal lexicon.
        formation: bool,
    },
    /// Xiaohe sound-shape.
    XiaoheSoundShape,
    /// Biaoxingma.
    Biaoxingma,
}
