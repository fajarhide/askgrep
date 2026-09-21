fn claude_halves(is_claude: bool, is_all: bool, is_hook: bool, is_mcp: bool) -> (bool, bool) {
    if is_claude || is_all || (is_hook && is_mcp) || (!is_hook && !is_mcp) {
        return (true, true);
    }
    (is_hook, is_mcp)
}
