use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;
use std::process;

// --- CONFIGURATION ---
// Change this to the path of your Obsidian file.
// You can use a tilde `~` for the home directory if using the shellexpand crate,
// otherwise use an absolute path.
const MARKDOWN_FILE_PATH: &str = "~/Documents/obsidian-vault/Bookmarks.md";

struct Bookmark {
    title: String,
    url: String,
    tags: String,
}

impl Bookmark {
    /// Formats the string displayed in Rofi
    fn to_rofi_string(&self) -> String {
        // We combine title and tags so you can search by tag in Rofi
        if self.tags.is_empty() {
            self.title.clone()
        } else {
            format!("{} {}", self.title, self.tags)
        }
    }
}

fn main() {
    // 1. Resolve the file path (handling ~ expansion)
    let path_str = shellexpand::tilde(MARKDOWN_FILE_PATH).to_string();
    let path = Path::new(&path_str);

    if !path.exists() {
        eprintln!("Error: File not found at {}", path_str);
        process::exit(1);
    }

    // 2. Parse the bookmarks
    let content = fs::read_to_string(path).unwrap_or_else(|_| {
        eprintln!("Error: Could not read file");
        process::exit(1);
    });
    
    let bookmarks = parse_markdown(&content);
    let args: Vec<String> = env::args().collect();

    // 3. Handle Rofi Logic
    if args.len() == 1 {
        // CASE A: No arguments provided. 
        // Print the list of bookmarks to stdout (for Rofi to display).
        for bookmark in &bookmarks {
            println!("{}", bookmark.to_rofi_string());
        }
    } else {
        // CASE B: Argument provided.
        // The user selected a line. Rofi passes that exact line as the first argument.
        let selection = &args[1];
        
        // Find the bookmark that produced this display string
        if let Some(target) = bookmarks.iter().find(|b| b.to_rofi_string() == *selection) {
            // Open the URL in the default browser
            if let Err(e) = open::that(&target.url) {
                eprintln!("Failed to open URL: {}", e);
            }
        } else {
            // Fallback: If nothing matched, just print the list again (prevents crashing)
            eprintln!("Selection not found in original file.");
        }
    }
}

fn parse_markdown(content: &str) -> Vec<Bookmark> {
    // Regex breakdown:
    // ^\s*-\s* : Matches the starting dash and optional whitespace
    // \[(?P<t>.*?)\] : Captures the Title inside []
    // \((?P<u>.*?)\) : Captures the URL inside ()
    // \s*(?P<r>.*)   : Captures the rest (tags/icons)
    let re = Regex::new(r"^\s*-\s*\[(?P<title>.*?)\]\((?P<url>.*?)\)\s*(?P<tags>.*)").unwrap();
    
    content
        .lines()
        .filter_map(|line| {
            re.captures(line).map(|caps| Bookmark {
                title: caps["title"].trim().to_string(),
                url: caps["url"].trim().to_string(),
                tags: caps["tags"].trim().to_string(),
            })
        })
        .collect()
}
