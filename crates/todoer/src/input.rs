pub fn resolve_input(arg: &str, stdin_value: Option<String>) -> anyhow::Result<String> {
    if arg == "-" {
        return stdin_value.ok_or_else(|| anyhow::anyhow!("stdin required"));
    }
    Ok(arg.to_string())
}
