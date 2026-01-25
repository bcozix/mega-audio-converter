use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Select, Text};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "audioconv", version = "1.0", about = "Audio Converter 2026 - MP3 Edition")]
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
        /// Use AI Vocal Removal (Slow, requires audio-separator)
        #[arg(long)]
        ai_karaoke: bool,
    },
    /// Batch convert a directory
    Batch {
        #[arg(short, long)]
        dir: String,
        #[arg(short, long, default_value = "all")]
        quality: String,
        /// Use AI Vocal Removal (Slow, requires audio-separator)
        #[arg(long)]
        ai_karaoke: bool,
    },
}

struct Profile {
    name: &'static str,
    ext: &'static str,
    codec: &'static str,
    bitrate: Option<&'static str>,
}

const PROFILES: &[Profile] = &[
    Profile { name: "low", ext: "mp3", codec: "libmp3lame", bitrate: Some("64k") },
    Profile { name: "mid", ext: "mp3", codec: "libmp3lame", bitrate: Some("160k") },
    Profile { name: "high", ext: "mp3", codec: "libmp3lame", bitrate: Some("320k") },
    Profile { name: "karaoke", ext: "mp3", codec: "libmp3lame", bitrate: Some("320k") },
    Profile { name: "preview", ext: "mp3", codec: "libmp3lame", bitrate: Some("96k") },
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Profiles => {
            println!("{:<10} | {:<8} | {:<8} | {}", "Profile", "Bitrate", "Format", "Default Method");
            println!("{:-<75}", "");
            for p in PROFILES {
                let method = if p.name == "karaoke" { "FFmpeg Filter" } else { "FFmpeg" };
                println!("{:<10} | {:<8} | {:<8} | {}", p.name, p.bitrate.unwrap_or("VBR"), p.ext, method);
            }
        }

        Commands::Convert { input, quality, all, ai_karaoke } => {
            let file = match input {
                Some(f) => f,
                None => Text::new("Path to audio file?").prompt()?,
            };
            let q_name = if all { "all".to_string() } else {
                match quality {
                    Some(q) => q,
                    None => Select::new("Select Quality:", PROFILES.iter().map(|p| p.name).collect()).prompt()?.to_string(),
                }
            };
            process_song(&file, &q_name, true, ai_karaoke).await?;
        }

        Commands::Batch { dir, quality, ai_karaoke } => {
            let files: Vec<PathBuf> = WalkDir::new(dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| is_audio_file(e.path()))
                .map(|e| e.path().to_owned())
                .collect();

            let pb = ProgressBar::new(files.len() as u64);
            pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")?);

            for file in files {
                let file_str = file.to_str().unwrap();
                pb.set_message(format!("Processing: {}", file.file_name().unwrap().to_str().unwrap()));
                if let Err(e) = process_song(file_str, &quality, false, ai_karaoke).await {
                    eprintln!("\nError processing {}: {}", file_str, e);
                }
                pb.inc(1);
            }
            pb.finish_with_message("Batch complete!");
        }
    }
    Ok(())
}

async fn process_song(input: &str, quality_request: &str, verbose: bool, ai_karaoke: bool) -> anyhow::Result<()> {
    let input_path = fs::canonicalize(input)?;
    let stem = input_path.file_stem().unwrap().to_str().unwrap();
    let output_folder = PathBuf::from("output").join(stem);
    fs::create_dir_all(&output_folder)?;

    if quality_request == "all" {
        for p in PROFILES {
            let _ = convert_file(&input_path, p.name, &output_folder, verbose, ai_karaoke).await;
        }
    } else {
        convert_file(&input_path, quality_request, &output_folder, verbose, ai_karaoke).await?;
    }
    Ok(())
}

async fn convert_file(input_path: &Path, quality_name: &str, out_dir: &Path, verbose: bool, ai_karaoke: bool) -> anyhow::Result<()> {
    let profile = PROFILES.iter().find(|p| p.name == quality_name).ok_or_else(|| anyhow::anyhow!("Invalid profile"))?;
    let stem = input_path.file_stem().unwrap().to_str().unwrap();
    let final_output = out_dir.join(format!("{}_{}.mp3", stem, profile.name));

    if quality_name == "karaoke" {
        if ai_karaoke {
            if verbose { println!("🧠 Running AI Separation (MP3 Mode) for {}...", stem); }
            let status = Command::new("audio-separator")
                .arg(input_path)
                .arg("--model_name").arg("UVR-MDX-NET-Inst_HQ_3") 
                .arg("--output_format").arg("MP3")
                .arg("--output_dir").arg(out_dir)
                .arg("--instrumental_only")
                .status()?;

            if status.success() {
                // Find the file generated by audio-separator (it usually appends model info to the name)
                for entry in fs::read_dir(out_dir)? {
                    let path = entry?.path();
                    let name = path.file_name().unwrap().to_str().unwrap();
                    // Identify the new instrumental file that isn't our final renamed file yet
                    if name.ends_with(".mp3") && !name.contains(&format!("{}_karaoke.mp3", stem)) {
                        fs::rename(path, &final_output)?;
                        break;
                    }
                }
            }
        } else {
            if verbose { println!("⚡ Using Fast Phase-Cancellation (FFmpeg MP3) for {}...", stem); }
            let filter = "asplit[a][b]; [a]lowpass=f=200[low]; [b]highpass=f=200,pan=mono|c0=c0-c1[high]; [low][high]amix=inputs=2:weights=1 -0.8,volume=1.25";
            Command::new("ffmpeg")
                .arg("-y")
                .arg("-i").arg(input_path)
                .arg("-af").arg(filter)
                .arg("-c:a").arg("libmp3lame")
                .arg("-b:a").arg("320k")
                .arg("-hide_banner").arg("-loglevel").arg("error")
                .arg(&final_output)
                .status()?;
        }
    } else {
        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-y").arg("-i").arg(input_path);

        if profile.name == "preview" { cmd.arg("-t").arg("40"); }

        cmd.arg("-c:a").arg(profile.codec);
        if let Some(br) = profile.bitrate { 
            cmd.arg("-b:a").arg(br); 
        }

        cmd.arg("-hide_banner").arg("-loglevel").arg("error").arg(&final_output);

        if verbose { println!("🚀 Generating {} MP3 version...", quality_name); }
        cmd.status()?;
    }
    Ok(())
}

fn is_audio_file(path: &Path) -> bool {
    let exts = ["mp3", "wav", "flac", "ogg", "m4a", "aac"];
    path.extension().and_then(|s| s.to_str()).map(|s| exts.contains(&s.to_lowercase().as_str())).unwrap_or(false)
}