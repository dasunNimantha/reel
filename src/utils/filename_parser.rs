use crate::model::{MediaType, ParsedMediaInfo};
use regex::Regex;

/// Parse a filename to extract media information
pub fn parse_filename(filename: &str) -> (MediaType, ParsedMediaInfo) {
    // Remove extension
    let name = filename
        .rsplit_once('.')
        .map(|(n, _)| n)
        .unwrap_or(filename);

    // Replace common separators with spaces
    let cleaned = name
        .replace('.', " ")
        .replace('_', " ")
        .replace('-', " ");

    let mut info = ParsedMediaInfo::default();

    // Try to detect TV show patterns - ordered by specificity (most specific first)
    
    // Pattern 1: S01E01, 1x01 - has both season and episode
    let season_episode_patterns = [
        r"(?i)[Ss](\d{1,2})[Ee](\d{1,2})",  // S01E01
        r"(?i)(\d{1,2})[xX](\d{1,2})",       // 1x01
        r"(?i)Season\s*(\d{1,2}).*Episode\s*(\d{1,2})", // Season 1 Episode 1
    ];

    for pattern in &season_episode_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(&cleaned) {
                if let (Some(season), Some(episode)) = (caps.get(1), caps.get(2)) {
                    info.season = season.as_str().parse().ok();
                    info.episode = episode.as_str().parse().ok();
                    
                    // Extract title (everything before the season/episode)
                    if let Some(m) = re.find(&cleaned) {
                        let title = cleaned[..m.start()].trim();
                        info.title = clean_title(title);
                        
                        // Try to get episode title (after S01E01)
                        let after = cleaned[m.end()..].trim();
                        if !after.is_empty() {
                            info.episode_title = Some(extract_episode_title(after));
                        }
                    }
                    
                    extract_quality_info(&cleaned, &mut info);
                    return (MediaType::TvShow, info);
                }
            }
        }
    }

    // Pattern 2: Episode X only (assume season 1)
    let episode_only_patterns = [
        r"(?i)Episode\s*(\d{1,3})",  // Episode 1, Episode 01
        r"(?i)\bEp\.?\s*(\d{1,3})",   // Ep 1, Ep.1, Ep01
        r"(?i)\bE(\d{1,3})\b",        // E01 (standalone)
    ];

    for pattern in &episode_only_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(&cleaned) {
                if let Some(episode) = caps.get(1) {
                    info.season = Some(1); // Default to season 1
                    info.episode = episode.as_str().parse().ok();
                    
                    // Extract title (everything before the episode marker)
                    if let Some(m) = re.find(&cleaned) {
                        let title = cleaned[..m.start()].trim();
                        info.title = clean_title(title);
                        
                        // Try to get episode title (after Episode X)
                        let after = cleaned[m.end()..].trim();
                        if !after.is_empty() {
                            info.episode_title = Some(extract_episode_title(after));
                        }
                    }
                    
                    extract_quality_info(&cleaned, &mut info);
                    return (MediaType::TvShow, info);
                }
            }
        }
    }

    // Try to detect movie with year
    if let Ok(re) = Regex::new(r"(.+?)[\s\.\-_]+\(?(\d{4})\)?") {
        if let Some(caps) = re.captures(&cleaned) {
            if let (Some(title), Some(year)) = (caps.get(1), caps.get(2)) {
                let year_num: u32 = year.as_str().parse().unwrap_or(0);
                // Sanity check year (movies from 1900-2030)
                if (1900..=2030).contains(&year_num) {
                    info.title = clean_title(title.as_str());
                    info.year = Some(year_num);
                    extract_quality_info(&cleaned, &mut info);
                    return (MediaType::Movie, info);
                }
            }
        }
    }

    // Fallback: treat as movie, use full name as title
    info.title = clean_title(&cleaned);
    extract_quality_info(&cleaned, &mut info);
    
    (MediaType::Unknown, info)
}

