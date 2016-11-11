//! create blog from markdown files.
//!
//! # features
//!
//! * markdown format
//! * TeX style math support
//! * highlight code block
//! * post tags index
//! * hidden post
//! * post title is the title of markdown file
//! * post url is some to path of markdown file

#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
       html_favicon_url = "https://www.rust-lang.org/favicon.ico",
       html_root_url = "http://fugangqiang.github.io/doc/mdblog.rs")]

#[macro_use] extern crate log;
extern crate chrono;
extern crate pulldown_cmark;
extern crate serde_json;
extern crate tera;
extern crate toml;
extern crate walkdir;

mod error;
mod post;
mod theme;
mod utils;

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde_json::Map;
use tera::{Tera, Context};
use walkdir::{DirEntry, WalkDir, WalkDirIterator};

pub use error::{Error, Result};
pub use post::Post;
pub use theme::Theme;
pub use utils::create_file;


/// blog object
pub struct Mdblog {
    /// blog root path
    root: PathBuf,
    /// blog theme
    theme: Theme,
    /// collection of blog posts
    posts: Vec<Rc<Post>>,
    /// tagged posts
    tags: BTreeMap<String, Vec<Rc<Post>>>,
    /// blog render
    renderer: Option<Tera>,
    /// blog config
    config: Option<toml::Value>,
}


impl Mdblog {
    /// create Mdblog from the `root` path
    pub fn new<P: AsRef<Path>>(root: P) -> Mdblog {
        Mdblog {
            root: root.as_ref().to_owned(),
            theme: Theme::new(&root),
            posts: Vec::new(),
            tags: BTreeMap::new(),
            renderer: None,
            config: None,
        }
    }

    /// init Mdblog with `theme`.
    ///
    /// theme directory is created at `root/_theme` directory.
    /// if `theme` is `None`, use the default theme(`simple`).
    pub fn init(&self, theme: Option<String>) -> Result<()> {
        if self.root.exists() {
            return Err(Error::RootDirExisted);
        }

        let mut hello_post = create_file(&self.root.join("posts").join("hello.md"))?;
        hello_post.write_all(b"date: 1970-01-01 00:00:00\n")?;
        hello_post.write_all(b"tags: hello, world\n")?;
        hello_post.write_all(b"\n")?;
        hello_post.write_all(b"# hello\n\nhello world!\n")?;

        let mut config = create_file(&self.root.join("config.toml"))?;
        config.write_all(b"[blog]\ntheme = simple\n")?;

        let name = theme.unwrap_or(self.get_config_theme());
        let mut t = Theme::new(&self.root);
        t.load(&name)?;
        t.init_dir()?;

        fs::create_dir_all(self.root.join("media"))?;
        Ok(())
    }

    /// create the blog html files to `root/_build/` directory.
    ///
    /// if `theme` is `None`, use the default theme(`simple`).
    pub fn build(&mut self, theme: Option<String>) -> Result<()> {
        let name = theme.unwrap_or(self.get_config_theme());
        self.load_theme(&name)?;
        self.load_posts()?;
        self.export()?;
        Ok(())
    }

    /// unimplemented.
    pub fn server(&self, port: u16) {
        println!("server blog at localhost:{}", port);
    }

    /// fetch the config theme from the `config.toml` file
    pub fn get_config_theme(&self) -> String {
        self.config
            .as_ref()
            .and_then(|v| v.lookup("blog.theme"))
            .and_then(|v| v.as_str())
            .map(|x| x.to_string())
            .unwrap_or("simple".to_string())
    }

    pub fn load_config(&mut self) -> Result<()> {
        let mut content = String::new();
        let config_path = self.root.join("config.toml");
        let mut f = File::open(&config_path).unwrap();
        f.read_to_string(&mut content).unwrap();
        self.config = content.parse().ok();
        Ok(())
    }

    pub fn load_theme(&mut self, theme: &str) -> Result<()> {
        self.theme.load(theme)?;
        let template_dir = self.root.join("_themes").join(&theme).join("templates");
        debug!("template dir: {}", template_dir.display());
        self.renderer = Some(Tera::new(&format!("{}/*", template_dir.display())));
        Ok(())
    }

    pub fn load_posts(&mut self) -> Result<()> {
        let posts_dir = self.root.join("posts");
        let walker = WalkDir::new(&posts_dir).into_iter();
        for entry in walker.filter_entry(|e| !is_hidden(e)) {
            let entry = entry.expect("get walker entry error");
            if !is_markdown_file(&entry) {
                continue;
            }
            let mut post = Post::new(&self.root,
                                     &entry.path()
                                           .strip_prefix(&self.root)
                                           .expect("create post path error")
                                           .to_owned());
            post.load()?;
            let post = Rc::new(post);
            self.posts.push(post.clone());
            if !post.is_hidden()? {
                for tag in post.tags() {
                    let mut ps = self.tags.entry(tag.to_string()).or_insert(Vec::new());
                    ps.push(post.clone());
                }
            }
        }
        self.posts.sort_by(|p1, p2| p2.datetime().cmp(&p1.datetime()));
        for (_, tag_posts) in self.tags.iter_mut() {
            tag_posts.sort_by(|p1, p2| p2.datetime().cmp(&p1.datetime()));
        }
        Ok(())
    }

