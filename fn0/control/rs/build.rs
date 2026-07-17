//! Besides Forte route codegen, this build script renders the repository's
//! `docs/` markdown tree and the marketing code snippets to HTML with syntax
//! highlighting (syntect) at build time, so the wasm binary serves pre-rendered
//! HTML with zero per-request markdown/highlighting cost.

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd, html};
use std::fmt::Write as _;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, StyleModifier, Theme, ThemeItem};
use syntect::html::{IncludeBackground, styled_line_to_highlighted_html};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

struct DocSource {
    route: &'static str,
    title: &'static str,
    path: &'static str,
}

struct SectionSource {
    title: &'static str,
    docs: &'static [DocSource],
}

const DOCS_DIR: &str = "../../../docs";
const SNIPPETS_DIR: &str = "snippets";

const SECTIONS: &[SectionSource] = &[
    SectionSource {
        title: "Getting Started",
        docs: &[
            DocSource {
                route: "",
                title: "Introduction",
                path: "README.md",
            },
            DocSource {
                route: "setup",
                title: "Setup",
                path: "setup.md",
            },
            DocSource {
                route: "development",
                title: "Development Workflow",
                path: "development.md",
            },
        ],
    },
    SectionSource {
        title: "Forte Framework",
        docs: &[
            DocSource {
                route: "forte/quick-reference",
                title: "Quick Reference",
                path: "forte/quick-reference.md",
            },
            DocSource {
                route: "forte/overview",
                title: "Overview",
                path: "forte/overview.md",
            },
            DocSource {
                route: "forte/project-structure",
                title: "Project Structure",
                path: "forte/project-structure.md",
            },
            DocSource {
                route: "forte/cli",
                title: "CLI Reference",
                path: "forte/cli.md",
            },
            DocSource {
                route: "forte/pages",
                title: "Pages",
                path: "forte/pages.md",
            },
            DocSource {
                route: "forte/head",
                title: "Per-page Head",
                path: "forte/head.md",
            },
            DocSource {
                route: "forte/apis",
                title: "API Endpoints",
                path: "forte/apis.md",
            },
            DocSource {
                route: "forte/actions",
                title: "Actions & Tasks",
                path: "forte/actions.md",
            },
            DocSource {
                route: "forte/frontend",
                title: "Frontend Runtime",
                path: "forte/frontend.md",
            },
            DocSource {
                route: "forte/sdk",
                title: "SDK Reference",
                path: "forte/sdk.md",
            },
            DocSource {
                route: "forte/codegen",
                title: "Code Generation",
                path: "forte/codegen.md",
            },
            DocSource {
                route: "forte/testing",
                title: "Testing",
                path: "forte/testing.md",
            },
            DocSource {
                route: "forte/troubleshooting",
                title: "Troubleshooting",
                path: "forte/troubleshooting.md",
            },
        ],
    },
    SectionSource {
        title: "Platform",
        docs: &[
            DocSource {
                route: "fn0/overview",
                title: "fn0 Platform",
                path: "fn0/overview.md",
            },
            DocSource {
                route: "fn0/limits",
                title: "Limits & Quotas",
                path: "fn0/limits.md",
            },
            DocSource {
                route: "doc-db/overview",
                title: "doc-db",
                path: "doc-db/overview.md",
            },
            DocSource {
                route: "object-storage/overview",
                title: "Object Storage",
                path: "object-storage/overview.md",
            },
        ],
    },
];

struct SnippetSource {
    constant_name: &'static str,
    file: &'static str,
    lang: &'static str,
    // wraps this term in <span class="trace-hl"> so the page can visually
    // trace one field across related snippets
    trace_term: Option<&'static str>,
}

const SNIPPETS: &[SnippetSource] = &[
    SnippetSource {
        constant_name: "SNIPPET_HOME_HELLO_RS_HTML",
        file: "home_hello.rs",
        lang: "rust",
        trace_term: None,
    },
    SnippetSource {
        constant_name: "SNIPPET_FORTE_HELLO_RS_HTML",
        file: "forte_hello.rs",
        lang: "rust",
        trace_term: Some("message"),
    },
    SnippetSource {
        constant_name: "SNIPPET_FORTE_PROPS_JSON_HTML",
        file: "forte_props.json",
        lang: "json",
        trace_term: Some("message"),
    },
    SnippetSource {
        constant_name: "SNIPPET_FORTE_HELLO_TSX_HTML",
        file: "forte_hello.tsx",
        lang: "tsx",
        trace_term: Some("message"),
    },
    SnippetSource {
        constant_name: "SNIPPET_FORTE_DOC_DB_RS_HTML",
        file: "forte_doc_db.rs",
        lang: "rust",
        trace_term: None,
    },
    SnippetSource {
        constant_name: "SNIPPET_FORTE_OBJECT_STORAGE_RS_HTML",
        file: "forte_object_storage.rs",
        lang: "rust",
        trace_term: None,
    },
];

fn main() {
    forte_codegen::generate_routes();
    generate_docs();
}

