use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::Duration;

use hoedown::renderer::html::Flags;
use hoedown::Html;
use hoedown::Markdown;
use hoedown::Render;
use walkdir::DirEntry;
use walkdir::WalkDir;

use cacher::Cacher;
use config;

pub mod page;

pub use self::page::*;

lazy_static! {
    static ref cacher: RwLock<Cacher<String, PageData>> = RwLock::new(Cacher::new(
        Duration::from_secs(600),
        Duration::from_secs(1000)
    ));
}

pub fn make_data_from_markdown(page_name: &str) -> Option<PageData> {
    let page_path = PathBuf::from(format!("{}{}.md", config::MARKDOWN_PATH, page_name));

    if Path::new(&page_path).exists() {
        let read_guard = cacher.read().unwrap();
        let mut page_data = read_guard.get(&page_name.to_string()).map(|v| v.clone());
        drop(read_guard);

        if page_data.is_none() {
            page_data = Some(
                cacher
                    .write()
                    .unwrap()
                    .get_or_insert(page_name.to_string(), move || {
                        let mut file = File::open(&page_path).unwrap();
                        let info = PageInfo::new(&page_path, &mut file);
                        let markdown = Markdown::read_from(file);
                        let mut html = Html::new(Flags::empty(), 0);
                        PageData::new(info, html.render(&markdown).to_str().unwrap().to_owned())
                    })
                    .clone(),
            );
        }

        page_data
    } else {
        None
    }
}

pub fn list_available_pages_from_dir(dir: &str) -> Vec<PageInfo> {
    let mapper = |entry: DirEntry| -> PageInfo {
        let mut file = File::open(entry.path()).unwrap();
        let path = entry.path().strip_prefix(dir).unwrap();
        let path_str = match path.parent() {
            Some(parent) if parent != Path::new("") => format!(
                "{}/{}",
                parent.display(),
                path.file_stem().unwrap().to_string_lossy()
            )
            .to_string(),
            _ => path.file_stem().unwrap().to_string_lossy().to_string(),
        };
        PageInfo::new(Path::new(&path_str), &mut file)
    };

    WalkDir::new(dir)
        .into_iter()
        .map(|e| e.unwrap())
        .filter(|e| e.path().is_file() && e.path().extension() == Some(OsStr::new("md")))
        .map(mapper)
        .collect()
}
