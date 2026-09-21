pub fn report_fix(
    field: &str,
    what: &str,
    outcome: anyhow::Result<()>,
    warnings: &mut Vec<String>,
) -> bool {
    match outcome {
        Ok(()) => {
            crate::agent_report!(
                "   {:<15} {}",
                field.bright_black(),
                format!("[FIXED] {what}").green().bold()
            );
            true
        }
        Err(e) => {
            crate::agent_report!(
                "   {:<15} {}",
                field.bright_black(),
                format!("[FAILED] {what}: {e}").red().bold()
            );
            warnings.push(format!("Could not repair {field} {what}: {e}"));
            false
        }
    }
}

pub trait AgentIntegration {
    /// CLI identifier used in `--[id]` (e.g. "vscode", "codex", "claude").
    fn id(&self) -> &'static str;

    /// Human-readable name for logging (e.g. "Claude Code").
    fn name(&self) -> &'static str;

    /// Runs the actual setup script.
    /// For Claude, it modifies `settings.json`. For Antigravity, it downloads the zip, etc.
    fn install(&self, exe_path: &str) -> anyhow::Result<()>;

    /// Uninstalls and removes configuration injected into the agent.
    fn uninstall(&self) -> anyhow::Result<()>;

    /// Runs a diagnostic check to see if the configuration is intact.
    /// `fix_mode` determines whether the doctor should attempt auto-repair.
    /// Returns `true` if healthy or successfully repaired.
    fn doctor_check(&self, fix_mode: bool, warnings: &mut Vec<String>) -> bool;

    /// What this host actually lets OMNI do to the bytes the model reads.
    ///
    /// Defaults to `McpOnly` so a new integration has to *claim* distillation
    /// rather than inherit the claim. Installing hooks is not the same as the
    /// host calling them, and three integrations shipped that gap at once:
    /// Cursor registered `afterFileEdit`, Cline registered Claude's
    /// `PreToolUse`, and Gemini matched on `"Bash"`. All three printed
    /// `[OK] installed` and recorded nothing (#351).
    fn tier(&self) -> Tier {
        Tier::McpOnly
    }
}

/// How much of OMNI's job a host permits, stated once so `doctor` and the README
/// cannot drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// The host applies OMNI's rewrite, so the model reads distilled output for
    /// its own built-in tools.
    Full,
    /// The ho