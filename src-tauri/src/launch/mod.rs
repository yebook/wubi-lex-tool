//! Pure parsing for process launch requests.

use std::ffi::{OsStr, OsString};

use serde::{Deserialize, Serialize};
use specta::Type;

const MAX_NAVIGATION_SCALARS: usize = 256;
const MAX_NOTICES: usize = 8;

/// A validated request that later shell layers can consume.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequest {
    /// Whether the initial window should be created hidden.
    pub start_hidden: bool,
    /// A syntax-validated internal path. Route ownership remains in the routing layer.
    pub navigation_path: Option<String>,
}

/// Stable launch diagnostic identifiers used by Rust, generated bindings and UI copy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LaunchNoticeCode {
    /// An argument was not valid Unicode.
    NonUnicodeArgument,
    /// An unsupported switch or positional argument was supplied.
    UnknownArgument,
    /// `/tray` appeared more than once.
    DuplicateTray,
    /// `--navigate` appeared more than once.
    DuplicateNavigate,
    /// `--navigate` had no following path value.
    MissingNavigatePath,
    /// The navigation path did not satisfy the transport envelope.
    InvalidNavigatePath,
}

/// A bounded, user-visible launch warning that never contains raw argv text.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LaunchNotice {
    /// Stable machine-readable code.
    pub code: LaunchNoticeCode,
    /// Chinese summary suitable for the initial status page.
    pub summary: String,
    /// Bounded technical evidence without the original argument value.
    pub detail: Option<String>,
    /// One-based position after the executable path, when applicable.
    pub argument_position: Option<u16>,
}

/// Result of parsing one process launch vector.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ParsedLaunch {
    /// Safe request. Any notice resets this to a normal visible launch.
    pub request: LaunchRequest,
    /// Bounded visible diagnostics.
    pub notices: Vec<LaunchNotice>,
}

/// Parses arguments after the executable path.
pub fn parse_launch_args<I>(arguments: I) -> ParsedLaunch
where
    I: IntoIterator<Item = OsString>,
{
    let arguments: Vec<OsString> = arguments.into_iter().collect();
    let mut request = LaunchRequest::default();
    let mut notices = Vec::new();
    let mut tray_seen = false;
    let mut navigate_seen = false;
    let mut index = 0usize;

    while index < arguments.len() {
        let position = argument_position(index);
        let Some(argument) = arguments[index].to_str() else {
            push_notice(
                &mut notices,
                LaunchNoticeCode::NonUnicodeArgument,
                "启动参数包含无法识别的文本，已按普通方式启动。",
                "参数不是有效的 Unicode 文本。",
                position,
            );
            index += 1;
            continue;
        };

        if argument.eq_ignore_ascii_case("/tray") {
            if tray_seen {
                push_notice(
                    &mut notices,
                    LaunchNoticeCode::DuplicateTray,
                    "后台启动参数重复，已按普通方式启动。",
                    "同一启动请求只能包含一次 /tray。",
                    position,
                );
            } else {
                tray_seen = true;
                request.start_hidden = true;
            }
            index += 1;
            continue;
        }

        if argument == "--navigate" {
            if navigate_seen {
                push_notice(
                    &mut notices,
                    LaunchNoticeCode::DuplicateNavigate,
                    "导航参数重复，已按普通方式启动。",
                    "同一启动请求只能包含一次 --navigate。",
                    position,
                );
                index = skip_navigate_value(&arguments, index);
                continue;
            }
            navigate_seen = true;

            let Some(candidate) = arguments.get(index + 1) else {
                push_notice(
                    &mut notices,
                    LaunchNoticeCode::MissingNavigatePath,
                    "导航参数缺少目标，已按普通方式启动。",
                    "--navigate 后必须提供一个内部路径。",
                    position,
                );
                index += 1;
                continue;
            };

            if is_switch(candidate) {
                push_notice(
                    &mut notices,
                    LaunchNoticeCode::MissingNavigatePath,
                    "导航参数缺少目标，已按普通方式启动。",
                    "--navigate 后必须提供一个内部路径。",
                    position,
                );
                index += 1;
                continue;
            }

            match candidate.to_str() {
                Some(path) => match validate_navigation_path(path) {
                    Ok(()) => request.navigation_path = Some(path.to_owned()),
                    Err(reason) => push_notice(
                        &mut notices,
                        LaunchNoticeCode::InvalidNavigatePath,
                        "导航目标无效，已按普通方式启动。",
                        reason,
                        argument_position(index + 1),
                    ),
                },
                None => push_notice(
                    &mut notices,
                    LaunchNoticeCode::NonUnicodeArgument,
                    "导航目标包含无法识别的文本，已按普通方式启动。",
                    "导航目标不是有效的 Unicode 文本。",
                    argument_position(index + 1),
                ),
            }
            index += 2;
            continue;
        }

        push_notice(
            &mut notices,
            LaunchNoticeCode::UnknownArgument,
            "存在不支持的启动参数，已按普通方式启动。",
            "参数类型不受支持。",
            position,
        );
        index += 1;
    }

    if notices.is_empty() {
        ParsedLaunch { request, notices }
    } else {
        ParsedLaunch {
            request: LaunchRequest::default(),
            notices,
        }
    }
}

