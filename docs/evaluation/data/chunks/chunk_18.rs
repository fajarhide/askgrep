fn is_omni_hook_command(cmd: &str) -> bool {
    cmd.contains("omni")
        && (cmd.contains("--pre-hook")
            || cmd.contains("--post-hook")
            || cmd.contains("--session-start"))
}
