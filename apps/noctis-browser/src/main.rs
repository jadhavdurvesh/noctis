use noctis_browser::{normalize_address, BlockKind, DocumentView};
use noctis_net::{fetch_url, FetchResponse};
use std::io::{self, Write};

fn usage() {
    println!(
        "NOCTIS Browser 0.1.0\n\
         Usage: noctis-browser [URL]\n\
         \n\
         Open a web page, read its content, and follow links from the terminal.\n\
         Bare hostnames default to HTTPS. Only HTTP and HTTPS are supported.\n\
         \n\
         Interactive commands:\n\
           <number>   Open a numbered page link\n\
           open URL   Navigate to a URL\n\
           b          Go back\n\
           f          Go forward\n\
           r          Reload current page\n\
           h          Show command help\n\
           q          Quit"
    );
}

fn render_page(final_url: &str, response: &FetchResponse, view: &DocumentView) {
    println!("\n{}", "=".repeat(76));
    println!("NOCTIS  |  {}", view.title);
    println!("{}", "-".repeat(76));
    println!("URL: {final_url}");
    println!("HTTP: {}", response.status);
    if let Some(content_type) = &response.content_type {
        println!("Type: {content_type}");
    }
    println!("{}", "-".repeat(76));

    for block in &view.blocks {
        match block.kind {
            BlockKind::Heading(level) => {
                println!(
                    "\n{} {}",
                    "#".repeat(level as usize),
                    block.text.to_uppercase()
                );
            }
            BlockKind::Paragraph | BlockKind::Other => println!("\n{}", block.text),
            BlockKind::ListItem => println!("  • {}", block.text),
            BlockKind::Preformatted => println!("\n    {}", block.text),
            BlockKind::Quote => println!("\n  > {}", block.text),
        }
    }

    if !view.links.is_empty() {
        println!("\n{}", "-".repeat(76));
        println!("LINKS");
        for link in &view.links {
            println!("  [{}] {}\n      {}", link.number, link.text, link.url);
        }
    }
    println!("{}", "=".repeat(76));
}

fn load_page(url: &str) -> Result<(FetchResponse, DocumentView), String> {
    let response = fetch_url(url).map_err(|error| error.to_string())?;
    let final_url = response.final_url.clone();
    let content_type = response
        .content_type
        .as_deref()
        .unwrap_or("")
        .to_ascii_lowercase();

    if content_type.contains("text/plain") {
        let text = String::from_utf8_lossy(&response.body);
        let view = DocumentView::from_plain_text(&text, &final_url);
        return Ok((response, view));
    }

    if !content_type.is_empty()
        && !(content_type.contains("text/html") || content_type.contains("application/xhtml+xml"))
    {
        return Err(format!(
            "NOCTIS alpha can display HTML and plain text, not {content_type}"
        ));
    }

    let html = String::from_utf8_lossy(&response.body);
    let view = DocumentView::from_html(&html, &final_url);
    Ok((response, view))
}

fn push_history(history: &mut Vec<String>, index: &mut usize, url: String) {
    if history.get(*index).is_some_and(|current| current == &url) {
        return;
    }
    history.truncate(index.saturating_add(1));
    history.push(url);
    *index = history.len() - 1;
}

fn prompt_line(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_owned())
}

fn main() {
    let mut args = std::env::args().skip(1);
    let first = args.next();
    if first
        .as_deref()
        .is_some_and(|arg| arg == "-h" || arg == "--help")
    {
        usage();
        return;
    }
    if first
        .as_deref()
        .is_some_and(|arg| arg == "-V" || arg == "--version")
    {
        println!("NOCTIS Browser 0.1.0");
        return;
    }
    if args.next().is_some() {
        eprintln!("noctis-browser: only one starting URL can be supplied");
        std::process::exit(2);
    }

    let start = match first {
        Some(value) => value,
        None => match prompt_line("NOCTIS address> ") {
            Ok(value) => value,
            Err(error) => {
                eprintln!("noctis-browser: input error: {error}");
                std::process::exit(1);
            }
        },
    };
    let start = match normalize_address(&start) {
        Ok(url) => url,
        Err(error) => {
            eprintln!("noctis-browser: {error}");
            std::process::exit(2);
        }
    };

    let mut history = vec![start];
    let mut history_index = 0usize;

    loop {
        let current_url = history[history_index].clone();
        println!("\nOpening {current_url} ...");
        let current_view = match load_page(&current_url) {
            Ok((response, view)) => {
                render_page(&response.final_url, &response, &view);
                Some(view)
            }
            Err(error) => {
                eprintln!("\nNOCTIS navigation error: {error}");
                None
            }
        };

        let command = match prompt_line("\nnoctis> ") {
            Ok(command) => command,
            Err(error) => {
                eprintln!("noctis-browser: input error: {error}");
                break;
            }
        };
        if command.is_empty() {
            continue;
        }

        match command.as_str() {
            "q" | "quit" | "exit" => break,
            "h" | "help" | "?" => usage(),
            "b" | "back" => {
                if history_index > 0 {
                    history_index -= 1;
                } else {
                    println!("No previous page.");
                }
            }
            "f" | "forward" => {
                if history_index + 1 < history.len() {
                    history_index += 1;
                } else {
                    println!("No forward page.");
                }
            }
            "r" | "reload" => {}
            value => {
                let target = value.strip_prefix("open ").map(str::trim);

                if let Some(url) = target {
                    match normalize_address(url) {
                        Ok(url) => push_history(&mut history, &mut history_index, url),
                        Err(error) => eprintln!("Invalid URL: {error}"),
                    }
                } else if let Ok(number) = value.parse::<usize>() {
                    if let Some(view) = &current_view {
                        if let Some(link) = view.links.iter().find(|link| link.number == number) {
                            let target = link.url.clone();
                            push_history(&mut history, &mut history_index, target);
                        } else {
                            println!("No link numbered {number} on this page.");
                        }
                    } else {
                        println!("No page is currently loaded.");
                    }
                } else {
                    match normalize_address(value) {
                        Ok(url) => push_history(&mut history, &mut history_index, url),
                        Err(error) => eprintln!("Unknown command or invalid URL: {error}"),
                    }
                }
            }
        }
    }

    println!("NOCTIS session closed.");
}
