use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

use marqant::{
    digest_state::{fhash, DigestState},
    log_summarizer::{LogSummarizer, SummarizerConfig},
    mq2_uni_decode, mq2_uni_encode, read_mq_metadata, Marqant, MQ2_UNI_DICT_ID,
};

pub fn run_cli() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let Some(cmd) = args.next() else {
        return print_help();
    };

    match cmd.as_str() {
        "dict-id" => {
            // Print dict_id derived from ~T/~S if present; or --uni prints built-in UNI dict id
            let mut input: Option<PathBuf> = None;
            let mut uni = false;
            for a in args.by_ref() {
                match a.as_str() {
                    "--uni" => uni = true,
                    s if !s.starts_with('-') && input.is_none() => input = Some(PathBuf::from(s)),
                    _ => return Err(anyhow!("unknown or duplicate arg: {a}")),
                }
            }
            if uni {
                println!("{}", MQ2_UNI_DICT_ID);
                return Ok(());
            }
            let mq = match input {
                Some(path) => fs::read_to_string(&path)
                    .with_context(|| format!("failed reading {}", path.display()))?,
                None => {
                    let mut buf = String::new();
                    io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };
            let info = read_mq_metadata(&mq)?;
            if let Some(id) = info.dict_id {
                println!("{}", id);
            } else {
                return Err(anyhow!(
                    "no ~T/~S maps present; cannot derive dict_id (use --uni for built-in)"
                ));
            }
        }
        "uni-encode" => {
            let mut input: Option<PathBuf> = None;
            let mut output: Option<PathBuf> = None;
            while let Some(a) = args.next() {
                match a.as_str() {
                    "-o" | "--output" => {
                        let Some(p) = args.next() else {
                            return Err(anyhow!("missing value for {a}"));
                        };
                        output = Some(PathBuf::from(p));
                    }
                    s if !s.starts_with('-') && input.is_none() => input = Some(PathBuf::from(s)),
                    _ => return Err(anyhow!("unknown or duplicate arg: {a}")),
                }
            }
            let bytes = match input {
                Some(path) => {
                    fs::read(&path).with_context(|| format!("failed reading {}", path.display()))?
                }
                None => {
                    let mut buf = Vec::new();
                    io::stdin().read_to_end(&mut buf)?;
                    buf
                }
            };
            let enc = mq2_uni_encode(&bytes)?;
            match output {
                Some(path) => fs::write(&path, enc)
                    .with_context(|| format!("failed writing {}", path.display()))?,
                None => {
                    io::stdout().write_all(&enc)?;
                }
            }
        }
        "uni-decode" => {
            let mut input: Option<PathBuf> = None;
            let mut output: Option<PathBuf> = None;
            while let Some(a) = args.next() {
                match a.as_str() {
                    "-o" | "--output" => {
                        let Some(p) = args.next() else {
                            return Err(anyhow!("missing value for {a}"));
                        };
                        output = Some(PathBuf::from(p));
                    }
                    s if !s.starts_with('-') && input.is_none() => input = Some(PathBuf::from(s)),
                    _ => return Err(anyhow!("unknown or duplicate arg: {a}")),
                }
            }
            let bytes = match input {
                Some(path) => {
                    fs::read(&path).with_context(|| format!("failed reading {}", path.display()))?
                }
                None => {
                    let mut buf = Vec::new();
                    io::stdin().read_to_end(&mut buf)?;
                    buf
                }
            };
            let dec = mq2_uni_decode(&bytes)?;
            match output {
                Some(path) => fs::write(&path, dec)
                    .with_context(|| format!("failed writing {}", path.display()))?,
                None => {
                    io::stdout().write_all(&dec)?;
                }
            }
        }
        "compress" => {
            let mut input: Option<PathBuf> = None;
            let mut output: Option<PathBuf> = None;
            let mut use_zlib = false;
            let mut use_semantic = false;
            let mut std_id: Option<String> = None;

            let iter = args.by_ref();
            while let Some(a) = iter.next() {
                match a.as_str() {
                    "-o" | "--output" => {
                        let Some(p) = iter.next() else {
                            return Err(anyhow!("missing value for {a}"));
                        };
                        output = Some(PathBuf::from(p));
                    }
                    "--binary" => {
                        use_zlib = true;
                    }
                    "--semantic" => {
                        use_semantic = true;
                    }
                    "--std" => {
                        let Some(id) = iter.next() else {
                            return Err(anyhow!("missing value for --std"));
                        };
                        std_id = Some(id);
                    }
                    s if !s.starts_with('-') && input.is_none() => input = Some(PathBuf::from(s)),
                    _ => return Err(anyhow!("unknown or duplicate arg: {a}")),
                }
            }

            let content = match input {
                Some(path) => fs::read_to_string(&path)
                    .with_context(|| format!("failed reading {}", path.display()))?,
                None => {
                    let mut buf = String::new();
                    io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };

            let mut flags = String::new();
            if use_zlib {
                flags.push_str("-zlib");
            }
            if use_semantic {
                if !flags.is_empty() {
                    flags.push(' ');
                }
                flags.push_str("-semantic");
            }
            if let Some(id) = &std_id {
                if !flags.is_empty() {
                    flags.push(' ');
                }
                flags.push_str("-std:");
                flags.push_str(id);
            }
            let flags_opt = if flags.is_empty() {
                None
            } else {
                Some(flags.as_str())
            };

            let mq = Marqant::compress_markdown_with_flags(&content, flags_opt)?;

            match output {
                Some(path) => fs::write(&path, mq)
                    .with_context(|| format!("failed writing {}", path.display()))?,
                None => {
                    io::stdout().write_all(mq.as_bytes())?;
                }
            }
        }
        "decompress" => {
            let mut input: Option<PathBuf> = None;
            let mut output: Option<PathBuf> = None;
            while let Some(a) = args.next() {
                match a.as_str() {
                    "-o" | "--output" => {
                        let Some(p) = args.next() else {
                            return Err(anyhow!("missing value for {a}"));
                        };
                        output = Some(PathBuf::from(p));
                    }
                    s if !s.starts_with('-') && input.is_none() => input = Some(PathBuf::from(s)),
                    _ => return Err(anyhow!("unknown or duplicate arg: {a}")),
                }
            }

            let mq = match input {
                Some(path) => fs::read_to_string(&path)
                    .with_context(|| format!("failed reading {}", path.display()))?,
                None => {
                    let mut buf = String::new();
                    io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };
            let md = Marqant::decompress_marqant(&mq)?;
            match output {
                Some(path) => fs::write(&path, md)
                    .with_context(|| format!("failed writing {}", path.display()))?,
                None => {
                    io::stdout().write_all(md.as_bytes())?;
                }
            }
        }
        "analyze" => {
            // very simple analysis: show token count and size effect
            let mut input: Option<PathBuf> = None;
            for a in args.by_ref() {
                match a.as_str() {
                    s if !s.starts_with('-') && input.is_none() => input = Some(PathBuf::from(s)),
                    _ => return Err(anyhow!("unknown or duplicate arg: {a}")),
                }
            }
            let content = match input {
                Some(path) => fs::read_to_string(&path)
                    .with_context(|| format!("failed reading {}", path.display()))?,
                None => {
                    let mut buf = String::new();
                    io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };
            let (tokens, tokenized) = Marqant::tokenize_content(&content);
            let savings = content.len() as isize - tokenized.len() as isize;
            println!("tokens: {}\nsavings: {} bytes", tokens.len(), savings);
        }
        "inspect" => {
            let mut input: Option<PathBuf> = None;
            let mut show_tokens = false;
            for a in args {
                match a.as_str() {
                    "--show-tokens" => {
                        show_tokens = true;
                    }
                    s if !s.starts_with('-') && input.is_none() => input = Some(PathBuf::from(s)),
                    _ => return Err(anyhow!("unknown or duplicate arg: {a}")),
                }
            }
            let mq = match input {
                Some(path) => fs::read_to_string(&path)
                    .with_context(|| format!("failed reading {}", path.display()))?,
                None => {
                    let mut buf = String::new();
                    io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };
            let info = read_mq_metadata(&mq)?;
            println!("kind: {}", info.kind);
            if let Some(v) = info.variant.as_deref() {
                println!("variant: {}", v);
            }
            if let Some(ts) = info.timestamp.as_deref() {
                println!("timestamp: {}", ts);
            }
            if let Some(o) = info.original_size {
                println!("original_size: {}", o);
            }
            if let Some(c) = info.compressed_size {
                println!("compressed_size: {}", c);
            }
            if let Some(t) = info.token_count {
                println!("token_count: {}", t);
            }
            if let Some(l) = info.level.as_deref() {
                println!("level/format: {}", l);
            }
            if let Some(id) = info.dict_id.as_deref() {
                println!("dict_id: {}", id);
            }
            if show_tokens {
                if let Some(t) = info.dict_t.as_deref() {
                    println!("~T{}", t);
                }
                if let Some(s) = info.dict_s.as_deref() {
                    println!("~S{}", s);
                }
            }
        }
        "tail" => {
            run_smart_tail(args)?;
        }
        _ => return print_help(),
    }

    Ok(())
}

fn print_help() -> Result<()> {
    let help = "mq - Marqant CLI\n\n\
Usage:\n\
  mq dict-id [<file.mq>|stdin] [--uni]\n\
  mq uni-encode <input> [-o <output>]\n\
  mq uni-decode <input> [-o <output>]\n\
  mq compress <input.md> [-o <output.mq>] [--binary] [--semantic] [--std <id>]\n\
  mq decompress <input.mq> [-o <output.md>]\n\
  mq analyze <input.md>\n\
  mq inspect <input.mq> [--show-tokens]\n\
  mq tail [<file>] [-n <lines>] [-D] [--raw]\n\n\
If <input> omitted, reads stdin. Writes to stdout if -o omitted.\n\n\
Smart Tail:\n\
  Drop-in tail replacement with AI-powered log summarization.\n\
  -D, --delta    Delta mode: only show changes since last run (stateful)\n\
  --raw          Classic tail behavior (no summarization)\n\n\
  Tip: alias tail='mq tail' for automatic smart logs everywhere!\n\n\
Delta Mode (-D):\n\
  Stateful analysis that tracks what you've seen before:\n\
  • Only shows NEW patterns (brand-new errors/warnings)\n\
  • Detects anomalies vs baseline (spikes, regressions)\n\
  • Suppresses repeated noise automatically\n\
  • State stored per-file in ~/.mq/state/ (inode-bound)\n\n\
Examples:\n\
  mq tail -n 100 app.log              # Smart summary\n\
  mq tail -D -n 2000 app.log          # Delta mode: only new patterns\n\
  tail -f app.log | mq tail           # Live smart tail\n\
  mq tail app.log --raw               # Classic tail mode";
    println!("{}", help);
    Ok(())
}

/// Run smart tail with log summarization
fn run_smart_tail(mut args: impl Iterator<Item = String>) -> Result<()> {
    let mut input: Option<PathBuf> = None;
    let mut num_lines: usize = 10;
    let mut raw_mode = false;
    let mut delta_mode = false;
    let mut config = SummarizerConfig::default();

    // Parse arguments
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-n" | "--lines" => {
                let Some(n) = args.next() else {
                    return Err(anyhow!("missing value for {arg}"));
                };
                num_lines = n.parse().with_context(|| format!("invalid number: {n}"))?;
            }
            "-D" | "--delta" => {
                delta_mode = true;
            }
            "--raw" => {
                raw_mode = true;
            }
            "--no-emoji" => {
                config.use_emojis = false;
            }
            s if !s.starts_with('-') && input.is_none() => {
                input = Some(PathBuf::from(s));
            }
            _ => return Err(anyhow!("unknown or duplicate arg: {arg}")),
        }
    }

    // Read input
    let lines = read_tail_lines(input.as_deref(), num_lines)?;

    // Raw mode - just print the lines
    if raw_mode {
        for line in lines {
            println!("{}", line);
        }
        return Ok(());
    }

    // Delta mode - only show changes since last run
    if delta_mode {
        if let Some(path) = input.as_deref() {
            run_delta_mode(path, &lines, &config)?;
        } else {
            return Err(anyhow!("Delta mode (-D) requires a file path (not stdin)"));
        }
    } else {
        // Smart mode - analyze and summarize
        let mut summarizer = LogSummarizer::new(config.clone());
        let summary = summarizer.summarize(&lines);
        let output = summary.format(&config);
        println!("{}", output);
    }

    Ok(())
}

/// Run delta mode - only show changes since last run
fn run_delta_mode(
    path: &std::path::Path,
    lines: &[String],
    config: &SummarizerConfig,
) -> Result<()> {
    // Load or create digest state
    let mut state = DigestState::load_or_create(path)?;

    let last_updated = if state.updated_unix > 0 {
        chrono::DateTime::from_timestamp(state.updated_unix, 0)
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        "never".to_string()
    };

    // Analyze lines and detect deltas
    let mut novel_patterns = Vec::new();
    let mut anomalies = Vec::new();
    let mut suppressed = std::collections::HashMap::new();

    for line in lines {
        let hash = fhash(line);
        let was_novel = state.is_novel(hash);

        // Record this pattern
        state.record_pattern(hash);
        let new_total = state.get_count(hash);

        if was_novel {
            // Brand new pattern
            novel_patterns.push(line.clone());
        } else {
            // Check for anomalies vs baseline
            let (multiplier, is_spike) = state.anomaly_score(hash, new_total);

            if is_spike && multiplier.is_finite() {
                anomalies.push((line.clone(), multiplier));
            } else {
                // Suppressed as noise
                *suppressed.entry(line.clone()).or_insert(0) += 1;
            }
        }
    }

    // Update baseline for next run
    if state.baseline.is_empty() {
        state.update_baseline();
    }

    // Save state
    state.updated_unix = chrono::Utc::now().timestamp();
    state.save(path)?;

    // Format output
    let emoji = config.use_emojis;
    println!("📊 delta (since {}, {} lines)", last_updated, lines.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Novel patterns
    if !novel_patterns.is_empty() {
        let icon = if emoji { "💎 " } else { "" };
        println!(
            "{}brand-new pattern{}",
            icon,
            if novel_patterns.len() == 1 { "" } else { "s" }
        );
        for pattern in novel_patterns.iter().take(10) {
            println!("  • {} (first seen)", pattern);
        }
        println!();
    }

    // Anomalies
    if !anomalies.is_empty() {
        let icon = if emoji { "🌟 " } else { "" };
        println!("{}anomalies vs baseline", icon);
        for (pattern, mult) in anomalies.iter().take(10) {
            println!("  • {} (↑{:.1}×)", pattern, mult);
        }
        println!();
    }

    // Suppressed noise
    if !suppressed.is_empty() {
        let icon = if emoji { "💤 " } else { "" };
        let total_suppressed: usize = suppressed.values().sum();
        println!("{}suppressed: {} lines", icon, total_suppressed);

        // Show top suppressed patterns
        let mut sorted: Vec<_> = suppressed.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));

        for (pattern, count) in sorted.iter().take(3) {
            let truncated = if pattern.len() > 60 {
                format!("{}...", &pattern[..57])
            } else {
                pattern.to_string()
            };
            println!("  • {}: {}", truncated, count);
        }
    }

    Ok(())
}

/// Read last N lines from file or stdin
fn read_tail_lines(path: Option<&std::path::Path>, num_lines: usize) -> Result<Vec<String>> {
    let content = match path {
        Some(p) => {
            fs::read_to_string(p).with_context(|| format!("failed reading {}", p.display()))?
        }
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    // Split into lines and take last N
    let all_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let start_idx = all_lines.len().saturating_sub(num_lines);
    Ok(all_lines[start_idx..].to_vec())
}
