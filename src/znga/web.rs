//! Web pages the person links in the Znga chat (specs/znga-chat.md,
//! REQ-ZCHAT-008): which links a message has, which sites the engine may
//! read, and the Link cards in an answer. The engine fetches the pages with
//! its own web tool; this module only decides what it may fetch.

/// Pages the engine may read for one question.
pub const MAX_PAGES: u32 = 8;

/// Link cards one answer may carry.
pub const MAX_LINKS: usize = 5;

/// Where the engine may read for one question.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Web {
    /// Sites (host names without `www.`), each with its subdomains.
    pub sites: Vec<String>,
    pub max_pages: u32,
}

impl Web {
    /// Reading limited to the sites of `links`; `None` when there are none.
    pub fn for_links(links: &[String]) -> Option<Self> {
        let mut sites: Vec<String> = Vec::new();
        for link in links {
            if let Some(site) = site(link)
                && !sites.contains(&site)
            {
                sites.push(site);
            }
        }
        (!sites.is_empty()).then_some(Self {
            sites,
            max_pages: MAX_PAGES,
        })
    }
}

/// The `http` and `https` links in `text`, in order, without repeats.
pub fn links(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        let word = word.trim_start_matches(['(', '[', '<', '"', '\'']);
        let lower = word.to_ascii_lowercase();
        if !(lower.starts_with("http://") || lower.starts_with("https://")) {
            continue;
        }
        let link = word.trim_end_matches(['.', ',', ';', ':', '!', '?', ')', ']', '>', '"', '\'']);
        if site(link).is_some() && !found.iter().any(|l| l == link) {
            found.push(link.to_owned());
        }
    }
    found
}

/// The site of an `http`/`https` link: its host, lower case, without a
/// leading `www.`, port or credentials. `None` for anything else.
pub fn site(link: &str) -> Option<String> {
    let lower = link.trim().to_ascii_lowercase();
    let rest = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))?;
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host = authority.rsplit('@').next().unwrap_or_default();
    let host = host.split(':').next().unwrap_or_default();
    let host = host.strip_prefix("www.").unwrap_or(host);
    let valid = host.contains('.')
        && !host.starts_with('.')
        && !host.ends_with('.')
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.');
    valid.then(|| host.to_owned())
}

/// A page the answer recommends: (title, address).
pub type Link = (String, String);

/// Splits `LINK: title | address` lines out of an answer. Only `http` and
/// `https` addresses are kept, at most [`MAX_LINKS`]; the rest of the
/// answer is returned trimmed.
pub fn split_links(answer: &str) -> (String, Vec<Link>) {
    let mut text: Vec<&str> = Vec::new();
    let mut found: Vec<Link> = Vec::new();
    for line in answer.lines() {
        let trimmed = line.trim().trim_start_matches(['-', '*', '•']).trim();
        let Some(rest) = trimmed
            .strip_prefix("LINK:")
            .or_else(|| trimmed.strip_prefix("Link:"))
        else {
            text.push(line);
            continue;
        };
        let (title, address) = match rest.rsplit_once('|') {
            Some((title, address)) => (title.trim(), address.trim()),
            None => ("", rest.trim()),
        };
        let address = address.trim_matches(['<', '>']);
        if site(address).is_none() || found.len() >= MAX_LINKS {
            continue;
        }
        if found.iter().any(|(_, a)| a == address) {
            continue;
        }
        let title = if title.is_empty() {
            site(address).unwrap_or_default()
        } else {
            title.to_owned()
        };
        found.push((title, address.to_owned()));
    }
    (text.join("\n").trim().to_owned(), found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_links_and_their_sites() {
        let text = "https://www.mistermobile.com.sg/trade-in/phone/apple-phone/\n\nfind a \
                    iphone from here (see http://shop.example.com:8080/a?b=1). Not ftp://x.y \
                    or www.plain.com";
        let found = links(text);
        assert_eq!(
            found,
            [
                "https://www.mistermobile.com.sg/trade-in/phone/apple-phone/",
                "http://shop.example.com:8080/a?b=1",
            ]
        );
        let web = Web::for_links(&found).unwrap();
        assert_eq!(web.sites, ["mistermobile.com.sg", "shop.example.com"]);
        assert_eq!(web.max_pages, MAX_PAGES);
        assert!(links("no links here").is_empty());
        assert_eq!(Web::for_links(&[]), None);
        assert_eq!(
            site("https://user:pw@WWW.Example.COM/x"),
            Some("example.com".into())
        );
        assert_eq!(site("https://localhost/x"), None);
        assert_eq!(site("javascript:alert(1)"), None);
    }

    #[test]
    fn splits_link_lines_from_the_answer() {
        let answer = "Two fit your budget:\n1. iPhone 12 64GB, S$280\n\n\
                      LINK: iPhone 12 64GB | https://shop.example.com/iphone-12\n\
                      - LINK: Bad | javascript:alert(1)\n\
                      LINK: https://shop.example.com/iphone-11\n\
                      LINK: again | https://shop.example.com/iphone-12";
        let (text, found) = split_links(answer);
        assert_eq!(text, "Two fit your budget:\n1. iPhone 12 64GB, S$280");
        assert_eq!(
            found,
            [
                (
                    "iPhone 12 64GB".to_owned(),
                    "https://shop.example.com/iphone-12".to_owned()
                ),
                (
                    "shop.example.com".to_owned(),
                    "https://shop.example.com/iphone-11".to_owned()
                ),
            ]
        );
        let many: String = (0..9)
            .map(|i| format!("LINK: p{i} | https://e.com/{i}\n"))
            .collect();
        assert_eq!(split_links(&many).1.len(), MAX_LINKS);
    }
}
