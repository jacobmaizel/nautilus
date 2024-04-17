pub mod extractors;
#[cfg(test)]
pub mod tests;
use regex::Regex;

pub fn format_slug(base: String) -> String {
    let s = base.replace(' ', "-");
    let re = Regex::new(r"[^a-zA-Z0-9-]").unwrap();
    let slug = re.replace_all(&s, "");
    slug.to_lowercase().to_string()
}

pub fn clean_username(username: String) -> String {
    let s = username.replace(' ', "-");
    let re = Regex::new(r"[^a-zA-Z0-9-]").unwrap();
    let res = re.replace_all(&s, "");
    res.to_lowercase().to_string()
}
