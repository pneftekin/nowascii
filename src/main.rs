use std::{collections::HashMap, env, error::Error, io::{self, Read, Write}, time::Duration};
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use zbus::blocking::{Connection, fdo};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() {
    if let Err(error) = run() {
        eprintln!("nowascii: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let width = parse_width()?;
    let connection = Connection::session()?;
    let players = fdo::DBusProxy::new(&connection)?.list_names()?;
    let mut candidates: Vec<_> = players.iter().filter_map(|name| {
        let value = name.as_str();
        value.starts_with("org.mpris.MediaPlayer2.").then_some(value.to_owned())
    }).collect();
    candidates.sort();
    if candidates.is_empty() { return Err("no MPRIS players found; start music in an MPRIS-compatible player".into()); }

    let mut selected = None;
    let mut diagnostics = Vec::new();
    for name in &candidates {
        let properties = match fdo::PropertiesProxy::builder(&connection)
            .destination(name.as_str())?
            .path("/org/mpris/MediaPlayer2")?
            .build()
        {
            Ok(proxy) => proxy,
            Err(error) => { diagnostics.push(format!("{name}: {error}")); continue; }
        };
        let interface = zbus::names::InterfaceName::try_from("org.mpris.MediaPlayer2.Player")
            .expect("MPRIS interface name is valid");
        let status = properties.get(interface.clone(), "PlaybackStatus")
            .map_err(|e| e.to_string())
            .and_then(|value| String::try_from(value).map_err(|e| e.to_string()));
        let metadata = properties.get(interface, "Metadata")
            .map_err(|e| e.to_string())
            .and_then(|value| HashMap::<String, zbus::zvariant::OwnedValue>::try_from(value).map_err(|e| e.to_string()));
        match (status, metadata) {
            (Ok(status), Ok(metadata)) => {
                if status == "Playing" { selected = Some(metadata); break; }
                if selected.is_none() { selected = Some(metadata); }
            }
            (status, metadata) => diagnostics.push(format!("{name}: PlaybackStatus={}, Metadata={}",
                status.err().unwrap_or_else(|| "ok".into()),
                metadata.err().unwrap_or_else(|| "ok".into()))),
        }
    }
    let metadata = selected.ok_or_else(|| format!("could not read metadata from any MPRIS player\n{}", diagnostics.join("\n")))?;
    let art_url = string_field(&metadata, "mpris:artUrl").ok_or("the current track has no album art (MPRIS mpris:artUrl is missing)")?;
    let bytes = load_art(&art_url)?;
    let image = image::load_from_memory(&bytes)?;
    render(&image, width)?;
    Ok(())
}

fn parse_width() -> Result<u32> {
    let mut args = env::args().skip(1);
    let mut width = 32u32;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => { println!("nowascii [--width N]\n\nShow the current MPRIS album cover as colored terminal art.\n\nOptions:\n  -w, --width N  Output width in terminal cells (default: 32)\n  -h, --help     Show this help"); std::process::exit(0); }
            "-w" | "--width" => width = args.next().ok_or("--width requires a number")?.parse()?,
            _ if arg.starts_with("--width=") => width = arg[8..].parse()?,
            _ => return Err(format!("unknown option: {arg} (try --help)").into()),
        }
    }
    if !(4..=200).contains(&width) { return Err("width must be between 4 and 200".into()); }
    Ok(width)
}

fn string_field(map: &HashMap<String, zbus::zvariant::OwnedValue>, key: &str) -> Option<String> {
    map.get(key).and_then(|v| String::try_from(v.clone()).ok())
}

fn load_art(url: &str) -> Result<Vec<u8>> {
    const LIMIT: u64 = 25 * 1024 * 1024;
    let bytes = if let Some(path) = url.strip_prefix("file://") {
        let path = percent_decode(path)?;
        std::fs::read(path)?
    } else if url.starts_with("https://") || url.starts_with("http://") {
        let response = ureq::AgentBuilder::new().timeout(Duration::from_secs(15)).build().get(url).call()?;
        let len = response.header("Content-Length").and_then(|s| s.parse::<u64>().ok());
        if len.is_some_and(|n| n > LIMIT) { return Err("album art is larger than 25 MiB".into()); }
        let mut bytes = Vec::new();
        response.into_reader().take(LIMIT + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > LIMIT { return Err("album art is larger than 25 MiB".into()); }
        bytes
    } else { return Err(format!("unsupported album art URL scheme: {url}").into()); };
    Ok(bytes)
}

fn percent_decode(input: &str) -> Result<String> {
    let bytes = input.as_bytes(); let mut out = Vec::with_capacity(bytes.len()); let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() { return Err("invalid percent escape in album art path".into()); }
            let hex = std::str::from_utf8(&bytes[i+1..i+3])?;
            out.push(u8::from_str_radix(hex, 16)?); i += 3;
        } else { out.push(bytes[i]); i += 1; }
    }
    Ok(String::from_utf8(out)?)
}

fn render(image: &DynamicImage, width: u32) -> Result<()> {
    // Most terminal cells are roughly twice as tall as they are wide. Use half
    // as many character rows as columns so the result looks square on screen.
    let target_w = width;
    let target_h = (width / 2).max(2);
    let (image_w, image_h) = image.dimensions();
    let side = image_w.min(image_h);
    let left = (image_w - side) / 2;
    let top = (image_h - side) / 2;
    let square = image.crop_imm(left, top, side, side);
    let resized = square.resize_exact(target_w, target_h, FilterType::Lanczos3).to_rgb8();
    const RAMP: &[u8] = b"@%#*+=-:.";
    let mut out = String::new();
    for y in 0..target_h {
        for x in 0..target_w {
            let pixel = resized.get_pixel(x, y).0;
            let luminance = (299 * u32::from(pixel[0]) + 587 * u32::from(pixel[1]) + 114 * u32::from(pixel[2])) / 1000;
            let index = (luminance as usize * (RAMP.len() - 1)) / 255;
            out.push_str(&format!("\x1b[38;2;{};{};{}m{}", pixel[0], pixel[1], pixel[2], RAMP[index] as char));
        }
        out.push_str("\x1b[0m\n");
    }
    print!("{out}"); io::stdout().flush()?;
    Ok(())
}

