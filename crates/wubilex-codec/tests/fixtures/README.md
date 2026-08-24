# Real codec fixtures

Run `cargo xtask fixtures` from anywhere inside the repository to download and
verify the eight real `.lex` fixtures. Run `cargo xtask fixtures --check` for a
strict offline integrity check. The downloaded `.lex` and `.lex.lzma` files are
ignored by Git; `manifest.json` is the source of truth for their URLs, sizes,
hashes, and expected scheme detection.

The files come from the legacy WubiLex online catalog at
`https://wubi.aardio.com`. The `aardio/wubi-lex` source repository is MIT
licensed, but the catalog does not publish a separate license for each third
party dictionary. These files are downloaded on demand for compatibility tests
and are not redistributed by this repository. Product redistribution requires
a separate license review.

The upstream URLs are not content-addressed. A size or digest mismatch means
the source changed and requires human review; do not update hashes solely to
make the command pass. The root `resource/` directory is local user data and is
never a fixture source or fallback.
