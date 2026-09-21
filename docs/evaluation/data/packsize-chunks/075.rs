fn run_install(source: &str) -> anyhow::Result<()> {
    let pi_bin = find_pi_binary().ok_or_else(|| {
        anyhow::anyhow!(
            "Pi binary not found on PATH. Install Pi first: https://github.com/earendil-works/pi"
        )
    })?;

    let args = vec!["install".to_string(), source.to_string()];
    crate::agent_report!("  {} Running: pi {}", "⟳".yellow(), args.join(" ").cyan());

    let output = Command::new(&pi_bin).args(&args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("  {} pi install failed: {}", "✗".red(), stderr.trim());
        eprintln!(
            "  {} You can try manually: pi {}",
            "→".cyan(),
            args.join(" ")
        );
        anyhow::bail!("Pi package install failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        for line in stdout.lines().take(10) {
            crate::agent_report!("    {}", line.bright_black());
        }
    }

    crate::agent_report!(
        "  {} Pi package installed successfully.",
        "✓".green().bold()
    );

    // Post-install: warn about duplicates.
    let snapshot = PiSettingsSnapshot::load();
    let duplicates = snapshot.duplicate_sources();
    if duplicates.len() > 1 {
        crate::agent_report!(
            "  {} {} OMNI-related sources detected in Pi settings:",
            "⚠".yellow(),
            duplicates.len()
        );
        for src in &duplicates {
            crate::agent_report!("    {} {}", "•".yellow(), src.bright_black());
        }
        crate::agent_report!(
            "  {} Remove duplicate entries to prevent double-loading.",
            "→".cyan()
        );
    }

    crate::agent_report!(
        "  {} Restart Pi to activate the OMNI extension.\n",
        "✓".green().bold()
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_omni_package_in_settings() {
        let json: Value =
            serde_json::from_str(r#"{"packages": [{"source": "git:github.com/fajarhide/omni"}]}"#)
                .unwrap();
        assert!(find_omni_references(&json));
    }

    #[test]
    fn detects_pi_omni_