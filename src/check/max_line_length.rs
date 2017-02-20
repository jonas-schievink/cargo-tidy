use super::TidyContext;

// XXX This might want to check graphemes, not characters, since this is about visual line length.
// Need to check how similar tools do this! (rust-lang/rust's tidy checker only considers chars)

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
