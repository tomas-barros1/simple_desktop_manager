use crate::models::DesktopEntry;

/// Case-insensitive substring filter against the user-visible searchable
/// fields: name, generic_name, comment, exec, and categories.
#[allow(dead_code)]
pub fn matches(entry: &DesktopEntry, query: &str) -> bool {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return true;
    }
    let fields = [
        &entry.name,
        &entry.generic_name,
        &entry.comment,
        &entry.exec,
        &entry.categories.join(" "),
        &entry.keywords.join(" "),
    ];
    fields
        .iter()
        .any(|field| field.to_ascii_lowercase().contains(&query))
}

/// Apply the filter to a slice and return matched entries (preserving order).
#[allow(dead_code)]
pub fn filter<'a>(entries: &'a [DesktopEntry], query: &str) -> Vec<&'a DesktopEntry> {
    entries.iter().filter(|e| matches(e, query)).collect()
}
