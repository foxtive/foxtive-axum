use crate::{FoxtiveAxumExt, FOXTIVE_AXUM};
use foxtive::helpers::FileExtHelper;
use std::path::{Path, PathBuf};

pub const DEFAULT_STATIC_MEDIA_EXTENSIONS: &[&str] = &[
    // Images
    "jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico", "tiff", "tif", // Audio
    "mp3", "wav", "ogg", "m4a", "aac", "flac", "wma", "opus", // Video
    "mp4", "avi", "mov", "wmv", "flv", "webm", "mkv", "m4v", "3gp", "ogv", // Documents
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "rtf", "odt", "ods", "odp",
    // Web assets
    "css", "js", "html", "htm", "xml", "json", "woff", "woff2", "ttf", "otf", "eot",
    // Archives
    "zip", "rar", "7z", "tar", "gz", "bz2", "xz", // Other common static files
    "swf", "eps", "ai", "psd", "sketch", "fig",
];

pub fn is_url_a_file(path: &str) -> bool {
    if let Some(ext) = FileExtHelper::new().get_extension(path) {
        return FOXTIVE_AXUM
            .app()
            .allowed_static_media_extensions
            .contains(&ext);
    }

    false
}

pub fn resolve_static_file_path(base_dir: &Path, request_path: &Path) -> PathBuf {
    let base_last = base_dir.file_name();

    // Normalize the request path by stripping leading slashes
    let components = request_path
        .components()
        .filter(|c| !matches!(c, std::path::Component::RootDir));

    // Peekable so we can inspect the first component
    let mut components = components.peekable();

    if let (Some(base_last), Some(first_component)) = (base_last, components.peek())
        && first_component.as_os_str() == base_last
    {
        components.next();
    }

    base_dir.join(components.collect::<PathBuf>())
}
