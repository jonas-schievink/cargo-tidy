use super::TidyContext;

pub fn check(cx: &mut TidyContext) {
    // FIXME we could give forbidden regexes names and print them instead of the regex
    for (lineno, line) in cx.lines_with_endings.iter().enumerate() {
        for matched_regex in cx.config.forbidden_content.0.matches(line).into_iter() {
            // FIXME apparently we do not get the actual position of the match, but we could create
            // a new `Regex` and ask it so we can set the error column properly
            let regex_str = &cx.config.forbidden_content.1[matched_regex];
            cx.error((lineno, 0), format!("line matches forbidden string '{}'", regex_str));
        }
    }
}
