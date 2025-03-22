use base64::{engine::general_purpose, Engine as _};
use data_encoding::BASE32;
use hex;
use qrcode_generator::QrCodeEcc;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, TOTP};
use urlencoding;

const TOTP_PERIOD: u64 = 30;
const TOTP_SKEW: u64 = 1;
const BACKUP_CODE_LENGTH: usize = 10;
const DEFAULT_BACKUP_CODES_COUNT: usize = 10;

/// Tạo secret key ngẫu nhiên cho 2FA
pub fn generate_secret() -> String {
    let mut rng = rand::rng();
    let secret: Vec<u8> = (0..16).map(|_| rng.random::<u8>()).collect();
    BASE32.encode(&secret).trim_end_matches('=').to_string()
}

/// Tạo URL cho QR code
///
/// Format chuẩn cho Google Authenticator:
/// otpauth://totp/ISSUER:ACCOUNT_NAME?secret=SECRET&issuer=ISSUER
pub fn generate_totp_url(secret: &str, username: &str, issuer: &str) -> String {
    let encoded_issuer = urlencoding::encode(issuer);
    let encoded_username = urlencoding::encode(username);

    format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period={}",
        encoded_issuer, encoded_username, secret, encoded_issuer, TOTP_PERIOD
    )
}

/// Tạo QR code từ URL và trả về dưới dạng base64
pub fn generate_qr_code(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let png_data = qrcode_generator::to_png_to_vec(url, QrCodeEcc::Low, 1024)?;
    let encoded = general_purpose::STANDARD.encode(&png_data);
    Ok(format!("data:image/png;base64,{}", encoded))
}

/// Tạo đối tượng TOTP từ secret
fn create_totp(secret: &str) -> Result<TOTP, Box<dyn std::error::Error>> {
    let padded_secret = if secret.len() % 8 != 0 {
        let padding_len = 8 - (secret.len() % 8);
        let mut padded = String::from(secret);
        padded.push_str(&"=".repeat(padding_len));
        padded
    } else {
        secret.to_string()
    };

    let secret_bytes = BASE32.decode(padded_secret.as_bytes())?;
    let totp = TOTP::new(Algorithm::SHA1, 6, 1, TOTP_PERIOD, secret_bytes)?;
    Ok(totp)
}

/// Xác thực mã TOTP
pub fn verify_totp(secret: &str, code: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let totp = create_totp(secret)?;

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Lỗi lấy thời gian: {}", e))?;

    let time = current_time.as_secs();

    for i in 0..=TOTP_SKEW {
        let check_time = time.saturating_sub(i * TOTP_PERIOD);
        if totp.check(code, check_time) {
            return Ok(true);
        }

        let check_time = time.saturating_add(i * TOTP_PERIOD);
        if totp.check(code, check_time) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Tạo danh sách các mã backup dùng một lần
///
/// Mỗi mã có độ dài BACKUP_CODE_LENGTH ký tự và được tạo ngẫu nhiên
/// Trả về danh sách các mã backup dạng plain text và danh sách các mã đã được hash
pub fn generate_backup_codes(count: Option<usize>) -> (Vec<String>, Vec<String>) {
    let count = count.unwrap_or(DEFAULT_BACKUP_CODES_COUNT);
    let mut rng = rand::rng();
    let mut plain_codes = Vec::with_capacity(count);
    let mut hashed_codes = Vec::with_capacity(count);

    for _ in 0..count {
        // Tạo mã ngẫu nhiên với các ký tự chữ và số
        let code: String = (0..BACKUP_CODE_LENGTH)
            .map(|_| {
                let idx = rng.random_range(0..36);
                if idx < 10 {
                    // Số 0-9
                    (b'0' + idx as u8) as char
                } else {
                    // Chữ cái a-z
                    (b'a' + (idx - 10) as u8) as char
                }
            })
            .collect();

        // Hash mã để lưu trữ an toàn
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        let hashed = hex::encode(hasher.finalize());

        plain_codes.push(code);
        hashed_codes.push(hashed);
    }

    (plain_codes, hashed_codes)
}

/// Xác thực mã backup
///
/// So sánh mã người dùng nhập với danh sách các mã đã hash
pub fn verify_backup_code(code: &str, hashed_codes: &[String]) -> Option<usize> {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    let hashed_input = hex::encode(hasher.finalize());

    hashed_codes
        .iter()
        .position(|hashed| *hashed == hashed_input)
}

/// Định dạng mã backup để hiển thị cho người dùng
///
/// Ví dụ: "abcdefghij" -> "abcde-fghij"
pub fn format_backup_code(code: &str) -> String {
    if code.len() >= BACKUP_CODE_LENGTH {
        let (first, second) = code.split_at(BACKUP_CODE_LENGTH / 2);
        format!("{}-{}", first, second)
    } else {
        code.to_string()
    }
}
