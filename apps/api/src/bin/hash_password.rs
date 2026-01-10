//! Password hashing utility for JejakCuan

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "verify" {
        // Verify mode: check current .env hash
        dotenvy::dotenv().ok();
        let hash = std::env::var("AUTH_PASSWORD_HASH").unwrap_or_default();
        println!("AUTH_PASSWORD_HASH from env:");
        println!("  Value: {}", hash);
        println!("  Length: {}", hash.len());
        
        match PasswordHash::new(&hash) {
            Ok(parsed) => {
                println!("  Hash parses: OK");
                let valid = Argon2::default().verify_password(b"admin123", &parsed).is_ok();
                println!("  Password 'admin123' valid: {}", valid);
            }
            Err(e) => {
                println!("  Hash parse error: {:?}", e);
            }
        }
        return;
    }
    
    let password = args.get(1).map(|s| s.as_str()).unwrap_or("admin123");
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password");
    
    println!("Password: {}", password);
    println!("Hash: {}", hash);
    println!();
    println!("Add to .env:");
    println!("AUTH_PASSWORD_HASH={}", hash);
}
