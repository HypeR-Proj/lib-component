// SPDX-FileCopyrightText: 2026 roolrz
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn diagnostic(kind: u8, message: String) -> Request {
    match kind {
        0 => Request::Status { message },
        1 => Request::ReportFailure { message },
        _ => Request::ReportCriticalFailure { message },
    }
}

#[test]
fn diagnostics_fit_after_json_encoding() -> Result<()> {
    for text in [
        String::new(),
        "short diagnostic".into(),
        "a".repeat(1024),
        "\u{1f642}".repeat(256),
        "\0".repeat(256),
        "\"\\\n\t\0\u{1f642}".repeat(256),
    ] {
        for kind in 0..3 {
            let request = diagnostic(kind, text.clone());
            let bytes = encode_request(request)?;
            assert!(bytes.len() <= MAX_MESSAGE);
            let decoded: Request =
                serde_json::from_slice(&bytes).map_err(|_| Error::InvalidResponse)?;
            let message = match decoded {
                Request::Status { message } if kind == 0 => message,
                Request::ReportFailure { message } if kind == 1 => message,
                Request::ReportCriticalFailure { message } if kind == 2 => message,
                _ => return Err(Error::InvalidResponse),
            };
            assert!(text.starts_with(&message));
            let limit = if kind == 2 { MAX_MESSAGE } else { 256 };
            assert!(message.chars().count() <= limit);
            if text.chars().count() < 256 {
                assert_eq!(message, text);
            } else {
                assert!(!message.is_empty());
            }
            // Truncation preserves the longest prefix allowed by both bounds.
            if message.chars().count() < limit
                && let Some(next) = text[message.len()..].chars().next()
            {
                let mut longer = message;
                longer.push(next);
                let request = diagnostic(kind, longer);
                assert!(
                    serde_json::to_vec(&request)
                        .map_err(|_| Error::InvalidResponse)?
                        .len()
                        > MAX_MESSAGE
                );
            }
        }
    }
    Ok(())
}

#[test]
fn oversized_routes_are_rejected_without_truncation() {
    for request in [
        Request::Connect {
            name: "a".repeat(MAX_MESSAGE),
            deadline_ns: 1,
        },
        Request::Publish {
            name: "a".repeat(MAX_MESSAGE),
        },
    ] {
        assert!(matches!(
            encode_request(request),
            Err(Error::InvalidResponse)
        ));
    }
}

#[test]
fn rejection_frames_preserve_kind_and_longest_fitting_diagnostic() -> Result<()> {
    let controls: String = (0..32).filter_map(char::from_u32).collect();
    for kind in [
        RejectionKind::NotOffered,
        RejectionKind::InvalidRequest,
        RejectionKind::Unavailable,
        RejectionKind::Restarting,
        RejectionKind::TimedOut,
        RejectionKind::ResourceLimit,
    ] {
        for message in [
            String::new(),
            "a".repeat(300),
            "a".repeat(4096),
            "🙂".repeat(256),
            controls.repeat(256),
            "\"\\\n\t\0🙂".repeat(256),
        ] {
            // Public fields may bypass Rejection::new's character limit.
            let reply = Reply {
                error: Some(Rejection {
                    kind,
                    message: message.clone(),
                }),
            };
            let original = serde_json::to_vec(&reply).map_err(|_| Error::InvalidResponse)?;
            let bytes = encode_reply(reply)?;
            assert!(bytes.len() <= MAX_MESSAGE);
            let decoded: Reply =
                serde_json::from_slice(&bytes).map_err(|_| Error::InvalidResponse)?;
            let error = decoded.error.ok_or(Error::InvalidResponse)?;
            assert_eq!(error.kind, kind);
            assert!(message.starts_with(&error.message));
            if original.len() <= MAX_MESSAGE {
                assert_eq!(error.message, message);
            } else {
                let next = message[error.message.len()..]
                    .chars()
                    .next()
                    .ok_or(Error::InvalidResponse)?;
                let mut longer = error;
                longer.message.push(next);
                assert!(
                    serde_json::to_vec(&Reply {
                        error: Some(longer)
                    })
                    .map_err(|_| Error::InvalidResponse)?
                    .len()
                        > MAX_MESSAGE
                );
            }
        }
    }
    assert_eq!(encode_reply(Reply { error: None })?, b"{\"error\":null}");
    Ok(())
}
