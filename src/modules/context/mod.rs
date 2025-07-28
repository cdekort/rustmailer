// Copyright © 2025 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::error::RustMailerResult;

pub mod controller;
pub mod executors;
pub mod status;

pub trait Initialize {
    async fn initialize() -> RustMailerResult<()>;
}

pub trait RustMailTask {
    fn start();
}
