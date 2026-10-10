pub const API_URL: &str = match option_env!("VITE_API_URL") {
    Some(valor) => valor,
    None => "http://localhost:3000",
};

pub const APP_ID: Option<&str> = option_env!("VITE_APP_ID");
pub const POS_ID: Option<&str> = option_env!("VITE_POS_ID");
pub const BUSINESS_ID: Option<&str> = option_env!("VITE_BUSINESS_ID");
pub const MANUAL_QR_PASSWORD: Option<&str> = option_env!("VITE_MANUAL_QR_PASSWORD");
