use super::TidyContext;

pub fn check(cx: &mut TidyContext) {
    let max_length = cx.config.max_line_length;

    for (lineno, content) in cx.content.lines().enumerate() {
        let length = content.chars().count();
        if length as u64 > max_length {
            cx.error((lineno, 0), format!("line too long (has {} characters, the limit is {})",
                length,
                max_length));
        }
    }
}
