use std::sync::Arc;

use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use whaleit_core::users::UserRepositoryTrait;

pub struct EmailService {
    mailer: Option<AsyncSmtpTransport<Tokio1Executor>>,
    from: Mailbox,
    app_url: String,
    user_repo: Arc<dyn UserRepositoryTrait>,
}

impl EmailService {
    pub fn new(user_repo: Arc<dyn UserRepositoryTrait>) -> Self {
        let smtp_host = std::env::var("WF_SMTP_HOST").ok();
        let smtp_port: u16 = std::env::var("WF_SMTP_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(465);
        let smtp_username = std::env::var("WF_SMTP_USERNAME").unwrap_or_default();
        let smtp_password = std::env::var("WF_SMTP_PASSWORD").unwrap_or_default();
        let smtp_from = std::env::var("WF_SMTP_FROM")
            .unwrap_or_else(|_| "WhaleIt <noreply@localhost>".to_string());
        let app_url =
            std::env::var("WF_APP_URL").unwrap_or_else(|_| "http://localhost:8088".to_string());

        let from: Mailbox = smtp_from.parse().unwrap_or_else(|_| {
            "WhaleIt <noreply@localhost>"
                .parse()
                .expect("default from address")
        });

        let mailer = smtp_host.map(|host| {
            let transport = if smtp_port == 465 {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
                    .unwrap()
                    .port(smtp_port)
                    .credentials(Credentials::new(smtp_username, smtp_password))
                    .build()
            } else {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
                    .unwrap()
                    .port(smtp_port)
                    .credentials(Credentials::new(
                        std::env::var("WF_SMTP_USERNAME").unwrap_or_default(),
                        std::env::var("WF_SMTP_PASSWORD").unwrap_or_default(),
                    ))
                    .build()
            };
            transport
        });

        EmailService {
            mailer,
            from,
            app_url,
            user_repo,
        }
    }

    pub async fn send_verification_email(
        &self,
        email: &str,
        token: &str,
    ) -> anyhow::Result<()> {
        let verify_url = format!("{}/auth/verify?token={}", self.app_url, token);

        if let Some(ref mailer) = self.mailer {
            let to: Mailbox = email.parse()?;
            let message = Message::builder()
                .from(self.from.clone())
                .to(to)
                .subject("Verify your WhaleIt account")
                .header(ContentType::TEXT_HTML)
                .body(format!(
                    "<p>Click <a href=\"{verify_url}\">here</a> to verify your email address.</p>\
                     <p>If you did not create an account, you can ignore this email.</p>"
                ))?;

            mailer.send(message).await?;
        } else {
            tracing::warn!(
                "SMTP not configured. Verification URL: {verify_url}"
            );
        }

        Ok(())
    }

    pub async fn send_password_reset_email(
        &self,
        email: &str,
        token: &str,
    ) -> anyhow::Result<()> {
        let reset_url = format!("{}/auth/reset-password?token={}", self.app_url, token);

        if let Some(ref mailer) = self.mailer {
            let to: Mailbox = email.parse()?;
            let message = Message::builder()
                .from(self.from.clone())
                .to(to)
                .subject("Reset your WhaleIt password")
                .header(ContentType::TEXT_HTML)
                .body(format!(
                    "<p>Click <a href=\"{reset_url}\">here</a> to reset your password.</p>\
                     <p>This link expires in 1 hour. If you did not request a password reset, you can ignore this email.</p>"
                ))?;

            mailer.send(message).await?;
        } else {
            tracing::warn!(
                "SMTP not configured. Password reset URL: {reset_url}"
            );
        }

        Ok(())
    }

    pub fn user_repo(&self) -> &Arc<dyn UserRepositoryTrait> {
        &self.user_repo
    }
}
