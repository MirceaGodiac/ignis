//! `ignis sources media …`: R11 media indexes from byte ranges (dry run unless `--execute`).

use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::Subcommand;
use ignis_core::contracts::{MediaKeyframeIndex, Origin, SourceId};
use regex::Regex;

use crate::media::range::{FileRange, HttpRange, RangeRead};
use crate::media::{self, frames, ts, zipdir};
use crate::sources::psi::RemoteFile;

/// Pause between geode calls (the endpoint is undocumented; stay polite).
const GEODE_PAUSE: Duration = Duration::from_millis(500);
const TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Subcommand, Debug)]
pub enum MediaCmd {
    /// Keyframe indexes of `.ts` recordings for M12's HLS playlists, read by byte range at NASA
    /// (or from a local file with `--file`). Written to `<IGNIS_DATA>/media/keyframes/`.
    Keyframes {
        /// Investigations (default: all 19).
        investigations: Vec<String>,
        /// Regex on file name.
        #[arg(long, default_value = ".*")]
        pattern: String,
        /// At most this many recordings (smallest first when sampling a few).
        #[arg(long)]
        limit: Option<usize>,
        /// Seconds of presentation time between probes; 0 = every keyframe (reads the whole file).
        #[arg(long, default_value_t = 4.0)]
        every_s: f64,
        /// Index this local copy instead of reading NASA (needs `--source-id`).
        #[arg(long)]
        file: Option<PathBuf>,
        #[arg(long)]
        source_id: Option<String>,
        /// Re-index recordings that already have a valid index for their size.
        #[arg(long)]
        force: bool,
        /// Read from NASA and write indexes (default: print the plan).
        #[arg(long)]
        execute: bool,
    },
    /// Central directories of remote ZIP archives, cached under `<IGNIS_DATA>/media/zipdir/`.
    Zipdir {
        investigations: Vec<String>,
        #[arg(long, default_value = ".*")]
        pattern: String,
        #[arg(long)]
        limit: Option<usize>,
        /// Print every entry.
        #[arg(long)]
        list: bool,
        #[arg(long)]
        execute: bool,
    },
    /// One entry of a remote ZIP archive, without the archive; TIFF frames converted on demand.
    Entry {
        investigation: String,
        /// File name of the archive in the PSI listing.
        archive: String,
        /// Entry name inside the archive.
        name: String,
        /// png | webp | original
        #[arg(long, default_value = "png")]
        format: String,
        /// Output file.
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        execute: bool,
    },
}

fn listing(invs: &[String], ext: &[&str], pattern: &str) -> anyhow::Result<Vec<RemoteFile>> {
    let re = Regex::new(pattern)?;
    let mut out = Vec::new();
    for inv in invs {
        let p = crate::paths::manifests().join(format!("{inv}.listing.jsonl"));
        if !p.exists() {
            continue;
        }
        for f in crate::sources::psi::read_listing(&p)? {
            let lower = f.file_name.to_ascii_lowercase();
            if !f.restricted && ext.iter().any(|e| lower.ends_with(e)) && re.is_match(&f.file_name)
            {
                out.push(f);
            }
        }
    }
    Ok(out)
}

fn pick(mut files: Vec<RemoteFile>, limit: Option<usize>) -> Vec<RemoteFile> {
    if let Some(n) = limit {
        files.sort_by_key(|f| f.size_bytes.unwrap_or(i64::MAX));
        files.truncate(n);
    }
    files
}

fn mb(b: u64) -> f64 {
    b as f64 / 1e6
}

fn existing(path: &std::path::Path, size: u64) -> bool {
    std::fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice::<MediaKeyframeIndex>(&b).ok())
        .is_some_and(|ix| ix.size_bytes == size && ix.validate().is_ok())
}

fn write_index(ix: &MediaKeyframeIndex) -> anyhow::Result<PathBuf> {
    let dir = media::keyframes_dir();
    std::fs::create_dir_all(&dir)?;
    let p = dir.join(media::index_file_name(ix.source_id.as_str()));
    let mut text = serde_json::to_string_pretty(ix)?;
    text.push('\n');
    zipdir::write_atomic(&p, text.as_bytes())?;
    Ok(p)
}