    pub fn export(&self) -> Result<()> {
        self.export_media()?;
        self.export_static()?;
        self.export_posts()?;
        self.export_index()?;
        self.export_tags()?;
        Ok(())
    }

    pub fn media_dest<P: AsRef<Path>>(&self, media: P) -> PathBuf {
        let relpath = media.as_ref().strip_prefix(&self.root.join("media")).expect("create post path error").to_owned();
        self.root.join("_builds/media").join(relpath)
    }

    pub fn export_media(&self) -> Result<()> {
        debug!("exporting media ...");
        let walker = WalkDir::new(&self.root.join("media")).into_iter();
        for entry in walker.filter_entry(|e| !is_hidden(e)) {
            let entry = entry.expect("get walker entry error");
            let src_path = entry.path();
            if src_path.is_dir() {
                fs::create_dir_all(self.media_dest(src_path))?;
                continue;
            }
            fs::copy(src_path, self.media_dest(src_path))?;
        }
        Ok(())
    }

    pub fn export_static(&self) -> Result<()> {
        self.theme.export_static()
    }

    pub fn export_posts(&self) -> Result<()> {
        for post in &self.posts {
            let dest = post.dest();
            let mut f = create_file(&dest)?;
            let html = self.render_post(post)?;
            f.write(html.as_bytes())?;
        }
        Ok(())
    }

    pub fn export_index(&self) -> Result<()> {
        let dest = self.root.join("_builds/index.html");
        let mut f = create_file(&dest)?;
        let html = self.render_index()?;
        f.write(html.as_bytes())?;
        Ok(())
    }

    pub fn export_tags(&self) -> Result<()> {
        for tag in self.tags.keys() {
            let dest = self.root.join(format!("_builds/blog/tags/{}.html", tag));
            let mut f = create_file(&dest)?;
            let html = self.render_tag(tag)?;
            f.write(html.as_bytes())?;
        }
        Ok(())
    }

    fn tag_url(&self, name: &str) -> String {
        format!("/blog/tags/{}.html", &name)
    }

    fn tag_map<T>(&self, name:&str, posts: &Vec<T>) -> Map<&str, String> {
        let mut map = Map::new();
        map.insert("name", name.to_string());
        let tag_len = format!("{:?}", &posts.len());
        map.insert("num", tag_len);
        map.insert("url", self.tag_url(&name));
        map
    }

    pub fn base_context(&self, title: &str) -> Context {
        let mut context = Context::new();
        context.add("title", &title);
        let mut all_tags = Vec::new();
        for (tag_key, tag_posts) in &self.tags {
            all_tags.push(self.tag_map(&tag_key, &tag_posts));
        }
        all_tags.sort_by(|a, b| a.get("name").map(|s| s.to_lowercase())
                                 .cmp(&b.get("name").map(|s| s.to_lowercase())));
        context.add("all_tags", &all_tags);
        context
    }

    pub fn render_post(&self, post: &Post) -> Result<String> {
        debug!("rendering post({}) ...", post.path.display());
        let tera = self.renderer.as_ref().expect("get renderer error");
        let mut context = self.base_context(&post.title());
        context.add("content", &post.content());
        let mut post_tags = Vec::new();
        if !post.is_hidden()? {
            context.add("datetime", &post.datetime().format("%Y-%m-%d %H:%M:%S").to_string());
            for tag_key in post.tags() {
                let tag_posts = self.tags.get(tag_key).expect(&format!("post tag({}) does not add to blog tags", tag_key));
                post_tags.push(self.tag_map(&tag_key, &tag_posts));
            }
        } else {
            context.add("datetime", &"".to_string());
        }

        context.add("post_tags", &post_tags);
        tera.render("post.tpl", context).map_err(|e| Error::Render(e))
    }

    pub fn render_index(&self) -> Result<String> {
        debug!("rendering index ...");
        let tera = self.renderer.as_ref().expect("get renderer error");
        let mut context = self.base_context("Fu");
        context.add("posts", &self.posts_maps(&self.posts));
        tera.render("index.tpl", context).map_err(|e| Error::Render(e))
    }

    fn posts_maps<'a>(&self, posts: &'a Vec<Rc<Post>>) -> Vec<Map<&'a str, String>> {
        let mut maps = Vec::new();
        for post in posts.iter().filter(|p| !p.is_hidden().unwrap()) {
            maps.push(post.map());
        }
        maps
    }

    pub fn render_tag(&self, tag:&str) -> Result<String> {
        debug!("rendering tag({}) ...", tag);
        let tera = self.renderer.as_ref().expect("get renderer error");
        let mut context = self.base_context(&tag);
        let posts = self.tags.get(tag).expect(&format!("get tag({}) error", &tag));
        context.add("posts", &self.posts_maps(&posts));
        tera.render("tag.tpl", context).map_err(|e| Error::Render(e))
    }
}


fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}


fn is_markdown_file(entry: &DirEntry) -> bool {
    if !entry.path().is_file() {
        return false;
    }
    let fname = entry.file_name().to_str();
    match fname {
        None => {
            return false;
        },
        Some(s) => {
            if s.starts_with(|c| (c == '.') | (c == '~')) {
                return false;
            } else if s.ends_with(".md") {
                return true;
            } else {
                return false;
            }
        }
    }
}
