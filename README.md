# mega-audio-converter

## Install audio_converter 1.0.2

### Install prebuilt binaries via shell script

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/bcozix/mega-audio-converter/releases/download/1.0.2/audio_converter-installer.sh | sh
```

### Install prebuilt binaries via powershell script

```sh
powershell -ExecutionPolicy Bypass -c "irm https://github.com/bcozix/mega-audio-converter/releases/download/1.0.2/audio_converter-installer.ps1 | iex"
```

### Install prebuilt binaries via Homebrew

```sh
brew install bcozix/mega-audio-converter/audio_converter
```

### Install prebuilt binaries into your npm project

```sh
npm install @bcozix/audio_converter@1.0.2
```

## Usage
```sh
cargo run -- convert -i song.wav -q high
```

```sh
cargo run -- convert -i song.wav -a
```

# Compiles with optimizations, then runs
```sh
cargo run --release -- convert -i song.wav -a
```

```sh
cargo run -- convert -i song.mp3 -q karaoke
```

```sh
cargo run -- convert -i audio_5b39782e-22f5-4ae5-860a-65f429446304.mp3 -q karaoke --fast-karaoke
```

```sh
cargo run -- batch -d ./input_songs #folder name
```