fn skip_navigate_value(arguments: &[OsString], index: usize) -> usize {
    match arguments.get(index + 1) {
        Some(candidate) if !is_switch(candidate) => index + 2,
        _ => index + 1,
    }
}

fn is_switch(argument: &OsStr) -> bool {
    argument
        .to_str()
        .is_some_and(|value| value.starts_with("--") || value.eq_ignore_ascii_case("/tray"))
}

fn argument_position(index: usize) -> Option<u16> {
    u16::try_from(index + 1).ok()
}

fn push_notice(
    notices: &mut Vec<LaunchNotice>,
    code: LaunchNoticeCode,
    summary: &str,
    detail: &str,
    argument_position: Option<u16>,
) {
    if notices.len() < MAX_NOTICES {
        notices.push(LaunchNotice {
            code,
            summary: summary.to_owned(),
            detail: Some(detail.to_owned()),
            argument_position,
        });
    }
}

fn validate_navigation_path(path: &str) -> Result<(), &'static str> {
    if !path.starts_with('/') {
        return Err("内部路径必须以 / 开头。");
    }
    if path.chars().count() > MAX_NAVIGATION_SCALARS {
        return Err("内部路径超过 256 个字符。");
    }
    if path.chars().any(char::is_control) {
        return Err("内部路径不能包含控制字符。");
    }
    if path.contains(['\\', '?', '#']) {
        return Err("内部路径不能包含查询、片段或反斜杠。");
    }
    if path != "/"
        && path
            .split('/')
            .skip(1)
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err("内部路径不能包含空段、当前目录段或父目录段。");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{LaunchNoticeCode, LaunchRequest, parse_launch_args};
    use std::ffi::OsString;

    fn parse(arguments: &[&str]) -> super::ParsedLaunch {
        parse_launch_args(arguments.iter().map(OsString::from))
    }

    #[test]
    fn accepts_normal_tray_navigation_and_combined_launches() {
        assert_eq!(parse(&[]).request, LaunchRequest::default());
        assert_eq!(
            parse(&["/TRAY"]).request,
            LaunchRequest {
                start_hidden: true,
                navigation_path: None,
            }
        );
        assert_eq!(
            parse(&["--navigate", "/settings/runtime"]).request,
            LaunchRequest {
                start_hidden: false,
                navigation_path: Some("/settings/runtime".to_owned()),
            }
        );
        assert_eq!(
            parse(&["/tray", "--navigate", "/overview"]).request,
            LaunchRequest {
                start_hidden: true,
                navigation_path: Some("/overview".to_owned()),
            }
        );
    }

    #[test]
    fn rejects_duplicates_missing_values_and_unknown_arguments_without_panicking() {
        let cases = [
            (vec!["/tray", "/tray"], LaunchNoticeCode::DuplicateTray),
            (
                vec!["--navigate", "/one", "--navigate", "/two"],
                LaunchNoticeCode::DuplicateNavigate,
            ),
            (vec!["--navigate"], LaunchNoticeCode::MissingNavigatePath),
            (vec!["--other"], LaunchNoticeCode::UnknownArgument),
        ];

        for (arguments, expected) in cases {
            let parsed = parse(&arguments);
            assert_eq!(parsed.request, LaunchRequest::default());
            assert!(parsed.notices.iter().any(|notice| notice.code == expected));
        }

        let missing_then_unknown = parse(&["--navigate", "--other"]);
        assert_eq!(missing_then_unknown.notices.len(), 2);
        assert_eq!(
            missing_then_unknown.notices[0].code,
            LaunchNoticeCode::MissingNavigatePath
        );
        assert_eq!(
            missing_then_unknown.notices[1].code,
            LaunchNoticeCode::UnknownArgument
        );
    }

    #[test]
    fn rejects_invalid_navigation_envelopes_and_accepts_scalar_limit() {
        for path in [
            "settings",
            "/settings?tab=runtime",
            "/settings#runtime",
            "/settings\\runtime",
            "/settings//runtime",
            "/settings/./runtime",
            "/settings/../runtime",
            "/settings/\u{0007}runtime",
        ] {
            let parsed = parse(&["--navigate", path]);
            assert_eq!(parsed.request, LaunchRequest::default());
            assert_eq!(
                parsed.notices[0].code,
                LaunchNoticeCode::InvalidNavigatePath
            );
        }

        let at_limit = format!("/{}", "界".repeat(255));
        let over_limit = format!("/{}", "界".repeat(256));
        assert_eq!(
            parse_launch_args([OsString::from("--navigate"), OsString::from(at_limit)])
                .notices
                .len(),
            0
        );
        assert_eq!(
            parse_launch_args([OsString::from("--navigate"), OsString::from(over_limit)]).notices
                [0]
            .code,
            LaunchNoticeCode::InvalidNavigatePath
        );
    }

    #[cfg(windows)]
    #[test]
    fn rejects_non_unicode_arguments_without_panicking_or_echoing_input() {
        use std::os::windows::ffi::OsStringExt;

        let invalid = OsString::from_wide(&[0xD800]);
        let parsed = parse_launch_args([invalid]);
        assert_eq!(parsed.request, LaunchRequest::default());
        assert_eq!(parsed.notices[0].code, LaunchNoticeCode::NonUnicodeArgument);
        assert!(
            !parsed.notices[0]
                .detail
                .as_deref()
                .unwrap_or_default()
                .contains('\u{FFFD}')
        );
    }
}
