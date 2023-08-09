// Copyright (c) 2023 Seokjin Han <raifthenerd@gmail.com>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! Dash/Zeal docsets for selected Julia packages

use std::path::{Path, PathBuf};
use std::{env, fmt, fs, io};

use rusqlite::Connection;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum EntryType {
    // from Documenter.Utilities.doccat
    Macro,
    Keyword,
    Method,
    Function,
    Type,
    Module,
    Constant,
    // from Documenter.HTMLWriter.SearchRecord
    #[serde(rename = "page")]
    Guide,
    Section,
}
impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize)]
struct Index {
    title: String,
    category: EntryType,
    location: String,
    page: String,
}

fn copy_dir(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            copy_dir(entry.path(), destination.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn prepare_folders(entry: fs::DirEntry) -> io::Result<(String, PathBuf, PathBuf)> {
    let package_root = entry.path();
    let package_name = package_root
        .file_name()
        .and_then(|x| x.to_str())
        .and_then(|x| x.strip_suffix(".jl"))
        .unwrap();
    let docset_root = env::temp_dir().join(format!("{}.jl.docset", package_name));
    copy_dir(
        package_root.join("stable"),
        docset_root.join("Contents/Resources/Documents"),
    )?;
    Ok((package_name.to_owned(), package_root, docset_root))
}

fn prepare_info(package_name: &String, docset_root: &Path) -> io::Result<()> {
    let contents = format!("\
    <?xml version=\"1.0\" encoding=\"UTF-8\"?>
    <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
    <plist version=\"1.0\">
    <dict>
        <key>CFBundleName</key>
        <string>{}.jl</string>
        <key>CFBundleIdentifier</key>
        <string>{}.jl</string>
        <key>DocSetPlatformFamily</key>
        <string>{}jl</string>
        <key>isDashDocset</key>
        <true/>
        <key>dashIndexFilePath</key>
        <string>index.html</string>
        <key>isJavaScriptEnabled</key>
        <false/>
    </dict>
    </plist>", package_name, package_name.to_lowercase(), package_name.to_lowercase()
    );
    fs::write(docset_root.join("Contents/Info.plist"), contents)
}

fn prepare_docset_db(docset_root: &Path) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(docset_root.join("Contents/Resources/docSet.dsidx"))?;
    conn.execute_batch(
        "BEGIN;
        CREATE TABLE searchIndex(id INTEGER PRIMARY KEY, name TEXT, type TEXT, path TEXT);
        CREATE UNIQUE INDEX anchor ON searchIndex (name, type, path);
        COMMIT;",
    )?;
    Ok(conn)
}

fn read_search_index(package_root: &Path) -> io::Result<String> {
    let content = fs::read_to_string(package_root.join("stable/search_index.js"))?;
    Ok(content.lines().nth(1).unwrap().to_owned())
}

fn insert_search_index(conn: &Connection, index: &Index) -> Result<usize, rusqlite::Error> {
    if !index.location.is_empty() {
        let name = if index.category != EntryType::Section {
            index.title.to_owned()
        } else {
            format!("{} - {}", index.title, index.page)
        };
        let path = if index.location.ends_with('/') {
            index.location.to_owned() + "index.html"
        } else {
            index.location.replace("/#", "/index.html#")
        };
        conn.execute(
            "INSERT OR IGNORE INTO searchIndex(name, type, path) VALUES (?1, ?2, ?3)",
            (name, &index.category.to_string(), path),
        )
    } else {
        Ok(0)
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    for entry in fs::read_dir("ext")? {
        let entry = entry?;
        let (package_name, package_dir, docset_dir) = prepare_folders(entry)?;
        prepare_info(&package_name, &docset_dir)?;
        let conn = prepare_docset_db(&docset_dir)?;
        let indices: Vec<Index> = serde_json::from_str(&read_search_index(&package_dir)?)?;
        for index in indices {
            insert_search_index(&conn, &index)?;
        }
    }
    Ok(())
}
