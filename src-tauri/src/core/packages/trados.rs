// core/packages/trados.rs — Trados Studio Package (.sdlppx / .sdlrpx) Engine
// Fully extracts Trados project packages and builds return packages with zero licensing dependencies.

use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use anyhow::{Result, Context};
use zip::{ZipArchive, ZipWriter, write::FileOptions};
use serde::{Deserialize, Serialize};
use quick_xml::reader::Reader;
use quick_xml::events::Event;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdlppxInfo {
    pub project_name: String,
    pub source_lang: String,
    pub target_lang: String,
    pub sdlxliff_files: Vec<PathBuf>,
    pub sdltm_files: Vec<PathBuf>,
}

/// Unpack a .sdlppx ZIP archive into extract_dir, extract metadata from project.sdlproj,
/// and return lists of translatable .sdlxliff files and embedded .sdltm databases.
pub fn unpack_sdlppx(package_path: &Path, extract_dir: &Path) -> Result<SdlppxInfo> {
    let file = std::fs::File::open(package_path)
        .with_context(|| format!("Failed to open .sdlppx file: {:?}", package_path))?;
    let mut zip = ZipArchive::new(file)
        .with_context(|| "Failed to read .sdlppx as ZIP archive")?;

    std::fs::create_dir_all(extract_dir)?;

    let mut sdlxliff_files = Vec::new();
    let mut sdltm_files = Vec::new();
    let mut sdlproj_content = String::new();

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let name = entry.name().to_string();
        let out_path = extract_dir.join(&name);

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut out_file = std::fs::File::create(&out_path)?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        out_file.write_all(&buf)?;

        let lower_name = name.to_lowercase();
        if lower_name.ends_with(".sdlxliff") {
            sdlxliff_files.push(out_path.clone());
        } else if lower_name.ends_with(".sdltm") {
            sdltm_files.push(out_path.clone());
        } else if lower_name.ends_with(".sdlproj") {
            sdlproj_content = String::from_utf8_lossy(&buf).to_string();
        }
    }

    let (project_name, source_lang, target_lang) = if !sdlproj_content.is_empty() {
        parse_sdlproj(&sdlproj_content)
    } else {
        (
            package_path.file_stem().and_then(|s| s.to_str()).unwrap_or("Trados Project").to_string(),
            "en-US".to_string(),
            "target".to_string(),
        )
    };

    Ok(SdlppxInfo {
        project_name,
        source_lang,
        target_lang,
        sdlxliff_files,
        sdltm_files,
    })
}

/// Parse project name and language pair from project.sdlproj XML manifest
fn parse_sdlproj(xml: &str) -> (String, String, String) {
    let mut reader = Reader::from_str(xml);
    let mut project_name = String::from("Trados Project");
    let mut source_lang = String::from("en-US");
    let mut target_lang = String::from("target");

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"Project" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Name" {
                            if let Ok(v) = attr.unescape_value() {
                                project_name = v.to_string();
                            }
                        }
                    }
                } else if name.as_ref() == b"SourceLanguageCode" {
                    if let Ok(Event::Text(ref t)) = reader.read_event() {
                        if let Ok(txt) = t.unescape() {
                            source_lang = txt.to_string();
                        }
                    }
                } else if name.as_ref() == b"TargetLanguageCode" {
                    if let Ok(Event::Text(ref t)) = reader.read_event() {
                        if let Ok(txt) = t.unescape() {
                            target_lang = txt.to_string();
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
    }

    (project_name, source_lang, target_lang)
}

/// Create a .sdlrpx return package archive containing updated target .sdlxliff files.
pub fn create_sdlrpx_package(
    base_dir: &Path,
    sdlxliff_relative_paths: &[PathBuf],
    output_sdlrpx_path: &Path,
) -> Result<()> {
    let out_file = std::fs::File::create(output_sdlrpx_path)
        .with_context(|| format!("Failed to create .sdlrpx file: {:?}", output_sdlrpx_path))?;
    let mut zip = ZipWriter::new(out_file);

    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for rel_path in sdlxliff_relative_paths {
        let abs_path = base_dir.join(rel_path);
        if !abs_path.exists() {
            continue;
        }

        let zip_entry_name = rel_path.to_string_lossy().replace('\\', "/");
        zip.start_file(zip_entry_name, options)?;

        let mut f = std::fs::File::open(&abs_path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        zip.write_all(&buf)?;
    }

    zip.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sdlproj_metadata() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Project Name="Legal Contract Review 2026">
  <SourceLanguageCode>en-US</SourceLanguageCode>
  <TargetLanguageCode>ar-SA</TargetLanguageCode>
</Project>"#;

        let (name, src, tgt) = parse_sdlproj(xml);
        assert_eq!(name, "Legal Contract Review 2026");
        assert_eq!(src, "en-US");
        assert_eq!(tgt, "ar-SA");
    }
}
