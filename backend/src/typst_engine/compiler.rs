use std::collections::HashMap;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Smart};
use typst::layout::PagedDocument;
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_kit::fonts::Fonts;
use typst_pdf::{PdfOptions, pdf};

/// Typst World implementasyonu — dreport backend için.
/// Fonts referans olarak tutulur (clone edilemez).
pub struct DreportWorld<'a> {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: &'a Fonts,
    main_source: Source,
    /// Sanal dosyalar (ör: base64 image'lar)
    files: HashMap<String, Bytes>,
}

impl<'a> DreportWorld<'a> {
    pub fn new(typst_markup: String, fonts: &'a Fonts, files: HashMap<String, Vec<u8>>) -> Self {
        let main_id = FileId::new_fake(VirtualPath::new("main.typ"));
        Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(fonts.book.clone()),
            fonts,
            main_source: Source::new(main_id, typst_markup),
            files: files.into_iter().map(|(k, v)| (k, Bytes::new(v))).collect(),
        }
    }
}

impl World for DreportWorld<'_> {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main_source.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main_source.id() {
            Ok(self.main_source.clone())
        } else {
            Err(FileError::NotFound(id.vpath().as_rooted_path().into()))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = id.vpath().as_rooted_path();
        let path_str = path.to_string_lossy();
        // Baştaki "/" veya "./" kaldır
        let clean_path = path_str.trim_start_matches('/').trim_start_matches("./");

        if let Some(bytes) = self.files.get(clean_path) {
            Ok(bytes.clone())
        } else {
            Err(FileError::NotFound(path.into()))
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.fonts.get(index)?.get()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let now = chrono::Utc::now();
        let offset_secs = offset.unwrap_or(0) * 3600;
        let tz = chrono::FixedOffset::east_opt(offset_secs as i32)?;
        let local = now.with_timezone(&tz);
        use chrono::Datelike;
        Datetime::from_ymd(
            local.year(),
            local.month().try_into().ok()?,
            local.day().try_into().ok()?,
        )
    }
}

/// Typst markup → PDF bytes
pub fn compile_pdf(typst_markup: String, fonts: &Fonts, files: HashMap<String, Vec<u8>>) -> Result<Vec<u8>, String> {
    let world = DreportWorld::new(typst_markup, fonts, files);

    // Derleme
    let warned = typst::compile::<PagedDocument>(&world);
    let document = warned.output.map_err(|errs| {
        errs.into_iter()
            .map(|e| e.message.to_string())
            .collect::<Vec<_>>()
            .join("; ")
    })?;

    // PDF export
    let options = PdfOptions {
        ident: Smart::Auto,
        timestamp: None,
        page_ranges: None,
        standards: Default::default(),
        tagged: false,
    };
    let pdf_bytes = pdf(&document, &options).map_err(|errs| {
        errs.into_iter()
            .map(|e| e.message.to_string())
            .collect::<Vec<_>>()
            .join("; ")
    })?;

    Ok(pdf_bytes)
}
