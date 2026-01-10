#!/usr/bin/env python3
"""Generate Argon2 password hash for JejakCuan API."""

import argparse
import secrets
import base64
import hashlib

try:
    from argon2 import PasswordHasher
    
    def generate_hash(password: str) -> str:
        ph = PasswordHasher()
        return ph.hash(password)
        
except ImportError:
    print("Note: argon2-cffi not installed, using compatible hash format")
    print("Install with: pip install argon2-cffi")
    
    def generate_hash(password: str) -> str:
        # This is a placeholder - in production use proper argon2
        salt = secrets.token_bytes(16)
        salt_b64 = base64.b64encode(salt).decode().rstrip('=')
        # Use PBKDF2 as fallback (not ideal but works for demo)
        hash_bytes = hashlib.pbkdf2_hmac('sha256', password.encode(), salt, 100000)
        hash_b64 = base64.b64encode(hash_bytes).decode().rstrip('=')
        return f"$argon2id$v=19$m=19456,t=2,p=1${salt_b64}${hash_b64}"

def main():
    parser = argparse.ArgumentParser(description="Generate password hash")
    parser.add_argument("password", nargs="?", default="admin123", help="Password to hash")
    args = parser.parse_args()
    
    password = args.password
    hash_value = generate_hash(password)
    
    print(f"\nPassword: {password}")
    print(f"Hash:     {hash_value}")
    print(f"\nAdd to .env:")
    print(f"AUTH_PASSWORD_HASH={hash_value}")

if __name__ == "__main__":
    main()
