use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::SeekFrom;
use std::path::Path;

use chrono::naive::NaiveDate;

use lang;

#[derive(Clone, Serialize)]
pub struct PageInfo {
    pub path: String,
    pub title: String,
    pub date: Option<NaiveDate>,
    pub lang: String,
}

impl PageInfo {
    pub fn new(path: &Path, file: &mut File) -> PageInfo {
        let mut reader = BufReader::new(file);
        let mut line = String::new();

        let mut date = None;
        let mut title = "Untitled";
        let mut lang = String::from("fr");

        let mut offset: u64 = 0;
        while let Ok(len) = reader.read_line(&mut line) {
            if line.starts_with("!date=") {
                if line.len() == 17 {
                    date = NaiveDate::parse_from_str(&line[6..16], "%Y-%m-%d").ok();
                }
            } else if line.starts_with("!lang=") {
                if line.len() == 9 {
                    lang = line[6..8].to_string();
                }
            } else if line.starts_with("#") {
                line.replace_range(..2, "");
                title = &line;
                let _ = reader.into_inner().seek(SeekFrom::Start(offset));
                break;
            }

            offset += len as u64;

            if len == 0 {
                break;
            }

            line.clear();
        }

        PageInfo {
            path: path.to_string_lossy().to_string(),
            title: title.to_string(),
            date,
            lang,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct PageData {
    pub info: PageInfo,
    pub content: String,
}

impl PageData {
    pub fn new(info: PageInfo, content: String) -> PageData {
        PageData { info, content }
    }
}

#[derive(Clone, Serialize)]
pub struct CorePageContext<'a> {
    pub locals: &'a lang::LocalMap,
    pub uri: String,
}

impl<'a> CorePageContext<'a> {
    pub fn new(local_map: &'a lang::LocalMap, uri: String) -> CorePageContext<'a> {
        CorePageContext {
            locals: local_map,
            uri,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct PageContext<'a> {
    pub page_data: PageData,
    pub category: String,
    pub core: CorePageContext<'a>,
}

impl<'a> PageContext<'a> {
    pub fn new(
        page_data: PageData,
        category: String,
        core: CorePageContext<'a>,
    ) -> PageContext<'a> {
        PageContext {
            page_data,
            category,
            core,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct IndexPageContext<'a> {
    pub pages: Vec<PageInfo>,
    pub core: CorePageContext<'a>,
}

impl<'a> IndexPageContext<'a> {
    pub fn new(pages: Vec<PageInfo>, core: CorePageContext<'a>) -> IndexPageContext<'a> {
        IndexPageContext { pages, core }
    }
}

#[derive(Clone, Serialize)]
pub struct MinimalPageContext<'a> {
    pub core: CorePageContext<'a>,
}

impl<'a> MinimalPageContext<'a> {
    pub fn new(core: CorePageContext<'a>) -> MinimalPageContext<'a> {
        MinimalPageContext { core }
    }
}
