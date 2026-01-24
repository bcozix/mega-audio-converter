use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Select, Text};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "audioconv", version = "1.0", about = "AI Audio Converter 2026")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available quality profiles
    Profiles,
    /// Convert a single file
    Convert {
        #[arg(short, long)]
        input: Option<String>,
        #[arg(short, long)]
        quality: Option<String>,
        #[arg(short, long)]
        all: bool,
        /// Use fast FFmpeg cancellation instead of AI for Karaoke
        #[arg(long)]
        fast_karaoke: bool,
    },
    /// Batch convert a directory
    Batch {
        #[arg(short, long)]
        dir: String,
        #[arg(short, long)]
        quality: String,
        #[arg(long)]
        fast_karaoke: bool,
    },
}

struct Profile {
    name: &'static str,
    ext: &'static str,
    codec: &'static str,
    bitrate: Option<&'static str>,
}

const PROFILES: &[Profile] = &[
    Profile { name: "low", ext: "ogg", codec: "libopus", bitrate: Some("48k") },
    Profile { name: "mid", ext: "mp3", codec: "libmp3lame", bitrate: Some("160k") },
    Profile { name: "high", ext: "mp3", codec: "libmp3lame", bitrate: Some("320k") },
    Profile { name: "karaoke", ext: "flac", codec: "flac", bitrate: None },
    Profile { name: "preview", ext: "ogg", codec: "libopus", bitrate: Some("96k") },
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Profiles => {
            println!("{:<10} | {:<8} | {:<8} | {}", "Profile", "Bitrate", "Format", "Method");
            println!("{:-<70}", "");
            for p in PROFILES {
                let method = if p.name == "karaoke" { "AI Separation" } else { "FFmpeg" };
                println!("{:<10} | {:<8} | {:<8} | {}", p.name, p.bitrate.unwrap_or("Lossless"), p.ext, method);
            }
        }

        Commands::Convert { input, quality, all, fast_karaoke } => {
            let file = match input {
                Some(f) => f,
                None => Text::new("Path to audio file?").prompt()?,
            };

            if all {
                for p in PROFILES {
                    let _ = process_file(&file, p.name, true, fast_karaoke).await;
                }
            } else {
                let q = match quality {
                    Some(q) => q,
                    None => Select::new("Select Quality:", PROFILES.iter().map(|p| p.name).collect()).prompt()?.to_string(),
                };
                process_file(&file, &q, true, fast_karaoke).await?;
            }
        }

        Commands::Batch { dir, quality, fast_karaoke } => {
            let files: Vec<PathBuf> = WalkDir::new(dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| is_audio_file(e.path()))
                .map(|e| e.path().to_owned())
                .collect();

            let pb = ProgressBar::new(files.len() as u64);
            pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?);

            for file in files {
                let _ = process_file(file.to_str().unwrap(), &quality, false, fast_karaoke).await;
                pb.inc(1);
            }
            pb.finish_with_message("Batch complete!");
        }
    }
    Ok(())
}

async fn process_file(input: &str, quality_name: &str, verbose: bool, fast_karaoke: bool) -> anyhow::Result<()> {
    let profile = PROFILES.iter().find(|p| p.name == quality_name).ok_or_else(|| anyhow::anyhow!("Invalid profile"))?;
    let path = Path::new(input);
    let stem = path.file_stem().unwrap().to_str().unwrap();
    let final_output = format!("{}_{}.{}", stem, profile.name, profile.ext);

    // ─── KARAOKE LOGIC ───
    if quality_name == "karaoke" {
        if fast_karaoke {
            if verbose {
                println!("⚡ Using Advanced Phase-Cancellation Karaoke Filter (Strong vocals ↓ | Bass/Kick preserved)...");
            }
            
            let filter_string = "\
            asplit[a][b]; \
            [a]lowpass=f=200[low]; \
            [b]highpass=f=200,pan=mono|c0=c0-c1[high]; \
            [low][high]amix=inputs=2:weights=1 -0.8,volume=1.25";
            
            let status = Command::new("ffmpeg")
                .arg("-i").arg(&input)
                .arg("-af").arg(filter_string)
                .arg("-c:a").arg("flac")
                .arg("-y")
                .arg("-hide_banner")
                .arg("-loglevel").arg("error")
                .arg(&final_output)
                .status()?;
            
            if verbose {
                if status.success() {
                    println!("✅ Karaoke version saved → {}", final_output);
                } else {
                    println!("❌ FFmpeg failed with status: {:?}", status);
                }
            }

        } else {
            // OPTION B: AI Vocal Removal (High Quality, 8GB RAM Safe)
            if verbose { println!("🧠 Running AI Separation (Model: MDX-Net)..."); }
            let ai_status = Command::new("audio-separator")
                .arg(input)
                .arg("--model_name").arg("UVR-MDX-NET-Inst_HQ_3") 
                .arg("--output_format").arg("FLAC")
                .arg("--output_dir").arg(".")
                .arg("--instrumental_only")
                .status()?;

            if verbose && ai_status.success() { 
                println!("✅ AI Karaoke finished. (Note: Check for Instrumental file in folder)"); 
            }
        }
    } else {
        // ─── STANDARD CONVERSION (MP3/OGG/PREVIEW) ───
        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-i").arg(input).arg("-y").arg("-hide_banner").arg("-loglevel").arg("error");

        if profile.name == "preview" { cmd.arg("-t").arg("25"); }
        cmd.arg("-c:a").arg(profile.codec);
        if let Some(br) = profile.bitrate { cmd.arg("-b:a").arg(br); }
        cmd.arg(&final_output);

        if verbose { println!("🚀 Converting to {}...", quality_name); }
        let status = cmd.status()?;
        if verbose && status.success() { println!("✅ Saved to {}", final_output); }
    }
    
    Ok(())
}

fn is_audio_file(path: &Path) -> bool {
    let exts = ["mp3", "wav", "flac", "ogg", "m4a", "aac"];
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| exts.contains(&s.to_lowercase().as_str()))
        .unwrap_or(false)
}