fn generate_docs() {
    println!("cargo:rerun-if-changed={DOCS_DIR}");
    println!("cargo:rerun-if-changed={SNIPPETS_DIR}");

    let syntax_set = two_face::syntax::extra_newlines();
    let theme = site_theme();

    let mut out = String::new();
    out.push_str("pub static SECTIONS: &[Section] = &[\n");
    for section in SECTIONS {
        writeln!(out, "    Section {{ title: {:?}, docs: &[", section.title).unwrap();
        for doc in section.docs {
            let path = Path::new(DOCS_DIR).join(doc.path);
            let markdown = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            let current_dir = doc.route.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("");
            let html = render_markdown(&markdown, current_dir, &syntax_set, &theme);
            writeln!(
                out,
                "        Doc {{ route: {:?}, title: {:?}, html: {:?} }},",
                doc.route, doc.title, html
            )
            .unwrap();
        }
        out.push_str("    ] },\n");
    }
    out.push_str("];\n");

    for snippet in SNIPPETS {
        let path = Path::new(SNIPPETS_DIR).join(snippet.file);
        let code = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let mut html = highlight_code(code.trim_end(), snippet.lang, &syntax_set, &theme);
        if let Some(term) = snippet.trace_term {
            html = wrap_trace_term(&html, term);
        }
        writeln!(
            out,
            "pub static {}: &str = {html:?};",
            snippet.constant_name
        )
        .unwrap();
    }

    let out_path = Path::new(&std::env::var("OUT_DIR").unwrap()).join("docs_generated.rs");
    std::fs::write(out_path, out).unwrap();
}

fn site_theme() -> Theme {
    let mut theme = Theme::default();
    theme.settings.foreground = Some(hex_color("#E9E9F1"));
    let scopes = [
        ("comment", "#6E7089"),
        ("keyword, storage", "#B4ABFF"),
        ("string", "#86E1B0"),
        (
            "constant, constant.numeric, constant.language, constant.character.escape",
            "#F2CD88",
        ),
        (
            "entity.name.type, support.type, support.class, entity.other.inherited-class",
            "#9CC7FF",
        ),
        ("entity.name.function, support.function", "#6FD3F2"),
        ("entity.name.tag", "#B4ABFF"),
        (
            "meta.attribute, entity.other.attribute-name, meta.annotation, punctuation.definition.annotation",
            "#9C9EB4",
        ),
    ];
    for (selector, color) in scopes {
        theme.scopes.push(ThemeItem {
            scope: selector.parse().unwrap(),
            style: StyleModifier {
                foreground: Some(hex_color(color)),
                background: None,
                font_style: None,
            },
        });
    }
    theme
}

fn hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    Color {
        r: u8::from_str_radix(&hex[0..2], 16).unwrap(),
        g: u8::from_str_radix(&hex[2..4], 16).unwrap(),
        b: u8::from_str_radix(&hex[4..6], 16).unwrap(),
        a: 0xFF,
    }
}

fn render_markdown(
    markdown: &str,
    current_dir: &str,
    syntax_set: &SyntaxSet,
    theme: &Theme,
) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let mut events: Vec<Event> = Vec::new();
    let mut parser = Parser::new_ext(markdown, options);
    while let Some(event) = parser.next() {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match &kind {
                    CodeBlockKind::Fenced(info) => {
                        info.split([',', ' ']).next().unwrap_or("").to_string()
                    }
                    CodeBlockKind::Indented => String::new(),
                };
                let mut code = String::new();
                for event in parser.by_ref() {
                    match event {
                        Event::End(TagEnd::CodeBlock) => break,
                        Event::Text(text) => code.push_str(&text),
                        _ => {}
                    }
                }
                let block = format!(
                    "<pre><code>{}</code></pre>\n",
                    highlight_code(&code, &lang, syntax_set, theme)
                );
                events.push(Event::Html(CowStr::from(block)));
            }
            other => events.push(other),
        }
    }

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

fn highlight_code(code: &str, lang: &str, syntax_set: &SyntaxSet, theme: &Theme) -> String {
    let syntax = if lang.is_empty() {
        None
    } else {
        syntax_set.find_syntax_by_token(lang)
    };
    let Some(syntax) = syntax else {
        return escape_html(code);
    };
    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut out = String::new();
    for line in LinesWithEndings::from(code) {
        let regions = highlighter
            .highlight_line(line, syntax_set)
            .expect("highlight line");
        out.push_str(
            &styled_line_to_highlighted_html(&regions, IncludeBackground::No)
                .expect("style line to html"),
        );
    }
    out
}

fn wrap_trace_term(html: &str, term: &str) -> String {
    let replacement = format!("<span class=\"trace-hl\">{term}</span>");
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    // replace only in text content, never inside tags
    while let Some(tag_start) = rest.find('<') {
        let (text, after) = rest.split_at(tag_start);
        out.push_str(&text.replace(term, &replacement));
        let tag_end = after.find('>').map(|i| i + 1).unwrap_or(after.len());
        out.push_str(&after[..tag_end]);
        rest = &after[tag_end..];
    }
    out.push_str(&rest.replace(term, &replacement));
    out
}

fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
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
