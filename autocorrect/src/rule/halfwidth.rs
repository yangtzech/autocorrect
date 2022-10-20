// autocorrect: false
use regex::Regex;
use std::collections::HashMap;

use super::CJK_RE;

#[derive(Clone)]
enum ReplaceMode {
    Replace,
    PrefixSpace,
    SuffixSpace,
}

#[derive(Clone, PartialEq)]
enum CharType {
    LeftQuote,
    RightQuote,
    Other,
}

#[derive(Clone)]
struct ReplaceRule {
    to: &'static str,
    mode: ReplaceMode,
    char_type: CharType,
}

impl ReplaceRule {
    fn new(to: &'static str) -> Self {
        Self {
            to,
            mode: ReplaceMode::Replace,
            char_type: CharType::Other,
        }
    }

    fn with_suffix_space(&mut self) -> Self {
        self.mode = ReplaceMode::SuffixSpace;
        self.clone()
    }

    fn with_prefix_space(&mut self) -> Self {
        self.mode = ReplaceMode::PrefixSpace;
        self.clone()
    }

    fn left_quote(&mut self) -> Self {
        self.char_type = CharType::LeftQuote;
        self.clone()
    }

    fn right_quote(&mut self) -> Self {
        self.char_type = CharType::RightQuote;
        self.clone()
    }
}

