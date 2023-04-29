use clap_complete::{generate_to, Shell};
use std::fs;

include!("src/cli.rs");

fn manpages(binary_name: &str) {
    let var = std::env::var_os("MANPAGES_DIR")
        .or_else(|| std::env::var_os("OUT_DIR"))
        .unwrap();
    let out_dir = std::path::PathBuf::from(var.to_string_lossy().to_string());
    let cmd = build_command();
    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer).unwrap();

    std::fs::write(out_dir.join(format!("{}.1", binary_name)), buffer).unwrap();
}

fn shell_completions(binary_name: &str) {
    let out_dir = std::env::var_os("SHELL_COMPLETIONS_DIR")
        .or_else(|| std::env::var_os("OUT_DIR"))
        .unwrap();
    fs::create_dir_all(&out_dir).unwrap();

    let mut command = build_command();
    for shell in [
        Shell::Bash,
        Shell::Fish,
        Shell::Zsh,
        Shell::PowerShell,
        Shell::Elvish,
    ] {
        generate_to(shell, &mut command, binary_name, &out_dir).unwrap();
    }
}

fn main() -> std::io::Result<()> {
    let binary_name = "powernotd";
    manpages(binary_name);
    shell_completions(binary_name);

    Ok(())
}
