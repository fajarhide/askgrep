pub fn run_init(args: &[String]) -> anyhow::Result<()> {
    if super::wants_help(args) {
        print_help();
        return Ok(());
    }
    super::check_flags("init", args, FLAGS)?;

    let mut is_claude = super::has_flag(args, "--claude");
    let mut is_cursor = super::has_flag(args, "--cursor");
    let mut is_zed = super::has_flag(args, "--zed");
    let mut is_cline = super::has_flag(args, "--cline");
    let mut is_roo = super::has_flag(args, "--roo") || super::has_flag(args, "--roo-code");
    let mut is_copilot = super::has_flag(args, "--copilot");
    let mut is_gemini = super::has_flag(args, "--gemini");
    let mut is_opencode = super::has_flag(args, "--opencode");
    let mut is_codex = super::has_flag(args, "--codex");
    let mut is_openclaw = super::has_flag(args, "--openclaw");
    let mut is_antigravity = super::has_flag(args, "--antigravity");
    let mut is_hermes = super::has_flag(args, "--hermes");
    let mut is_vscode = super::has_flag(args, "--vscode");
    let mut is_pi = super::has_flag(args, "--pi");

    let mut is_hook = super::has_flag(args, "--hook");
    let mut is_mcp = super::has_flag(args, "--mcp");
    let is_all = super::has_flag(args, "--all");
    let is_status = super::has_flag(args, "--status");
    let is_uninstall = super::has_flag(args, "--uninstall");

    if is_all {
        is_claude = true;
        is_hook = true;
        is_mcp = true;
    }

    // No flags -> Interactive Mode
    let no_flags = !is_claude
        && !is_cursor
        && !is_zed
        && !is_cline
        && !is_roo
        && !is_copilot
        && !is_gemini
        && !is_opencode
        && !is_codex
        && !is_openclaw
        && !is_antigravity
        && !is_hermes
        && !is_vscode
        && !is_pi
        && !is_status
        && !is_uninstall
        && !is_hook
        && !is_mcp;

    // Set when the menu could not be shown, so `target_ids` below installs into
    // the host that ran the command instead of erroring on the absent tty (#528).
    let mut detected: Option<&'static str> = None;

    if no_flags {
        println!(
            "\n{} {}: Setup OMNI for your preferred AI Agent\n",
            "omni".bold().cyan(),
            "init".bold().yellow()
        );

        if !std::io::stdin().is_terminal() {
            let host = non_interactive_host()?;
            detected = Some(host);
            prin