fn report(name: &str, ix: &MediaKeyframeIndex, req: u64, secs: f64) {
    println!(
        "  {name}: {} keyframes ({:?}), {:.1} s, {:?}, read {:.2} MB of {:.1} MB ({:.2} %) in {req} requests, {secs:.1} s",
        ix.keyframes.len(),
        ix.coverage,
        ix.duration_s(),
        ix.codec,
        mb(ix.bytes_read),
        mb(ix.size_bytes),
        100.0 * ix.bytes_read as f64 / ix.size_bytes.max(1) as f64,
    );
}

pub fn run(cmd: MediaCmd, all: Vec<String>) -> anyhow::Result<()> {
    let invs = |v: Vec<String>| if v.is_empty() { all.clone() } else { v };
    match cmd {
        MediaCmd::Keyframes {
            investigations,
            pattern,
            limit,
            every_s,
            file,
            source_id,
            force,
            execute,
        } => {
            let opts = ts::TsOptions {
                sample_every_s: (every_s > 0.0).then_some(every_s),
                ..Default::default()
            };
            if let Some(path) = file {
                let id = source_id.ok_or_else(|| anyhow::anyhow!("--file needs --source-id"))?;
                let r = FileRange::open(&path)?;
                let t = Instant::now();
                // A local copy is a measurement of NASA's file, not a fixture.
                let ix = ts::index(&r, SourceId::new(id)?, opts, Origin::System)?;
                let p = write_index(&ix)?;
                report(
                    &path.display().to_string(),
                    &ix,
                    r.meter().requests(),
                    t.elapsed().as_secs_f64(),
                );
                println!("  -> {}", p.display());
                return Ok(());
            }
            let files = pick(listing(&invs(investigations), &[".ts"], &pattern)?, limit);
            let total: i64 = files.iter().filter_map(|f| f.size_bytes).sum();
            println!(
                "{} .ts recordings, {:.1} GB at NASA; index every {} s of PTS (byte ranges only) -> {}",
                files.len(),
                total as f64 / 1e9,
                every_s,
                media::keyframes_dir().display()
            );
            if !execute {
                for f in files.iter().take(20) {
                    println!(
                        "  {} {:.1} MB",
                        f.file_name,
                        mb(f.size_bytes.unwrap_or(0) as u64)
                    );
                }
                println!("dry run: add --execute to read the byte ranges and write the indexes");
                return Ok(());
            }
            let (mut done, mut skipped, mut failed, mut read) = (0, 0, 0, 0u64);
            for f in &files {
                let id = f.listing_source_id();
                let Some(size) = f.size_bytes.map(|s| s as u64) else {
                    println!("  {}: no size in the listing, skipped", f.file_name);
                    failed += 1;
                    continue;
                };
                let p = media::keyframes_dir().join(media::index_file_name(&id));
                if !force && existing(&p, size) {
                    skipped += 1;
                    continue;
                }
                let t = Instant::now();
                let res = (|| {
                    let url = media::sign(f, TIMEOUT)?;
                    std::thread::sleep(GEODE_PAUSE);
                    let r = HttpRange::open(&url, Some(size), TIMEOUT)?;
                    let ix = ts::index(&r, SourceId::new(id.clone())?, opts, Origin::System)?;
                    write_index(&ix)?;
                    anyhow::Ok((ix, r.meter().requests()))
                })();
                match res {
                    Ok((ix, req)) => {
                        report(&f.file_name, &ix, req, t.elapsed().as_secs_f64());
                        read += ix.bytes_read;
                        done += 1;
                    }
                    Err(e) => {
                        println!("  {}: FAILED {e:#}", f.file_name);
                        failed += 1;
                    }
                }
            }
            println!(
                "indexed {done}, already current {skipped}, failed {failed}; read {:.1} MB in total",
                mb(read)
            );
            anyhow::ensure!(failed == 0, "{failed} recordings were not indexed");
            Ok(())
        }
        MediaCmd::Zipdir {
            investigations,
            pattern,
            limit,
            list,
            execute,
        } => {
            let files = pick(listing(&invs(investigations), &[".zip"], &pattern)?, limit);
            let total: i64 = files.iter().filter_map(|f| f.size_bytes).sum();
            let cache = zipdir::DirCache::new(media::cache_dir().join("zipdir"));
            println!(
                "{} ZIP archives, {:.1} GB at NASA; central directories -> {}",
                files.len(),
                total as f64 / 1e9,
                media::cache_dir().join("zipdir").display()
            );
            if !execute {
                for f in files.iter().take(20) {
                    println!(
                        "  {} {:.1} MB",
                        f.file_name,
                        mb(f.size_bytes.unwrap_or(0) as u64)
                    );
                }
                println!("dry run: add --execute to read the central directories");
                return Ok(());
            }
            let mut failed = 0;
            for f in &files {
                let id = f.listing_source_id();
                let t = Instant::now();
                let res = (|| {
                    let size = f.size_bytes.map(|s| s as u64);
                    if let Some(d) = size.and_then(|s| cache.load(&id, s).transpose()) {
                        return anyhow::Ok((d?, 0, 0));
                    }
                    let url = media::sign(f, TIMEOUT)?;
                    std::thread::sleep(GEODE_PAUSE);
                    let r = HttpRange::open(&url, size, TIMEOUT)?;
                    let d = cache.get(&id, &r)?;
                    anyhow::Ok((d, r.meter().requests(), r.meter().bytes()))
                })();
                match res {
                    Ok((d, req, bytes)) => {
                        let tiffs = d
                            .entries
                            .iter()
                            .filter(|e| frames::is_tiff(&e.name))
                            .count();
                        println!(
                            "  {}: {} entries ({tiffs} TIFF), directory {:.2} MB; read {:.2} MB in {req} requests, {:.1} s",
                            f.file_name,
                            d.entries.len(),
                            mb(d.raw.len() as u64),
                            mb(bytes),
                            t.elapsed().as_secs_f64()
                        );
                        if list {
                            for e in &d.entries {
                                println!("    {} {} B (method {})", e.name, e.size, e.method);
                            }
                        }
                    }
                    Err(e) => {
                        println!("  {}: FAILED {e:#}", f.file_name);
                        failed += 1;
                    }
                }
            }
            anyhow::ensure!(failed == 0, "{failed} archives were not read");
            Ok(())
        }
        MediaCmd::Entry {
            investigation,
            archive,
            name,
            format,
            out,
            execute,
        } => {
            let fmt = match format.as_str() {
                "original" => None,
                f => Some(
                    frames::FrameFormat::parse(f)
                        .ok_or_else(|| anyhow::anyhow!("--format is png, webp or original"))?,
                ),
            };
            let f = listing(
                &[investigation.clone()],
                &[".zip"],
                &format!("^{}$", regex::escape(&archive)),
            )?
            .into_iter()
            .next()
            .ok_or_else(|| {
                anyhow::anyhow!("{archive} is not a ZIP in the {investigation} listing")
            })?;
            if !execute {
                println!(
                    "would read {name} from {} ({:.1} MB) by byte range -> {}",
                    f.file_name,
                    mb(f.size_bytes.unwrap_or(0) as u64),
                    out.display()
                );
                return Ok(());
            }
            let id = f.listing_source_id();
            let url = media::sign(&f, TIMEOUT)?;
            let r = HttpRange::open(&url, f.size_bytes.map(|s| s as u64), TIMEOUT)?;
            let cache = zipdir::DirCache::new(media::cache_dir().join("zipdir"));
            let d = cache.get(&id, &r)?;
            let e = d
                .entry(&name)
                .ok_or_else(|| anyhow::anyhow!("{name} is not an entry of {archive}"))?;
            let bytes = zipdir::read_entry(&r, e)?;
            let (body, cached) = match fmt {
                Some(fm) if frames::is_tiff(&e.name) => {
                    frames::FrameCache::new(media::cache_dir().join("frames")).get(&bytes, fm)?
                }
                Some(_) => anyhow::bail!("{name} is not a TIFF; use --format original"),
                None => (bytes, false),
            };
            std::fs::write(&out, &body)?;
            println!(
                "{name}: {} B -> {} ({} B{}); read {:.2} MB of the {:.1} MB archive in {} requests",
                e.size,
                out.display(),
                body.len(),
                if cached { ", from the frame cache" } else { "" },
                mb(r.meter().bytes()),
                mb(r.size()),
                r.meter().requests()
            );
            Ok(())
        }
    }
}
