fn pi_local_settings_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".pi")
        .join("settings.json")
}

/// Read-only parsed Pi settings for diagnostics.
struct PiSettingsSnapshot {
    _path: PathBuf,
    json: Option<Value>,
}

impl PiSettingsSnapshot {
    /// Load settings, preferring whichever path contains an OMNI package reference.
    /// Falls back to whichever file exists.
    fn load() -> Self {
        let global_path = pi_settings_path();
        let local_path = pi_local_settings_path();

        let global_json = std::fs::read_to_string(&global_path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok());

        let local_json = std::fs::read_to_string(&local_path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok());

        let (path, json) = if let Some(ref g) = global_json
            && find_omni_references(g)
        {
            (global_path, global_json)
        } else if let Some(ref l) = local_json
            && find_omni_references(l)
        {
            (local_path, local_json)
        } else {
            (global_path, global_json.or(local_json))
        };

        Self { _path: path, json }
    }

    /// True if settings contain a package entry referencing OMNI or pi-omni.
    fn has_omni_package(&self) -> bool {
        let Some(val) = &self.json else {
            return false;
        };
        find_omni_references(val)
    }

    /// Return package/extension sources that appear to be OMNI-related duplicates.
    fn duplicate_sources(&self) -> Vec<String> {
        let Some(val) = &self.json else {
            return vec![];
        };
        collect_omni_sources(val)
    }
}

/// Search a JSON value recursively for strings containing "omni" (case-insensitive).