use bright_sdk_download::{fetch_sdk_with_progress, list_platforms, resolve_sdk_with_hash, Step};
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::process;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    let exe = std::path::Path::new(&args[0])
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("bright-sdk-downloader")
        .to_string();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("");

    let result = match cmd {
        "resolve" => cmd_resolve(&args[2..], &exe),
        "fetch" => cmd_fetch(&args[2..], &exe),
        "platforms" => cmd_platforms(),
        "--version" | "-V" => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            return;
        }
        _ => {
            print_usage(&exe);
            process::exit(if cmd.is_empty() { 0 } else { 1 });
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn cmd_resolve(args: &[String], exe: &str) -> Result<(), bright_sdk_download::Error> {
    let (platform, version, _, hash, cert) = parse_args(args);
    let platform = platform.unwrap_or_else(|| {
        eprintln!("Usage: {exe} resolve -p <platform> [-v <version>] [--hash <hash>] [--cert]");
        process::exit(1);
    });
    let result = resolve_sdk_with_hash(&platform, &version, hash.as_deref(), cert)?;
    println!("{}", serde_json::to_string(&result).unwrap());
    Ok(())
}

fn cmd_fetch(args: &[String], exe: &str) -> Result<(), bright_sdk_download::Error> {
    let (platform, version, output, hash, cert) = parse_args(args);
    let platform = platform.unwrap_or_else(|| {
        eprintln!("Usage: {exe} fetch -p <platform> [-v <version>] [-o <dir>] [--hash <hash>] [--cert]");
        process::exit(1);
    });

    let start = Instant::now();
    let mut step_times: Vec<(Step, f64)> = Vec::new();
    let mut current_step = Step::Resolve;
    let mut step_start = Instant::now();

    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::with_template("{bar:40.cyan/blue} {pos}% | {elapsed_precise} | {msg}")
            .unwrap()
            .progress_chars("██░"),
    );
    pb.set_message("resolve");
    pb.set_position(0);

    let pb_clone = pb.clone();
    let result = fetch_sdk_with_progress(
        &platform,
        &version,
        &output,
        hash.as_deref(),
        cert,
        Some(Box::new(move |step, done, total| {
            if step != current_step {
                let elapsed = step_start.elapsed().as_secs_f64();
                step_times.push((current_step, elapsed));
                current_step = step;
                step_start = Instant::now();
            }
            let pct = match step {
                Step::Resolve => 5,
                Step::Download => {
                    if total > 0 {
                        5 + ((done as f64 / total as f64) * 80.0) as u64
                    } else {
                        5
                    }
                }
                Step::Verify => 88,
                Step::Extract => 95,
            };
            pb_clone.set_position(pct.min(100));
            pb_clone.set_message(step.to_string());
        })),
    );

    pb.set_position(100);
    pb.set_message("done");
    pb.finish_and_clear();

    let result = result?;

    let total_secs = start.elapsed().as_secs_f64();
    eprintln!("Done → {} ({:.1}s)", result.output, total_secs);
    println!("{}", serde_json::to_string(&result).unwrap());
    Ok(())
}

fn cmd_platforms() -> Result<(), bright_sdk_download::Error> {
    let platforms = list_platforms()?;
    println!("{}", serde_json::to_string(&platforms).unwrap());
    Ok(())
}

fn parse_args(args: &[String]) -> (Option<String>, String, String, Option<String>, bool) {
    let mut platform = None;
    let mut version = "latest".to_string();
    let mut output = ".".to_string();
    let mut hash = None;
    let mut cert = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--platform" => {
                i += 1;
                if i < args.len() {
                    platform = Some(args[i].clone());
                }
            }
            "-v" | "--version" => {
                i += 1;
                if i < args.len() {
                    version = args[i].clone();
                }
            }
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output = args[i].clone();
                }
            }
            "--hash" | "-h" => {
                i += 1;
                if i < args.len() {
                    hash = Some(args[i].clone());
                }
            }
            "--cert" | "-c" => {
                cert = true;
            }
            other if !other.starts_with('-') && platform.is_none() => {
                platform = Some(other.to_string());
            }
            _ => {}
        }
        i += 1;
    }
    (platform, version, output, hash, cert)
}

#[cfg(test)]
mod tests {
    use super::parse_args;

    #[test]
    fn parse_args_cert_long_flag() {
        let args: Vec<String> =
            vec!["-p".into(), "win".into(), "--cert".into()];
        let (platform, _, _, _, cert) = parse_args(&args);
        assert_eq!(platform.as_deref(), Some("win"));
        assert!(cert);
    }

    #[test]
    fn parse_args_cert_short_flag() {
        let args: Vec<String> = vec!["-p".into(), "win".into(), "-c".into()];
        let (_, _, _, _, cert) = parse_args(&args);
        assert!(cert);
    }

    #[test]
    fn parse_args_cert_false_by_default() {
        let args: Vec<String> = vec!["-p".into(), "android".into()];
        let (_, _, _, _, cert) = parse_args(&args);
        assert!(!cert);
    }

    #[test]
    fn parse_args_hash_and_cert_together() {
        let args: Vec<String> = vec![
            "-p".into(), "win".into(),
            "--hash".into(), "abc123".into(),
            "--cert".into(),
        ];
        let (_, _, _, hash, cert) = parse_args(&args);
        assert_eq!(hash.as_deref(), Some("abc123"));
        assert!(cert);
    }

    #[test]
    fn parse_args_all_flags() {
        let args: Vec<String> = vec![
            "-p".into(), "win".into(),
            "-v".into(), "1.617.770".into(),
            "-o".into(), "/tmp".into(),
            "-h".into(), "deadbeef".into(),
            "-c".into(),
        ];
        let (platform, version, output, hash, cert) = parse_args(&args);
        assert_eq!(platform.as_deref(), Some("win"));
        assert_eq!(version, "1.617.770");
        assert_eq!(output, "/tmp");
        assert_eq!(hash.as_deref(), Some("deadbeef"));
        assert!(cert);
    }
}

fn print_usage(exe: &str) {
    eprintln!(
        "{exe} — BrightSDK download CLI (Rust)\n\n\
         Commands:\n\
         \x20 resolve    Resolve version + download URL (JSON)\n\
         \x20 fetch      Download and extract SDK archive\n\
         \x20 platforms  List available platform keys\n\n\
         Options:\n\
         \x20 -p, --platform   Platform key (android, ios, tizen...)\n\
         \x20 -v, --version    Version or \"latest\" (default: latest)\n\
         \x20 -o, --output     Output directory (default: .)\n\
         \x20 -h, --hash       Override cert hash (win only)\n\
         \x20 -c, --cert       Use certified URL (win only)\n\n\
         Environment:\n\
         \x20 SDK_API_KEY      Required. BrightSDK API key.\n\n\
         Examples:\n\
         \x20 {exe} resolve -p android\n\
         \x20 {exe} fetch -p ios -o ./libs\n\
         \x20 {exe} fetch -p win --cert\n\
         \x20 {exe} platforms"
    );
}
