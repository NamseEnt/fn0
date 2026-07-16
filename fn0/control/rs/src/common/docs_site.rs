//! Serves the repository's `docs/` markdown tree at `/docs`.
//!
//! All rendering (markdown → HTML, syntax highlighting, link rewriting,
//! heading anchors) happens in `build.rs`; this module only exposes the
//! pre-rendered pages and the sidebar navigation.

use serde::Serialize;

pub struct Doc {
    pub route: &'static str,
    pub title: &'static str,
    pub html: &'static str,
}

pub struct Section {
    pub title: &'static str,
    pub docs: &'static [Doc],
}

include!(concat!(env!("OUT_DIR"), "/docs_generated.rs"));

#[derive(Serialize)]
pub struct NavItem {
    pub route: String,
    pub title: String,
}

#[derive(Serialize)]
pub struct NavSection {
    pub title: String,
    pub items: Vec<NavItem>,
}

pub fn nav() -> Vec<NavSection> {
    SECTIONS
        .iter()
        .map(|section| NavSection {
            title: section.title.to_string(),
            items: section
                .docs
                .iter()
                .map(|doc| NavItem {
                    route: doc.route.to_string(),
                    title: doc.title.to_string(),
                })
                .collect(),
        })
        .collect()
}

pub fn find(route: &str) -> Option<&'static Doc> {
    SECTIONS
        .iter()
        .flat_map(|section| section.docs.iter())
        .find(|doc| doc.route == route)
}
