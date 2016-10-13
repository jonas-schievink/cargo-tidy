use super::TidyContext;

use std::str::FromStr;

macro_rules! indent_kinds {
    ( $( $kind:ident = $ch:expr ),+ ) => {
        #[derive(Debug, PartialEq)]
        pub enum IndentKind {
            $($kind),+
        }

        impl IndentKind {
            fn as_char(&self) -> char {
                match *self {
                    $( IndentKind::$kind => $ch, )+
                }
            }

            fn name(&self) -> &'static str {
                // I'd like to to_lower this, but the allocation doesn't feel justified :/
                match *self {
                    $( IndentKind::$kind => stringify!($kind), )+
                }
            }
        }

        const INDENT_KINDS: &'static [IndentKind] = &[
            $( IndentKind::$kind ),+
        ];
    };
}

indent_kinds!(Tabs = '\t', Spaces = ' ');

/// Describes an indentation style.
///
/// Indentation is done with N repetitions of an `IndentKind`.
///
/// To allow any number of indentation characters, N can be set to 1.
///
/// This is useful for enforcing indentation in certain code styles. For example, the following
/// code would be rejected by `IndentKind::Space` and an `indent_amount` of 4, because the last 2
/// lines are indented with 6 spaces:
///
/// ```rust,ignore
/// my_fn(first_parameter.get(),
///       second_parameter.get(),
///       &0);
/// ```
///
/// To be accepted, the code can be formatted like this (let's pretend it's too long to fit on
/// a single line):
///
/// ```rust,ignore
/// my_fn(
///     first_parameter.get(),
///     second_parameter.get(),
///     &0);
/// ```
///
/// Note that a different number than 1 is inherently incompatible with how rustfmt (currently)
/// decides to format code.
#[derive(Debug)]
pub struct IndentationStyle {
    indent_kind: IndentKind,
    indent_amount: u64,
}

impl FromStr for IndentationStyle {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn mk_err() -> Result<IndentationStyle, &'static str> {
            Err("invalid indentation style, expected a string like 'N tabs', 'N spaces', 'tabs' or \
                'spaces'")
        }

        // The accepted syntax is "N tab", "N tabs", "N space", "N spaces", "tabs", "spaces".
        // The latter 2 set an `indent_amount` of 1.

        let mut parts = s.split_whitespace();
        match (parts.next(), parts.next()) {
            (Some(number), Some(kind)) => {
                // "N tab(s)" / "N space(s)"
                let number = match number.parse() {
                    Ok(n) => n,
                    Err(_) => return mk_err(),
                };

                Ok(IndentationStyle {
                    indent_kind: match kind {
                        "tab" | "tabs" => IndentKind::Tabs,
                        "space" | "spaces" => IndentKind::Spaces,
                        _ => return mk_err(),
                    },
                    indent_amount: number,
                })
            }
            (Some(kind), None) => Ok(IndentationStyle {
                indent_kind: match kind {
                    "tabs" => IndentKind::Tabs,
                    "spaces" => IndentKind::Spaces,
                    _ => return mk_err(),
                },
                indent_amount: 1,
            }),
            _ => mk_err(),
        }
    }
}

pub fn check(cx: &mut TidyContext) {
    if let Some(ref style) = cx.config.indentation_style {
        // FIXME Write a test for this stuff!

        for (lineno, line) in cx.content.lines().enumerate() {
            // Check that the line isn't indented with the wrong indentation. To do that, take all
            // the whitespace at the beginning of the line and make sure it's the correct kind.
            'indent_char_loop: for indent_ch in line.chars().take_while(|ch| ch.is_whitespace()) {
                for indent_kind in INDENT_KINDS {
                    if *indent_kind != style.indent_kind {
                        if indent_ch == indent_kind.as_char() {
                            cx.error((lineno, 0),
                                format!("line is indented with {}, expected {}",
                                    indent_kind.name(),
                                    style.indent_kind.name()));
                            break 'indent_char_loop;
                        }
                    }
                }
            }

            // Check that it's indented with a correct amount of indentation
            let count = line.chars().take_while(|ch| *ch == style.indent_kind.as_char()).count();

            if count as u64 % style.indent_amount != 0 {
                // FIXME not sure if `count` is good here
                cx.error((lineno, count),
                    format!("line is indented with {} {}, expected a multiple of {}",
                        count,
                        style.indent_kind.name(),
                        style.indent_amount));
            }
        }
    }
}