lazy_static! {
    static ref CHAR_WIDTH_MAP: HashMap<&'static str, &'static str> = map!(
      "ａ" => "a", "ｂ" => "b", "ｃ" => "c", "ｄ" => "d", "ｅ" => "e", "ｆ" => "f", "ｇ" => "g", "ｈ" => "h", "ｉ" => "i", "ｊ" => "j", "ｋ" => "k", "ｌ" => "l", "ｍ" => "m", "ｎ" => "n", "ｏ" => "o", "ｐ" => "p", "ｑ" => "q", "ｒ" => "r", "ｓ" => "s", "ｔ" => "t", "ｕ" => "u", "ｖ" => "v", "ｗ" => "w", "ｘ" => "x", "ｙ" => "y", "ｚ" => "z", "Ａ" => "A", "Ｂ" => "B", "Ｃ" => "C", "Ｄ" => "D", "Ｅ" => "E", "Ｆ" => "F", "Ｇ" => "G", "Ｈ" => "H", "Ｉ" => "I", "Ｊ" => "J", "Ｋ" => "K", "Ｌ" => "L", "Ｍ" => "M", "Ｎ" => "N", "Ｏ" => "O", "Ｐ" => "P", "Ｑ" => "Q", "Ｒ" => "R", "Ｓ" => "S", "Ｔ" => "T", "Ｕ" => "U", "Ｖ" => "V", "Ｗ" => "W", "Ｘ" => "X", "Ｙ" => "Y", "Ｚ" => "Z", "１" => "1", "２" => "2", "３" => "3", "４" => "4", "５" => "5", "６" => "6", "７" => "7", "８" => "8", "９" => "9", "０" => "0", "　" => " ",
    );

    static ref HALF_TIME_RE: Regex = regexp!("{}", r"(\d)(：)(\d)");
    // More than 2 words and leading with words
    static ref ENGLISH_RE: Regex = regexp!("{}", r#"([\w]+[ ,.'?!&:]+[\w]+)"#);
    static ref START_WITH_WORD_RE: Regex = regexp!("{}", r#"^\s*[\w]+"#);
    static ref QUOTE_RE: Regex = regexp!("{}", r#"^\s*(["'`]).+(["'`])\s*$"#);

    static ref PUNCTUATION_MAP: HashMap<&'static str, ReplaceRule> = map!(
        "‘" => ReplaceRule::new("'").left_quote(),
        "’" => ReplaceRule::new("'").right_quote(),

        "，" => ReplaceRule::new(",").with_suffix_space(),
        "、" => ReplaceRule::new(",").with_suffix_space(),
        "。" => ReplaceRule::new(".").with_suffix_space(),
        "：" => ReplaceRule::new(":").with_suffix_space(),
        "；" => ReplaceRule::new(".").with_suffix_space(),
        "！" => ReplaceRule::new("!").with_suffix_space(),
        "？" => ReplaceRule::new("?").with_suffix_space(),

        // Quotes prefix
        "“" => ReplaceRule::new(r#"""#).left_quote().with_prefix_space(),
        "（" => ReplaceRule::new("(").left_quote().with_prefix_space(),
        "【" => ReplaceRule::new("[").left_quote().with_prefix_space(),
        "「" => ReplaceRule::new("[").left_quote().with_prefix_space(),
        "《" => ReplaceRule::new(r#"""#).left_quote().with_prefix_space(),

        // Quotes suffix
        "”" => ReplaceRule::new(r#"""#).right_quote().with_suffix_space(),
        "）" => ReplaceRule::new(")").right_quote().with_suffix_space(),
        "】" => ReplaceRule::new("]").right_quote().with_suffix_space(),
        "」" => ReplaceRule::new("]").right_quote().with_suffix_space(),
        "》" => ReplaceRule::new(r#"""#).right_quote().with_suffix_space(),
    );
}

trait CharMatching {
    fn is_ascii_alphanumeric_punctuation(&self) -> bool;
    fn is_alphanumeric_or_space(&self) -> bool;
}

impl CharMatching for char {
    /// Match is a-z, A-Z, 0-9, all ASCII punctuations
    fn is_ascii_alphanumeric_punctuation(&self) -> bool {
        self.is_ascii_alphanumeric() || self.is_ascii_punctuation()
    }

    fn is_alphanumeric_or_space(&self) -> bool {
        self.is_ascii_alphanumeric() || self.eq(&' ') || self.eq(&'\t')
    }
}

pub fn format(text: &str) -> String {
    let mut out = String::from("");
    for line in text.split_inclusive('\n') {
        out.push_str(&format_line(line));
    }

    out
}

fn is_may_only_english(text: &str) -> bool {
    if CJK_RE.is_match(text) {
        return false;
    }

    // Characters which pass CHAR_WIDTH_MAP replacement
    if ENGLISH_RE.is_match(text) && START_WITH_WORD_RE.is_match(text) {
        // Maybe English, pass
        return true;
    }

    if QUOTE_RE.is_match(text) {
        return true;
    }

    false
}

fn format_line(text: &str) -> String {
    let is_english = is_may_only_english(text);
    let mut out = String::new();

    let mut parts = text.split("").peekable();
    while let Some(part) = parts.next() {
        let next_part = parts.peek().unwrap_or(&"");
        let last_part = out.chars().last().unwrap_or(' ');

        if is_english {
            // Remove duplicate space without CJK contents
            // if part.ends_with(|s: char| s.is_whitespace())
            //     && !next_part.starts_with(|s: char| s.is_ascii_alphanumeric_punctuation())
            // {
            //     part = "";
            // }

            // Fix punctuation without CJK contents
            if let Some(rule) = PUNCTUATION_MAP.get(part) {
                // Do not change left quote when is last char.
                if rule.char_type == CharType::LeftQuote && next_part.is_empty() {
                    out.push_str(part);
                    continue;
                }

                match rule.mode {
                    ReplaceMode::SuffixSpace => {
                        out.push_str(rule.to);
                        if next_part.starts_with(|s: char| s.is_alphanumeric()) {
                            out.push(' ');
                        }
                    }
                    ReplaceMode::PrefixSpace => {
                        if last_part.is_alphanumeric() {
                            out.push(' ');
                        }
                        out.push_str(rule.to);
                    }
                    ReplaceMode::Replace => {
                        out.push_str(rule.to);
                    }
                }
                continue;
            }
        }

        if let Some(new_str) = CHAR_WIDTH_MAP.get(part) {
            out.push_str(new_str);
            continue;
        }

        out.push_str(part);
    }

    // Fix 12：00 -> 12:00
    out = HALF_TIME_RE
        .replace_all(&out, |cap: &regex::Captures| cap[0].replace('：', ":"))
        .to_string();

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_cases(cases: HashMap<&str, &str>) {
        for (source, exptected) in cases.into_iter() {
            let actual = format(source);
            assert_eq!(exptected, actual);
        }
    }

    #[test]
    fn test_halfwidth_alphabetic_numbers() {
        let source = "测试:ａｂｃｄｅｆｇｈｉｊｋｌｍｎｏｐｑｒｓｔｕｖｗｘｙｚＡＢＣＤＥＦＧＨＩＪＫＬＭＮＯＰＱＲＳＴＵＶＷＸＹＺ１２３４５６７８９０";
        assert_eq!(
            "测试:abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890",
            format(source)
        );

        assert_eq!(
            "他说：我们将在16:32分出发去CBD中心。",
            format("他说：我们将在１６：３２分出发去ＣＢＤ中心。")
        );

        // Fullwidth space
        assert_eq!(
            "ジョイフル－後場売り気配 200 店舗を閉鎖へ 7 月以降、不採算店中心に",
            format("ジョイフル－後場売り気配　200 店舗を閉鎖へ　7 月以降、不採算店中心に")
        );
    }

    #[test]
    fn test_halfwidth_punctuation_ignores() {
        let cases = map! [
            "。" => "。",
            "，" => "，",
            "SHA1。" => "SHA1。",
            "a。" => "a。",
            "foo-bar-dar。" => "foo-bar-dar。",
            "hello)。" => "hello)。",
            "说：你好 english。" => "说：你好 english。",
            "‘腾讯’ - 发布 - ‘新版’本微信" => "‘腾讯’ - 发布 - ‘新版’本微信",
            "${item.name}（ID ${item.id}）" => "${item.name}（ID ${item.id}）",
            "{{ t('name') }}：{{ item.extraKeys.join(' | ') }}" => "{{ t('name') }}：{{ item.extraKeys.join(' | ') }}",
        ];
        assert_cases(cases);
    }

    #[test]
    fn test_halfwidth_punctuation() {
        let cases = map! [
            "hello。" => "hello。",
            "hello 你好。" => "hello 你好。",
            "中文1\nhello world。\n中文2" => "中文1\nhello world.\n中文2",
            "  \n  Said：Come and，Join us！  \n  " => "  \n  Said: Come and, Join us!  \n  ",
            "Said：Come and，Join us！" => "Said: Come and, Join us!",
            "_（HTML5 Rocks）_" => "_(HTML5 Rocks)_",
            "  Start with space next word？Join us?" => "  Start with space next word? Join us?",
            ", Not start with word will not change。" => ", Not start with word will not change。",
            "：“Not start with word will not change”" => "：“Not start with word will not change”",
            "Come and， Join us！" => "Come and, Join us!",
            "The microphone or camera is occupied，Please check and re-record the video。" => "The microphone or camera is occupied, Please check and re-record the video.",
            "The Exchange’s" => "The Exchange's",
            "The “Convertible Amount” case。" => r#"The "Convertible Amount" case."#,
            "The“Convertible Amount”case。" => r#"The "Convertible Amount" case."#,
            "The（Convertible Amount）case！" => r#"The (Convertible Amount) case!"#,
            "The【Convertible Amount】case？" => "The [Convertible Amount] case?",
            "The「Convertible Amount」case：" => "The [Convertible Amount] case:",
            "The《Convertible Amount》case，" => r#"The "Convertible Amount" case,"#,
            "revenue conditions among the suppliers’ customers" => "revenue conditions among the suppliers' customers",
            "Reason: CORS header ‘Origin’ cannot be added" => "Reason: CORS header 'Origin' cannot be added",
        ];

        assert_cases(cases);
    }

    #[test]
    fn test_ignore_left_quote_in_last() {
        let cases = map! [
            "Escher puzzle (" => "Escher puzzle (",
            "Escher puzzle【" => "Escher puzzle【",
            "Escher puzzle《" => "Escher puzzle《",
            "Escher puzzle“" => "Escher puzzle“",
            "Escher puzzle‘" => "Escher puzzle‘",
            "Escher puzzle「" => "Escher puzzle「",
        ];

        assert_cases(cases);
    }

    #[test]
    fn test_halfwidth_punctuation_with_in_quote() {
        let cases = map! [
            r#""Hi！""# => r#""Hi!""#,
            r#""hello-world。""# => r#""hello-world.""#,
            r#""Only the first time break。""# => r#""Only the first time break.""#,
            r#"'Only the first time break？'"# => r#"'Only the first time break?'"#,
            r#"`Only the first time break！`"# => r#"`Only the first time break!`"#,
        ];

        assert_cases(cases);
    }
}
