use std::env;
use lettre::{ Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;

pub async fn send_stock_alert_email(email_message: String) -> Result<(), Box<dyn std::error::Error>> {
    let smtp_user = env::var("SMTP_USERNAME")?;
    let smtp_pass = env::var("SMTP_PASSWORD")?;
    let smtp_server = env::var("SMTP_SERVER")?;
    let to_address = env::var("ADMIN_EMAIL")?; 

    let email = Message::builder()
        .from(smtp_user.parse()?)
        .to(to_address.parse()?)
        .subject(" Product Out of Stock Alert")
        .body(format!("{email_message}"))?;

  
    let creds = Credentials::new(smtp_user, smtp_pass);

    let mailer = SmtpTransport::relay(&smtp_server)?
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => {
            println!("Stock alert email sent for sent");
            Ok(())
        },
        Err(e) => {
            println!("Email send failed: {:?}", e);
 
            Err(Box::new(e))
        }
    }
}
