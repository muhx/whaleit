use serde::Serialize;

pub struct EmailService {
    provider: EmailProvider,
    from_email: String,
    from_name: String,
    app_url: String,
}

enum EmailProvider {
    SendGrid {
        api_key: String,
        client: reqwest::Client,
    },
    Smtp(lettre::AsyncSmtpTransport<lettre::Tokio1Executor>),
    None,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SendGridPayload {
    from: SendGridEmail,
    personalizations: Vec<SendGridPersonalization>,
    content: Vec<SendGridContent>,
}

#[derive(Serialize)]
struct SendGridEmail {
    email: String,
    name: Option<String>,
}

#[derive(Serialize)]
struct SendGridPersonalization {
    to: Vec<SendGridEmail>,
    subject: String,
}

#[derive(Serialize)]
struct SendGridContent {
    r#type: String,
    value: String,
}

impl EmailService {
    pub fn new() -> Self {
        let from_email = std::env::var("WF_MAIL_FROM")
            .or_else(|_| std::env::var("WF_SMTP_FROM"))
            .unwrap_or_else(|_| "noreply@localhost".to_string());
        let from_name =
            std::env::var("WF_MAIL_FROM_NAME").unwrap_or_else(|_| "WhaleIt".to_string());
        let app_url =
            std::env::var("WF_APP_URL").unwrap_or_else(|_| "http://localhost:8088".to_string());

        let provider = if let Ok(api_key) = std::env::var("WF_SENDGRID_API_KEY") {
            if !api_key.is_empty() {
                tracing::info!("Email provider: SendGrid");
                EmailProvider::SendGrid {
                    api_key,
                    client: reqwest::Client::new(),
                }
            } else {
                Self::build_smtp_provider()
            }
        } else {
            Self::build_smtp_provider()
        };

        EmailService {
            provider,
            from_email,
            from_name,
            app_url,
        }
    }

    fn build_smtp_provider() -> EmailProvider {
        let smtp_host = match std::env::var("WF_SMTP_HOST") {
            Ok(h) if !h.is_empty() => h,
            _ => {
                tracing::info!("Email provider: none (URLs will be logged)");
                return EmailProvider::None;
            }
        };
        let smtp_port: u16 = std::env::var("WF_SMTP_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(465);
        let smtp_username = std::env::var("WF_SMTP_USERNAME").unwrap_or_default();
        let smtp_password = std::env::var("WF_SMTP_PASSWORD").unwrap_or_default();

        use lettre::transport::smtp::authentication::Credentials;

        let transport = if smtp_port == 465 {
            lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&smtp_host)
                .expect("SMTP relay")
                .port(smtp_port)
                .credentials(Credentials::new(smtp_username, smtp_password))
                .build()
        } else {
            lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::starttls_relay(&smtp_host)
                .expect("SMTP starttls")
                .port(smtp_port)
                .credentials(Credentials::new(
                    std::env::var("WF_SMTP_USERNAME").unwrap_or_default(),
                    std::env::var("WF_SMTP_PASSWORD").unwrap_or_default(),
                ))
                .build()
        };
        tracing::info!("Email provider: SMTP ({}:{})", smtp_host, smtp_port);
        EmailProvider::Smtp(transport)
    }

    pub async fn send_verification_email(&self, email: &str, token: &str) -> anyhow::Result<()> {
        let url = format!("{}/auth/verify?token={}", self.app_url, token);
        let html = format!(
            "<p>Click <a href=\"{url}\">here</a> to verify your email address.</p>\
             <p>If you did not create an account, you can ignore this email.</p>"
        );
        self.send(email, "Verify your WhaleIt account", &html, &url)
            .await
    }

    pub async fn send_password_reset_email(&self, email: &str, token: &str) -> anyhow::Result<()> {
        let url = format!("{}/auth/reset-password?token={}", self.app_url, token);
        let html = format!(
            "<p>Click <a href=\"{url}\">here</a> to reset your password.</p>\
             <p>This link expires in 1 hour. If you did not request a password reset, you can ignore this email.</p>"
        );
        self.send(email, "Reset your WhaleIt password", &html, &url)
            .await
    }

    async fn send(
        &self,
        to: &str,
        subject: &str,
        html: &str,
        fallback_url: &str,
    ) -> anyhow::Result<()> {
        match &self.provider {
            EmailProvider::SendGrid { api_key, client } => {
                let payload = SendGridPayload {
                    from: SendGridEmail {
                        email: self.from_email.clone(),
                        name: Some(self.from_name.clone()),
                    },
                    personalizations: vec![SendGridPersonalization {
                        to: vec![SendGridEmail {
                            email: to.to_string(),
                            name: None,
                        }],
                        subject: subject.to_string(),
                    }],
                    content: vec![SendGridContent {
                        r#type: "text/html".to_string(),
                        value: html.to_string(),
                    }],
                };

                let resp = client
                    .post("https://api.sendgrid.com/v3/mail/send")
                    .header("Authorization", format!("Bearer {api_key}"))
                    .json(&payload)
                    .send()
                    .await?;

                let status = resp.status();
                if !status.is_success() {
                    let body = resp.text().await.unwrap_or_default();
                    anyhow::bail!("SendGrid API error ({}): {}", status, body);
                }
            }
            EmailProvider::Smtp(transport) => {
                use lettre::message::header::ContentType;
                use lettre::message::Mailbox;
                use lettre::AsyncTransport;

                let from: Mailbox = format!("{} <{}>", self.from_name, self.from_email)
                    .parse()
                    .expect("from address");
                let to_addr: Mailbox = to.parse()?;
                let message = lettre::Message::builder()
                    .from(from)
                    .to(to_addr)
                    .subject(subject)
                    .header(ContentType::TEXT_HTML)
                    .body(html.to_string())?;
                transport.send(message).await?;
            }
            EmailProvider::None => {
                tracing::warn!("No email provider configured. URL: {fallback_url}");
            }
        }
        Ok(())
    }
}
