//! Prompter library integration.
//!
//! Calls the prompter crate's public API to compose profiles into a
//! prompt string for injection into agent commands.

/// Base profiles always prepended to user-specified profiles.
const BASE_PROFILES: &[&str] = &["core.baseline", "core.agent", "core.git"];

/// Validate that all profile names exist in prompter's configuration.
///
/// # Errors
/// Returns an error listing unknown profile names.
pub fn validate_profiles(user_profiles: &[String]) -> Result<(), String> {
    let available = prompter::available_profiles(None)?;

    let unknown: Vec<&str> = user_profiles
        .iter()
        .filter(|p| !available.contains(p))
        .map(String::as_str)
        .collect();

    if unknown.is_empty() {
        Ok(())
    } else {
        Err(format!("unknown profiles: {}", unknown.join(", ")))
    }
}

/// Build the full profile list (base + user-specified).
#[must_use]
pub fn build_profile_list(user_profiles: &[String]) -> Vec<String> {
    let mut profiles: Vec<String> = BASE_PROFILES.iter().map(|&s| s.to_owned()).collect();
    profiles.extend_from_slice(user_profiles);
    profiles
}

/// Compose profiles into a prompt string.
///
/// Prepends base profiles, validates all names, then renders via the
/// prompter library.
///
/// # Errors
/// Returns an error if profile validation or rendering fails.
pub fn compose_prompt(user_profiles: &[String]) -> Result<String, String> {
    validate_profiles(user_profiles)?;
    let all_profiles = build_profile_list(user_profiles);

    let bytes = prompter::render_to_vec(&all_profiles, None)?;
    String::from_utf8(bytes).map_err(|e| format!("prompt contains invalid UTF-8: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_profile_list_prepends_base() {
        let user = vec!["rust.full".to_owned()];
        let all = build_profile_list(&user);
        assert_eq!(all[0], "core.baseline");
        assert_eq!(all[1], "core.agent");
        assert_eq!(all[2], "core.git");
        assert_eq!(all[3], "rust.full");
    }

    #[test]
    fn build_profile_list_empty_user() {
        let all = build_profile_list(&[]);
        assert_eq!(all.len(), 3);
    }
}
