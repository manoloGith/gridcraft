//! Android (DexGridCraft): the system's fonts live in `/system/fonts` and there is no
//! fontconfig, so each role and script fallback also looks for the faces Android ships there.
//! Only built for Android; on every other system nothing changes.

const DIR: &str = "/system/fonts";

fn faces(files: &[(&str, u32)]) -> Vec<(String, u32)> {
    files.iter().map(|(f, i)| (format!("{DIR}/{f}"), *i)).collect()
}

/// More candidates for a role of `font_setup::candidates`, after the desktop ones.
pub(super) fn candidates(role: &str) -> Vec<(String, u32)> {
    match role {
        "ui" | "cell" | "cell-italic" => faces(&[("Roboto-Regular.ttf", 0), ("RobotoFlex-Regular.ttf", 0), ("NotoSans-Regular.ttf", 0), ("DroidSans.ttf", 0)]),
        "ui-bold" | "cell-bold" | "cell-bold-italic" => faces(&[("Roboto-Bold.ttf", 0), ("Roboto-Regular.ttf", 0), ("NotoSans-Bold.ttf", 0), ("DroidSans-Bold.ttf", 0)]),
        "serif" => faces(&[("NotoSerif-Regular.ttf", 0), ("DroidSerif-Regular.ttf", 0)]),
        "math" | "math-italic" | "math-bold" => faces(&[("NotoSansMath-Regular.ttf", 0), ("NotoSerif-Regular.ttf", 0)]),
        "mono" => faces(&[("DroidSansMono.ttf", 0), ("CutiveMono.ttf", 0)]),
        _ => Vec::new(),
    }
}

/// More candidates for a script fallback of `font_fallback::fallback_faces`, by its key.
pub(super) fn fallback(key: &str) -> Vec<(String, u32)> {
    match key {
        // NotoSansCJK-Regular.ttc: 0 Japanese, 1 Korean, 2 Simplified Chinese, 3 Traditional.
        "sys-cjk-han-ja" => faces(&[("NotoSansCJK-Regular.ttc", 0)]),
        "sys-cjk-han-zh" => faces(&[("NotoSansCJK-Regular.ttc", 2)]),
        "sys-cjk-hangul" => faces(&[("NotoSansCJK-Regular.ttc", 1)]),
        "sys-thai" => faces(&[("NotoSansThai-Regular.ttf", 0), ("NotoSansThaiUI-Regular.ttf", 0)]),
        "sys-lao" => faces(&[("NotoSansLao-Regular.ttf", 0), ("NotoSansLaoUI-Regular.ttf", 0)]),
        "sys-hebrew" => faces(&[("NotoSansHebrew-Regular.ttf", 0)]),
        "sys-arabic" => faces(&[("NotoNaskhArabic-Regular.ttf", 0), ("NotoNaskhArabicUI-Regular.ttf", 0), ("NotoSansArabic-Regular.ttf", 0)]),
        "sys-symbols" => faces(&[("NotoSansSymbols-Regular-Subsetted.ttf", 0), ("NotoSansSymbols-Regular.ttf", 0)]),
        "sys-symbols-2" => faces(&[("NotoSansSymbols-Regular-Subsetted2.ttf", 0), ("NotoSansSymbols2-Regular.ttf", 0)]),
        _ => Vec::new(),
    }
}