/// Clean up a title string
fn clean_title(title: &str) -> String {
    // Remove quality indicators, codec info, etc.
    let patterns = [
        r"(?i)\b(720p|1080p|2160p|4k|uhd)\b",
        r"(?i)\b(bluray|bdrip|brrip|webrip|web-dl|hdtv|dvdrip|hdrip)\b",
        r"(?i)\b(x264|x265|h264|h265|hevc|avc|xvid)\b",
        r"(?i)\b(aac|ac3|dts|dts-hd|atmos|truehd|flac|mp3)\b",
        r"(?i)\b(proper|repack|extended|unrated|directors cut)\b",
        r"(?i)\b(multi|dual|5\.1|7\.1)\b",
        r"\[.*?\]",  // Anything in brackets
        r"\(.*?\)",  // Anything in parentheses (except year which we already extracted)
    ];

    let mut cleaned = title.to_string();
    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            cleaned = re.replace_all(&cleaned, " ").to_string();
        }
    }

    // Clean up multiple spaces and trim
    let space_re = Regex::new(r"\s+").unwrap();
    space_re.replace_all(&cleaned, " ").trim().to_string()
}

/// Extract quality information from filename
fn extract_quality_info(text: &str, info: &mut ParsedMediaInfo) {
    // Quality (resolution)
    let quality_patterns = [
        (r"(?i)\b2160p\b", "2160p"),
        (r"(?i)\b4k\b", "4K"),
        (r"(?i)\buhd\b", "UHD"),
        (r"(?i)\b1080p\b", "1080p"),
        (r"(?i)\b720p\b", "720p"),
        (r"(?i)\b480p\b", "480p"),
    ];

    for (pattern, quality) in &quality_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(text) {
                info.quality = Some(quality.to_string());
                break;
            }
        }
    }

    // Source
    let source_patterns = [
        (r"(?i)\bbluray\b", "BluRay"),
        (r"(?i)\bbdrip\b", "BDRip"),
        (r"(?i)\bweb-?dl\b", "WEB-DL"),
        (r"(?i)\bwebrip\b", "WEBRip"),
        (r"(?i)\bhdtv\b", "HDTV"),
        (r"(?i)\bdvdrip\b", "DVDRip"),
    ];

    for (pattern, source) in &source_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(text) {
                info.source = Some(source.to_string());
                break;
            }
        }
    }

    // Codec
    let codec_patterns = [
        (r"(?i)\bx265\b", "x265"),
        (r"(?i)\bhevc\b", "HEVC"),
        (r"(?i)\bh\.?265\b", "H.265"),
        (r"(?i)\bx264\b", "x264"),
        (r"(?i)\bh\.?264\b", "H.264"),
        (r"(?i)\bavc\b", "AVC"),
    ];

    for (pattern, codec) in &codec_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(text) {
                info.codec = Some(codec.to_string());
                break;
            }
        }
    }

    // Audio
    let audio_patterns = [
        (r"(?i)\bdts-?hd\b", "DTS-HD"),
        (r"(?i)\batmos\b", "Atmos"),
        (r"(?i)\btruehd\b", "TrueHD"),
        (r"(?i)\bdts\b", "DTS"),
        (r"(?i)\bac3\b", "AC3"),
        (r"(?i)\baac\b", "AAC"),
        (r"(?i)\bflac\b", "FLAC"),
    ];

    for (pattern, audio) in &audio_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(text) {
                info.audio = Some(audio.to_string());
                break;
            }
        }
    }

    // Release group (usually at the end after a dash)
    if let Ok(re) = Regex::new(r"-([A-Za-z0-9]+)$") {
        if let Some(caps) = re.captures(text.trim()) {
            if let Some(group) = caps.get(1) {
                let group_name = group.as_str();
                // Filter out common false positives
                let exclude = ["720p", "1080p", "2160p", "x264", "x265", "HEVC", "AAC", "DTS"];
                if !exclude.iter().any(|&e| e.eq_ignore_ascii_case(group_name)) {
                    info.group = Some(group_name.to_string());
                }
            }
        }
    }
}

/// Extract episode title from text after S01E01
fn extract_episode_title(text: &str) -> String {
    clean_title(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tv_show() {
        let (media_type, info) = parse_filename("Breaking.Bad.S01E01.Pilot.720p.BluRay.x264.mkv");
        assert_eq!(media_type, MediaType::TvShow);
        assert_eq!(info.title, "Breaking Bad");
        assert_eq!(info.season, Some(1));
        assert_eq!(info.episode, Some(1));
        assert_eq!(info.quality, Some("720p".to_string()));
    }

    #[test]
    fn test_parse_movie() {
        let (media_type, info) = parse_filename("The.Matrix.1999.1080p.BluRay.x264.mkv");
        assert_eq!(media_type, MediaType::Movie);
        assert_eq!(info.title, "The Matrix");
        assert_eq!(info.year, Some(1999));
    }
}

