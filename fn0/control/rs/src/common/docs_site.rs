//! Serves the repository's `docs/` markdown tree at `/docs`.
//!
//! Markdown files are embedded at compile time and rendered to HTML per
//! request with pulldown-cmark. Relative `.md` links are rewritten to
//! `/docs/...` routes and headings get GitHub-style id anchors.

use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, TagEnd, html};
use serde::Serialize;

pub struct Doc {
    pub route: &'static str,
    pub title: &'static str,
    pub markdown: &'static str,
}

pub struct Section {
    pub title: &'static str,
    pub docs: &'static [Doc],
}

pub static SECTIONS: &[Section] = &[
    Section {
        title: "Getting Started",
        docs: &[
            Doc {
                route: "",
                title: "Introduction",
                markdown: include_str!("../../../../../docs/README.md"),
            },
            Doc {
                route: "setup",
                title: "Setup",
                markdown: include_str!("../../../../../docs/setup.md"),
            },
            Doc {
                route: "development",
                title: "Development Workflow",
                markdown: include_str!("../../../../../docs/development.md"),
            },
        ],
    },
    Section {
        title: "Forte Framework",
        docs: &[
            Doc {
                route: "forte/quick-reference",
                title: "Quick Reference",
                markdown: include_str!("../../../../../docs/forte/quick-reference.md"),
            },
            Doc {
                route: "forte/overview",
                title: "Overview",
                markdown: include_str!("../../../../../docs/forte/overview.md"),
            },
            Doc {
                route: "forte/project-structure",
                title: "Project Structure",
                markdown: include_str!("../../../../../docs/forte/project-structure.md"),
            },
            Doc {
                route: "forte/cli",
                title: "CLI Reference",
                markdown: include_str!("../../../../../docs/forte/cli.md"),
            },
            Doc {
                route: "forte/pages",
                title: "Pages",
                markdown: include_str!("../../../../../docs/forte/pages.md"),
            },
            Doc {
                route: "forte/apis",
                title: "API Endpoints",
                markdown: include_str!("../../../../../docs/forte/apis.md"),
            },
            Doc {
                route: "forte/actions",
                title: "Actions & Tasks",
                markdown: include_str!("../../../../../docs/forte/actions.md"),
            },
            Doc {
                route: "forte/frontend",
                title: "Frontend Runtime",
                markdown: include_str!("../../../../../docs/forte/frontend.md"),
            },
            Doc {
                route: "forte/sdk",
                title: "SDK Reference",
                markdown: include_str!("../../../../../docs/forte/sdk.md"),
            },
            Doc {
                route: "forte/codegen",
                title: "Code Generation",
                markdown: include_str!("../../../../../docs/forte/codegen.md"),
            },
            Doc {
                route: "forte/testing",
                title: "Testing",
                markdown: include_str!("../../../../../docs/forte/testing.md"),
            },
            Doc {
                route: "forte/troubleshooting",
                title: "Troubleshooting",
                markdown: include_str!("../../../../../docs/forte/troubleshooting.md"),
            },
        ],
    },
    Section {
        title: "Platform",
        docs: &[
            Doc {
                route: "fn0/overview",
                title: "fn0 Platform",
                markdown: include_str!("../../../../../docs/fn0/overview.md"),
            },
            Doc {
                route: "doc-db/overview",
                title: "doc-db",
                markdown: include_str!("../../../../../docs/doc-db/overview.md"),
            },
            Doc {
                route: "object-storage/overview",
                title: "Object Storage",
                markdown: include_str!("../../../../../docs/object-storage/overview.md"),
            },
        ],
    },
];

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

pub fn render_html(doc: &Doc) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let current_dir = doc.route.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("");
    let mut events: Vec<Event> = Parser::new_ext(doc.markdown, options).collect();

    for event in &mut events {
        if let Event::Start(Tag::Link { dest_url, .. }) = event
            && let Some(rewritten) = rewrite_markdown_link(dest_url, current_dir)
        {
            *dest_url = CowStr::from(rewritten);
        }
    }
    add_heading_ids(&mut events);

    let mut out = String::new();
    html::push_html(&mut out, events.into_iter());
    out
}

fn rewrite_markdown_link(dest: &str, current_dir: &str) -> Option<String> {
    if dest.starts_with('#')
        || dest.starts_with('/')
        || dest.contains("://")
        || dest.starts_with("mailto:")
    {
        return None;
    }
    let (path, fragment) = match dest.split_once('#') {
        Some((path, fragment)) => (path, Some(fragment)),
        None => (dest, None),
    };
    let path = path.strip_suffix(".md")?;

    let mut segments: Vec<&str> = current_dir.split('/').filter(|s| !s.is_empty()).collect();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            segment => segments.push(segment),
        }
    }
    if segments.last() == Some(&"README") {
        segments.pop();
    }

    let mut route = String::from("/docs");
    for segment in segments {
        route.push('/');
        route.push_str(segment);
    }
    if let Some(fragment) = fragment {
        route.push('#');
        route.push_str(fragment);
    }
    Some(route)
}

fn add_heading_ids(events: &mut [Event]) {
    let mut used_slugs: Vec<String> = Vec::new();
    let mut pending: Vec<(usize, String)> = Vec::new();

    let mut index = 0;
    while index < events.len() {
        if let Event::Start(Tag::Heading { id: None, .. }) = &events[index] {
            let mut text = String::new();
            let mut cursor = index + 1;
            while cursor < events.len() {
                match &events[cursor] {
                    Event::End(TagEnd::Heading(_)) => break,
                    Event::Text(t) | Event::Code(t) => text.push_str(t),
                    _ => {}
                }
                cursor += 1;
            }
            let mut slug = slugify(&text);
            if slug.is_empty() {
                slug = format!("section-{index}");
            }
            let mut unique = slug.clone();
            let mut n = 1;
            while used_slugs.contains(&unique) {
                n += 1;
                unique = format!("{slug}-{n}");
            }
            used_slugs.push(unique.clone());
            pending.push((index, unique));
            index = cursor;
        }
        index += 1;
    }

    for (index, slug) in pending {
        if let Event::Start(Tag::Heading { id, .. }) = &mut events[index] {
            *id = Some(CowStr::from(slug));
        }
    }
}

fn slugify(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    let mut last_dash = true;
    for c in text.chars() {
        if c.is_alphanumeric() {
            slug.extend(c.to_lowercase());
            last_dash = false;
        } else if (c.is_whitespace() || c == '-' || c == '_') && !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    slug
}
