// autocorrect: false
mod fullwidth;
mod halfwidth;
mod rule;
pub(crate) mod spellcheck;
mod strategery;
mod word;

use regex::Regex;
use rule::{Rule, RuleResult};

use crate::result::Severity;

lazy_static! {
    static ref RULES: Vec<Rule> = vec![
        // Rule: space-word
        Rule::new("space-word", word::format_space_word),
        // Rule: space-punctuation
        Rule::new("space-punctuation", word::format_space_punctuation),
        // Rule: fullwidth
        Rule::new("fullwidth", fullwidth::format),
        // Rule: halfwidth
        Rule::new("halfwidth", halfwidth::format),
    ];

    static ref AFTER_RULES: Vec<Rule> = vec![
        // Rule: no-space-fullwidth
        Rule::new("no-space-fullwidth", word::format_no_space_fullwidth),
        Rule::new("spellcheck", spellcheck::format),
    ];
}

lazy_static! {
    static ref FULL_DATE_RE: Regex = regexp!(
        "{}",
        r"[ ]{0,}\d+[ ]{0,}年 [ ]{0,}\d+[ ]{0,}月 [ ]{0,}\d+[ ]{0,}[日号][ ]{0,}"
    );
    static ref CJK_RE: Regex = regexp!("{}", r"\p{CJK}");
    static ref SPACE_RE: Regex = regexp!("{}", r"[ ]");
    // start with Path or URL http://, https://, mailto://, app://, /foo/bar/dar, without //foo/bar/dar
    static ref PATH_RE: Regex = regexp!("{}", r"^(([a-z\d]+)://)|(^/?[\w\d\-]+/)");
}

/// Get all rule names for default enable
#[allow(dead_code)]
pub fn default_rule_names() -> Vec<String> {
    let mut rule_names = vec![];
    RULES.iter().for_each(|r| rule_names.push(r.name.clone()));
    AFTER_RULES
        .iter()
        .for_each(|r| rule_names.push(r.name.clone()));

    rule_names
}

pub(crate) fn format_or_lint(text: &str, lint: bool) -> RuleResult {
    let mut result = RuleResult::new(text);

    // skip if not has CJK
    if !CJK_RE.is_match(text) {
        return result;
    }

    result.out = String::from("");
    let mut part = String::new();
    for ch in text.chars() {
        part.push(ch);

        // Is next char is newline or space, break part to format
        if ch == ' ' || ch == '\n' || ch == '\r' {
            let mut sub_result = RuleResult::new(&part.clone());
            sub_result.severity = result.severity;

            part.clear();

            format_part(&mut sub_result, lint);

            result.out.push_str(&sub_result.out);
            result.severity = sub_result.severity;
        }
    }

    if !part.is_empty() {
        let mut sub_result = RuleResult::new(&part.clone());
        sub_result.severity = result.severity;

        format_part(&mut sub_result, lint);

        result.out.push_str(&sub_result.out);
        result.severity = sub_result.severity;
    }

    format_after_rules(&mut result, lint);

    result
}

fn format_part(result: &mut RuleResult, lint: bool) {
    if !CJK_RE.is_match(&result.out) {
        return;
    }

    if PATH_RE.is_match(&result.out) {
        return;
    }

    for rule in RULES.iter() {
        if lint {
            rule.lint(result);
        } else {
            rule.format(result);
        }
    }
}

fn format_after_rules(result: &mut RuleResult, lint: bool) {
    for rule in AFTER_RULES.iter() {
        if lint {
            rule.lint(result);
        } else {
            rule.format(result);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rule_names() {
        let rule_names = default_rule_names();
        let expect = vec![
            "space-word",
            "space-punctuation",
            "fullwidth",
            "halfwidth",
            "no-space-fullwidth",
            "spellcheck",
        ];
        assert_eq!(expect, rule_names);
    }

    #[test]
    fn test_format_part() {
        let mut result = RuleResult::new("Hello世界.");
        format_part(&mut result, false);
        assert_eq!("Hello 世界。", result.out);
        assert_eq!(Severity::Error, result.severity);

        let mut result = RuleResult::new("Hello世界.");
        format_part(&mut result, true);
        assert_eq!("Hello 世界。", result.out);
        assert_eq!(Severity::Error, result.severity);

        let mut result = RuleResult::new("Hello 世界。");
        format_part(&mut result, true);
        assert_eq!("Hello 世界。", result.out);
        assert_eq!(Severity::Pass, result.severity);
    }

    #[test]
    fn test_format_after_rules() {
        crate::config::setup_test();

        let mut result = RuleResult::new("测试 ios 应用， 与技术");
        format_after_rules(&mut result, false);
        assert_eq!("测试 ios 应用，与技术", result.out);
        assert_eq!(Severity::Error, result.severity);

        let mut result = RuleResult::new("测试 ios 应用， 与技术");
        format_after_rules(&mut result, true);
        assert_eq!("测试 iOS 应用，与技术", result.out);
        assert_eq!(Severity::Error, result.severity);
    }

    #[test]
    fn test_format_or_lint() {
        crate::config::setup_test();

        let result = format_or_lint("测试ios应用， 与技术", false);
        assert_eq!("测试 ios 应用，与技术", result.out);
        assert_eq!(Severity::Error, result.severity);

        let result = format_or_lint("测试ios应用， 与技术", true);
        assert_eq!("测试 iOS 应用，与技术", result.out);
        assert_eq!(Severity::Error, result.severity);

        // Pass case
        let result = format_or_lint("测试 iOS 应用，与技术", false);
        assert_eq!("测试 iOS 应用，与技术", result.out);
        assert_eq!(Severity::Pass, result.severity);

        let result = format_or_lint("测试 iOS 应用，与技术", true);
        assert_eq!("测试 iOS 应用，与技术", result.out);
        assert_eq!(Severity::Pass, result.severity);
    }
}
