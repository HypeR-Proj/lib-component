// SPDX-FileCopyrightText: 2026 roolrz
// SPDX-License-Identifier: Apache-2.0

use super::*;
use hyper_os::Status;

#[test]
fn consumer_error_boundary_preserves_rejection_and_transport()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    fn consumer(
        result: std::result::Result<(), ConnectionError>,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        result?;
        Ok(())
    }
    for kind in [
        RejectionKind::NotOffered,
        RejectionKind::InvalidRequest,
        RejectionKind::Unavailable,
        RejectionKind::Restarting,
        RejectionKind::TimedOut,
        RejectionKind::ResourceLimit,
    ] {
        let rejection = Rejection::new(kind, "provider generation ended");
        let error = consumer(Err(ConnectionError::Rejected(rejection.clone())))
            .err()
            .ok_or("missing rejection")?;
        assert!(matches!(error.downcast_ref::<ConnectionError>(),
            Some(ConnectionError::Rejected(actual)) if actual == &rejection));
    }
    let error = consumer(Err(Error::Status(Status::PEER_CLOSED).into()))
        .err()
        .ok_or("missing transport error")?;
    assert!(matches!(
        error.downcast_ref::<ConnectionError>(),
        Some(ConnectionError::Transport(Error::Status(
            Status::PEER_CLOSED
        )))
    ));
    Ok(())
}
