use serde::{Deserialize, Serialize};

/// Parsed metadata from a single font file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontMeta {
    /// Font family name from name table (nameID 16 preferred, fallback nameID 1)
    pub family: String,
    /// usWeightClass from OS/2 table (100-900)
    pub weight: u16,
    /// fsSelection bit 0 from OS/2 table
    pub italic: bool,
    pub units_per_em: u16,
    /// sTypoAscender from OS/2 table
    pub ascender: i16,
    /// sTypoDescender from OS/2 table
    pub descender: i16,
}

/// Variant key for looking up a specific font within a family
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct FontVariantKey {
    pub weight: u16,
    pub italic: bool,
}

/// Summary of a font family with all its available variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFamilyInfo {
    pub family: String,
    pub variants: Vec<FontVariantKey>,
}

impl FontMeta {
    pub fn variant_key(&self) -> FontVariantKey {
        FontVariantKey {
            weight: self.weight,
            italic: self.italic,
        }
    }

    pub fn is_bold(&self) -> bool {
        self.weight >= 700
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Read a big-endian u16 from `data` at `offset`. Returns `None` if out of bounds.
fn read_u16(data: &[u8], offset: usize) -> Option<u16> {
    if offset + 2 > data.len() {
        return None;
    }
    Some(u16::from_be_bytes([data[offset], data[offset + 1]]))
}

/// Read a big-endian i16 from `data` at `offset`. Returns `None` if out of bounds.
fn read_i16(data: &[u8], offset: usize) -> Option<i16> {
    if offset + 2 > data.len() {
        return None;
    }
    Some(i16::from_be_bytes([data[offset], data[offset + 1]]))
}

/// Read a big-endian u32 from `data` at `offset`. Returns `None` if out of bounds.
fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    if offset + 4 > data.len() {
        return None;
    }
    Some(u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

/// Find a table in the font's table directory by its 4-byte ASCII tag.
/// Returns `(offset, length)` into `data`.
fn find_table(data: &[u8], tag: &[u8; 4]) -> Option<(usize, usize)> {
    // Offset table (first 12 bytes):
    //   0: sfVersion (u32) — 0x00010000 for TrueType, 'OTTO' for CFF
    //   4: numTables (u16)
    //   6: searchRange (u16)
    //   8: entrySelector (u16)
    //  10: rangeShift (u16)
    let num_tables = read_u16(data, 4)? as usize;

    // Table directory starts at offset 12, each entry is 16 bytes:
    //   0: tag (4 bytes)
    //   4: checksum (u32)
    //   8: offset (u32)
    //  12: length (u32)
    for i in 0..num_tables {
        let entry_offset = 12 + i * 16;
        if entry_offset + 16 > data.len() {
            return None;
        }

        if &data[entry_offset..entry_offset + 4] == tag {
            let table_offset = read_u32(data, entry_offset + 8)? as usize;
            let table_length = read_u32(data, entry_offset + 12)? as usize;
            // Basic sanity check
            if table_offset.checked_add(table_length)? > data.len() {
                return None;
            }
            return Some((table_offset, table_length));
        }
    }
    None
}

/// Decode a UTF-16BE byte slice into a `String`.
fn decode_utf16be(raw: &[u8]) -> Option<String> {
    if !raw.len().is_multiple_of(2) {
        return None;
    }
    let code_units: Vec<u16> = raw
        .chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16(&code_units).ok()
}

/// Decode a MacRoman (platform 1, encoding 0) byte slice into a `String`.
/// MacRoman overlaps with ASCII for 0x00–0x7F; we accept those and replace
/// high bytes with the Unicode replacement character for simplicity, since
/// font family names are almost always pure ASCII.
fn decode_mac_roman(raw: &[u8]) -> String {
    raw.iter()
        .map(|&b| {
            if b < 0x80 {
                b as char
            } else {
                // Simplified: map non-ASCII MacRoman bytes to replacement char.
                // Full MacRoman table not needed for typical font family names.
                '\u{FFFD}'
            }
        })
        .collect()
}

/// Extract the font family name from the `name` table.
///
/// Prefers nameID 16 (Typographic Family Name) over nameID 1 (Font Family).
/// Among platforms, prefers Windows (3) and Unicode (0) for UTF-16BE, falls
/// back to Macintosh (1) for MacRoman.
fn read_family_name(data: &[u8], table_offset: usize, table_length: usize) -> Option<String> {
    let tbl = table_offset;
    // name table header:
    //   0: format (u16)
    //   2: count (u16)
    //   4: stringOffset (u16) — offset from start of table to string storage
    let count = read_u16(data, tbl + 2)? as usize;
    let string_offset = read_u16(data, tbl + 4)? as usize;
    let storage_base = tbl + string_offset;

    // Each name record (12 bytes, starting at tbl + 6):
    //   0: platformID (u16)
    //   2: encodingID (u16)
    //   4: languageID (u16)
    //   6: nameID (u16)
    //   8: length (u16)
    //  10: offset (u16) — from storage_base

    // We collect candidates, preferring nameID 16 over 1, and Windows/Unicode
    // over Mac.
    let mut best: Option<String> = None;
    let mut best_priority: u8 = 0; // higher = better

    for i in 0..count {
        let rec = tbl + 6 + i * 12;
        if rec + 12 > tbl + table_length {
            break;
        }

        let platform_id = read_u16(data, rec)?;
        let encoding_id = read_u16(data, rec + 2)?;
        let name_id = read_u16(data, rec + 6)?;
        let str_length = read_u16(data, rec + 8)? as usize;
        let str_offset = read_u16(data, rec + 10)? as usize;

        // Only interested in nameID 1 (Font Family) or 16 (Typographic Family)
        if name_id != 1 && name_id != 16 {
            continue;
        }

        let name_priority = if name_id == 16 { 4 } else { 0 };

        let abs_start = storage_base + str_offset;
        let abs_end = abs_start + str_length;
        if abs_end > data.len() {
            continue;
        }
        let raw = &data[abs_start..abs_end];

        let (decoded, platform_priority) = match platform_id {
            // Platform 0 — Unicode: UTF-16BE
            0 => {
                if let Some(s) = decode_utf16be(raw) {
                    (s, 2u8)
                } else {
                    continue;
                }
            }
            // Platform 1 — Macintosh, encoding 0 = MacRoman
            1 if encoding_id == 0 => (decode_mac_roman(raw), 1u8),
            // Platform 3 — Windows, encoding 1 = Unicode BMP (UTF-16BE)
            3 if encoding_id == 1 => {
                if let Some(s) = decode_utf16be(raw) {
                    (s, 3u8)
                } else {
                    continue;
                }
            }
            _ => continue,
        };

        let priority = name_priority + platform_priority;
        if priority > best_priority {
            best_priority = priority;
            best = Some(decoded);
        }
    }

    best
}

/// Parse font metadata from raw TTF/OTF bytes.
///
/// Returns `None` if the data is too short, tables are missing, or offsets
/// point outside the buffer.
pub fn parse_font_meta(data: &[u8]) -> Option<FontMeta> {
    // Minimum: 12-byte offset table header
    if data.len() < 12 {
        return None;
    }

    // ---- OS/2 table ----
    let (os2_off, os2_len) = find_table(data, b"OS/2")?;
    // Need at least 72 bytes for sTypoDescender (offset 70, 2 bytes)
    if os2_len < 72 {
        return None;
    }
    let weight = read_u16(data, os2_off + 4)?;
    let fs_selection = read_u16(data, os2_off + 62)?;
    let italic = (fs_selection & 1) != 0;
    let ascender = read_i16(data, os2_off + 68)?;
    let descender = read_i16(data, os2_off + 70)?;

    // ---- head table ----
    let (head_off, head_len) = find_table(data, b"head")?;
    // unitsPerEm is at offset 18 (2 bytes), so need at least 20 bytes
    if head_len < 20 {
        return None;
    }
    let units_per_em = read_u16(data, head_off + 18)?;

    // ---- name table ----
    let (name_off, name_len) = find_table(data, b"name")?;
    let family = read_family_name(data, name_off, name_len)?;

    Some(FontMeta {
        family,
        weight,
        italic,
        units_per_em,
        ascender,
        descender,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_table_returns_none_on_empty() {
        assert!(find_table(&[], b"head").is_none());
    }

    #[test]
    fn parse_font_meta_returns_none_on_garbage() {
        assert!(parse_font_meta(&[0u8; 11]).is_none());
        assert!(parse_font_meta(&[0u8; 64]).is_none());
    }

    #[test]
    fn variant_key_and_is_bold() {
        let meta = FontMeta {
            family: "Test".into(),
            weight: 700,
            italic: true,
            units_per_em: 1000,
            ascender: 800,
            descender: -200,
        };
        assert!(meta.is_bold());
        assert!(meta.italic);
        let key = meta.variant_key();
        assert_eq!(key.weight, 700);
        assert!(key.italic);

        let regular = FontMeta {
            weight: 400,
            italic: false,
            ..meta.clone()
        };
        assert!(!regular.is_bold());
    }

    #[test]
    fn decode_utf16be_basic() {
        // "AB" in UTF-16BE
        let raw = [0x00, 0x41, 0x00, 0x42];
        assert_eq!(decode_utf16be(&raw).unwrap(), "AB");
    }

    #[test]
    fn decode_utf16be_odd_length_returns_none() {
        assert!(decode_utf16be(&[0x00, 0x41, 0x00]).is_none());
    }

    #[test]
    fn decode_mac_roman_ascii() {
        let raw = b"Noto Sans";
        assert_eq!(decode_mac_roman(raw), "Noto Sans");
    }
}
