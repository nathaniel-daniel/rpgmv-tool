use crate::Options as RootOptions;
use anyhow::Context;
use anyhow::bail;
use anyhow::ensure;
use clap::CommandFactory;
use clap::Parser;
use clap_complete::Shell;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Parser)]
#[command(about = "Generate shell completions")]
pub struct Options {
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

    #[arg(short = 's', long = "shell")]
    shell: Option<Shell>,

    #[arg(short = 'i', long = "install")]
    install: bool,
}

fn install(shell: Shell, command: &mut clap::Command, command_name: &str) -> anyhow::Result<()> {
    if !cfg!(windows) {
        bail!("The --install flag is not supported on this platform.");
    }

    if shell != Shell::PowerShell {
        bail!("The {shell} shell is not supported");
    }

    let output = Command::new("powershell.exe")
        .arg("-NoProfile")
        .args(["-Command", "Write-Host $PROFILE -NoNewline"])
        .output()?;
    ensure!(output.status.success());
    let profile =
        String::from_utf8(output.stdout).context("$PROFILE path contains invalid unicode")?;
    let profile = PathBuf::from(profile);
    let profile_parent = profile.parent().context("Missing $PROFILE parent folder")?;
    let completions_dir = profile_parent.join("Completions");

    match std::fs::create_dir(&completions_dir) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => {
            return Err(error).context("Failed to create Completions folder");
        }
    }

    let completion_file_name = format!("{command_name}.ps1");
    let completion_path = completions_dir.join(&completion_file_name);
    let completion_path_temp = completion_path.with_added_extension("temp");
    let mut file = File::options()
        .create(true)
        .write(true)
        .read(false)
        .truncate(false)
        .open(&completion_path_temp)
        .context("Failed to open completion file")?;
    file.try_lock()
        .context("failed to lock temp file for writing")?;
    file.set_len(0)?;
    clap_complete::generate(shell, command, command_name, &mut file);
    file.flush()?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(completion_path_temp, completion_path)?;

    let mut file = File::options()
        .create(true)
        .write(true)
        .read(true)
        .truncate(false)
        .open(&profile)
        .context("Failed to open $PROFILE")?;
    file.try_lock()
        .context("Failed to lock $PROFILE for writing")?;

    let completion_line = format!(r". $PSScriptRoot\Completions\{completion_file_name}");
    let mut profile_string = String::new();
    file.read_to_string(&mut profile_string)?;
    let has_completion_line = profile_string.lines().any(|line| line == completion_line);

    if !has_completion_line {
        if !profile_string.ends_with('\n') {
            file.write_all(b"\n")?;
        }
        file.write_all(completion_line.as_bytes())?;
    }
    file.flush()?;
    file.sync_all()?;
    drop(file);

    Ok(())
}

pub fn exec(options: Options) -> anyhow::Result<()> {
    let mut command = RootOptions::command();

    let shell = options
        .shell
        .map(Ok)
        .unwrap_or_else(|| Shell::from_env().context("Failed to determine shell"))?;

    let command_name = command.get_name().to_string();

    let stdout = std::io::stdout();
    let mut output: Box<dyn Write> = match options.output {
        Some(output) => {
            let file = File::create(&output)
                .with_context(|| format!("Failed to create file \"{}\"", output.display()))?;
            Box::new(file)
        }
        None => {
            let lock = stdout.lock();
            Box::new(lock)
        }
    };

    clap_complete::generate(shell, &mut command, &command_name, &mut output);

    if options.install {
        install(shell, &mut command, &command_name)?;
    }

    Ok(())
}
