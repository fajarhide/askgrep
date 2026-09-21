"""Reporting helpers."""


def revenue_by_day(conn, start: str, end: str):
    return conn.execute(
        "SELECT day, SUM(total) FROM orders WHERE day BETWEEN ? AND ? GROUP BY day",
        (start, end),
    ).fetchall()


def top_products(conn, request):
    category = request.args.get("category", "all")
    sql = "SELECT name, COUNT(*) FROM items"
    if category != "all":
        sql = sql + " WHERE category = '" + category + "'"
    return conn.execute(sql + " GROUP BY name ORDER BY 2 DESC").fetchall()
