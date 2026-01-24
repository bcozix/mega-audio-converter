// use clap::{Parser, Subcommand};
// use indicatif::{ProgressBar, ProgressStyle};
// use inquire::{Select, Text};
// use std::path::{Path, PathBuf};
// use std::process::Command;
// use std::sync::Arc;
// use walkdir::WalkDir;

// #[derive(Parser)]
// #[command(name = "audioconv", version = "1.0", about = "AI Audio Converter 2026")]
// struct Cli {
//     #[command(subcommand)]
//     command: Commands,
// }

// #[derive(Subcommand)]
// enum Commands {
//     /// List available quality profiles
//     Profiles,
//     /// Convert a single file
//     Convert {
//         #[arg(short, long)]
//         input: Option<String>,
//         #[arg(short, long)]
//         quality: Option<String>,
//         #[arg(short, long)]
//         all: bool,
//     },
//     /// Batch convert a directory
//     Batch {
//         #[arg(short, long)]
//         dir: String,
//         #[arg(short, long)]
//         quality: String,
//     },
// }

// struct Profile {
//     name: &'static str,
//     ext: &'static str,
//     codec: &'static str,
//     bitrate: Option<&'static str>,
// }

// const PROFILES: &[Profile] = &[
//     Profile { name: "low", ext: "ogg", codec: "libopus", bitrate: Some("48k") },
//     Profile { name: "mid", ext: "mp3", codec: "libmp3lame", bitrate: Some("160k") },
//     Profile { name: "high", ext: "mp3", codec: "libmp3lame", bitrate: Some("320k") },
//     Profile { name: "karaoke", ext: "flac", codec: "flac", bitrate: None },
//     Profile { name: "preview", ext: "ogg", codec: "libopus", bitrate: Some("96k") },
// ];

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     let cli = Cli::parse();

//     match cli.command {
//         Commands::Profiles => {
//             println!("{:<10} | {:<8} | {:<8} | {}", "Profile", "Bitrate", "Format", "Use Case");
//             println!("{:-<70}", "");
//             for p in PROFILES {
//                 let note = if p.name == "karaoke" { "(AI Vocal Removal)" } else { "" };
//                 println!("{:<10} | {:<8} | {:<8} | {} {}", p.name, p.bitrate.unwrap_or("Lossless"), p.ext, "Standard Tier", note);
//             }
//         }

//         Commands::Convert { input, quality, all } => {
//             let file = match input {
//                 Some(f) => f,
//                 None => Text::new("Path to audio file?").prompt()?,
//             };

//             if all {
//                 run_all_profiles(&file).await?;
//             } else {
//                 let q = match quality {
//                     Some(q) => q,
//                     None => Select::new("Select Quality:", PROFILES.iter().map(|p| p.name).collect()).prompt()?.to_string(),
//                 };
//                 process_file(&file, &q, true).await?;
//             }
//         }

//         Commands::Batch { dir, quality } => {
//             let files: Vec<PathBuf> = WalkDir::new(dir)
//                 .into_iter()
//                 .filter_map(|e| e.ok())
//                 .filter(|e| is_audio_file(e.path()))
//                 .map(|e| e.path().to_owned())
//                 .collect();

//             let pb = ProgressBar::new(files.len() as u64);
//             pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?);

//             for file in files {
//                 let _ = process_file(file.to_str().unwrap(), &quality, false).await;
//                 pb.inc(1);
//             }
//             pb.finish_with_message("Batch complete!");
//         }
//     }
//     Ok(())
// }

// async fn process_file(input: &str, quality_name: &str, verbose: bool) -> anyhow::Result<()> {
//     let profile = PROFILES.iter().find(|p| p.name == quality_name).ok_or_else(|| anyhow::anyhow!("Invalid profile"))?;
//     let path = Path::new(input);
//     let stem = path.file_stem().unwrap().to_str().unwrap();
//     let final_output = format!("{}_{}.{}", stem, profile.name, profile.ext);

//     if quality_name == "karaoke" {
//         if verbose { println!("🧠 Running AI Vocal Removal (this may take a few minutes on 8GB RAM)..."); }
        
//         // 1. Run AI Separator (Instrumental only)
//         // We use the 'MDX' model which is efficient for 8GB RAM in 2026
//         let ai_status = Command::new("audio-separator")
//             .arg(input)
//             .arg("--model_name").arg("UVR-MDX-NET-Inst_HQ_3") 
//             .arg("--output_format").arg("FLAC")
//             .arg("--output_dir").arg(".")
//             .arg("--instrumental_only")
//             .status()?;

//         if ai_status.success() {
//             // Note: audio-separator usually appends the model name to the output. 
//             // In a real app, you'd find the generated file and rename it to final_output.
//             if verbose { println!("✅ AI Separation complete. Karaoke track generated."); }
//         }
//     } else {
//         // Standard FFmpeg Conversion
//         let mut cmd = Command::new("ffmpeg");
//         cmd.arg("-i").arg(input).arg("-y").arg("-hide_banner").arg("-loglevel").arg("error");

//         if profile.name == "preview" { cmd.arg("-t").arg("25"); }
//         cmd.arg("-c:a").arg(profile.codec);
//         if let Some(br) = profile.bitrate { cmd.arg("-b:a").arg(br); }
//         cmd.arg(&final_output);

//         if verbose { println!("🚀 Converting to {}...", quality_name); }
//         let status = cmd.status()?;
//         if verbose && status.success() { println!("✅ Saved to {}", final_output); }
//     }
    
//     Ok(())
// }

// async fn run_all_profiles(input: &str) -> anyhow::Result<()> {
//     for p in PROFILES {
//         process_file(input, p.name, true).await?;
//     }
//     Ok(())
// }

// fn is_audio_file(path: &Path) -> bool {
//     let exts = ["mp3", "wav", "flac", "ogg", "m4a", "aac"];
//     path.extension()
//         .and_then(|s| s.to_str())
//         .map(|s| exts.contains(&s.to_lowercase().as_str()))
//         .unwrap_or(false)
// }
