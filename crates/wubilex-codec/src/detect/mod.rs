//! Pure content-based codec detection helpers.

use crate::{LexScheme, LexiconDocument};

/// Detects one of the eight supported lexicon schemes from feature entries.
#[must_use]
pub fn scheme(document: &LexiconDocument) -> LexScheme {
    let mut features = Features::default();
    for entry in document.entries() {
        features.observe(entry.code().as_str(), entry.text());
    }

    if features.zhengma_q_month && features.zhengma_e_world {
        return LexScheme::Zhengma { formation: false };
    }
    if features.xiaohe_aakk && features.xiaohe_hedn {
        return LexScheme::XiaoheSoundShape;
    }
    if features.zhengma_qv_month && features.zhengma_ev_world {
        return LexScheme::Zhengma { formation: true };
    }
    if features.wubi092_sr && features.wubi092_ks && features.wubi092_ms {
        return LexScheme::Wubi092;
    }
    if features.biaoxingma_hodd && features.biaoxingma_opto {
        return LexScheme::Biaoxingma;
    }

    let mut scores = Scores::default();
    scores.exclusive(features.teb, SchemeIndex::Wubi98);
    scores.exclusive(features.othc, SchemeIndex::Wubi98);
    scores.exclusive(features.tuwb, SchemeIndex::Wubi98);
    scores.exclusive(features.uqwn, SchemeIndex::Wubi86);
    scores.shared_98_06(features.khdy);
    scores.exclusive(features.xfxy, SchemeIndex::Wubi06);
    scores.exclusive(features.ks, SchemeIndex::Wubi091);
    scores.exclusive(features.lm, SchemeIndex::Wubi091);
    scores.exclusive(features.ms, SchemeIndex::Wubi091);
    scores.winner()
}

#[derive(Default)]
struct Features {
    zhengma_q_month: bool,
    zhengma_e_world: bool,
    xiaohe_aakk: bool,
    xiaohe_hedn: bool,
    zhengma_qv_month: bool,
    zhengma_ev_world: bool,
    wubi092_sr: bool,
    wubi092_ks: bool,
    wubi092_ms: bool,
    biaoxingma_hodd: bool,
    biaoxingma_opto: bool,
    teb: bool,
    othc: bool,
    tuwb: bool,
    uqwn: bool,
    khdy: bool,
    xfxy: bool,
    ks: bool,
    lm: bool,
    ms: bool,
}

impl Features {
    fn observe(&mut self, code: &str, text: &str) {
        match (code, text) {
            ("q", "月") => self.zhengma_q_month = true,
            ("e", "世") => self.zhengma_e_world = true,
            ("aakk", "啊") => self.xiaohe_aakk = true,
            ("hedn", "鹤") => self.xiaohe_hedn = true,
            ("qv", "月") => self.zhengma_qv_month = true,
            ("ev", "世") => self.zhengma_ev_world = true,
            ("sr", "版") => self.wubi092_sr = true,
            ("ks", "吃") => self.wubi092_ks = true,
            ("ms", "见") => self.wubi092_ms = true,
            ("hodd", "够") => self.biaoxingma_hodd = true,
            ("opto", "啊") => self.biaoxingma_opto = true,
            ("teb", "笔" | "筆") => self.teb = true,
            ("othc", "煅") => self.othc = true,
            ("tuwb", "舱" | "艙") => self.tuwb = true,
            ("uqwn", "瓷") => self.uqwn = true,
            ("khdy", "跋") => self.khdy = true,
            ("xfxy", "线" | "線") => self.xfxy = true,
            ("ks", "整") => self.ks = true,
            ("lm", "示") => self.lm = true,
            ("ms", "刺") => self.ms = true,
            _ => {}
        }
    }
}

#[derive(Clone, Copy)]
enum SchemeIndex {
    Wubi86 = 0,
    Wubi98 = 1,
    Wubi06 = 2,
    Wubi091 = 3,
}

#[derive(Default)]
struct Scores([i8; 4]);

impl Scores {
    fn exclusive(&mut self, hit: bool, expected: SchemeIndex) {
        for (index, score) in self.0.iter_mut().enumerate() {
            let is_expected = index == expected as usize;
            *score += if hit == is_expected { 1 } else { -1 };
        }
    }

    fn shared_98_06(&mut self, hit: bool) {
        for (index, score) in self.0.iter_mut().enumerate() {
            let is_expected = matches!(index, 1 | 2);
            *score += if hit == is_expected { 1 } else { -1 };
        }
    }

    fn winner(&self) -> LexScheme {
        if strictly_greatest(&self.0, SchemeIndex::Wubi98) {
            LexScheme::Wubi98
        } else if strictly_greatest(&self.0, SchemeIndex::Wubi06) {
            LexScheme::Wubi06
        } else if strictly_greatest(&self.0, SchemeIndex::Wubi091) {
            LexScheme::Wubi091
        } else {
            LexScheme::Wubi86
        }
    }
}

fn strictly_greatest(scores: &[i8; 4], candidate: SchemeIndex) -> bool {
    let index = candidate as usize;
    scores
        .iter()
        .enumerate()
        .all(|(other, score)| other == index || scores[index] > *score)
}
