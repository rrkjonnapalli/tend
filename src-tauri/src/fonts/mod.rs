//! The font families installed on this device, system and user, for the note view's reading font.

#[cfg(test)]
mod tests;

/// Every family name, sorted the way a reader looks one up.
pub fn families() -> Vec<String> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    // A face's first family name is its English one; the rest are translations of it.
    let names = db.faces().filter_map(|face| face.families.first().map(|(name, _)| name.clone()));
    listed(names)
}

/// Sorted without case, each once, without the system's hidden families (".SF NS").
fn listed(names: impl Iterator<Item = String>) -> Vec<String> {
    let mut out: Vec<String> = names.filter(|name| !name.starts_with('.')).collect();
    out.sort_by_key(|name| name.to_lowercase());
    out.dedup();
    out
}
