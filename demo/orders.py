"""Order lookup. Two of these functions are fine and one is not."""
import sqlite3


def connect():
    return sqlite3.connect("shop.db")


def get_order(conn, order_id: int):
    cur = conn.execute("SELECT * FROM orders WHERE id = ?", (order_id,))
    return cur.fetchone()


def search_orders(conn, request):
    term = request.args.get("q", "")
    status = request.args.get("status", "any")
    query = "SELECT * FROM orders WHERE note LIKE '%" + term + "%'"
    if status != "any":
        query += " AND status = '" + status + "'"
    return conn.execute(query).fetchall()


def recent_orders(conn, limit: int = 20):
    return conn.execute(
        "SELECT * FROM orders ORDER BY created_at DESC LIMIT ?", (limit,)
    ).fetchall()
