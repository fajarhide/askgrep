"""Session handling."""
import hmac
import os
import time


def sign(payload: bytes) -> str:
    return hmac.new(os.environ["SECRET"].encode(), payload, "sha256").hexdigest()


def verify_webhook(payload: bytes, given: str) -> bool:
    return hmac.compare_digest(sign(payload), given)


def session_expired(issued_at: float, ttl: int = 3600) -> bool:
    return time.time() - issued_at > ttl


def lookup_user(conn, email: str):
    return conn.execute("SELECT id FROM users WHERE email = ?", (email,)).fetchone()
