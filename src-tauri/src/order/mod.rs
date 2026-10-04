//! Fractional order keys (base 62), the same scheme as `src/lib/util/order-key.ts`.

#[cfg(test)]
mod tests;

const DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const BASE: usize = 62;

fn digit(c: u8) -> usize {
    DIGITS.iter().position(|&d| d == c).unwrap_or(0)
}

/// A key that sorts after `a` and before `b` (`None` = no neighbour on that side).
pub fn between(a: Option<&str>, b: Option<&str>) -> String {
    let lo = a.unwrap_or("").as_bytes();
    let hi = b.map(str::as_bytes);
    let mut out = Vec::new();
    let mut open = hi.is_none();
    let mut i = 0;
    // Ends once a digit fits between the bounds; at the latest when `lo` runs out, where the
    // open upper bound leaves room. Callers pass `a < b`.
    loop {
        let da = lo.get(i).map_or(0, |&c| digit(c));
        let db = if open {
            BASE
        } else {
            hi.and_then(|h| h.get(i)).map_or(BASE, |&c| digit(c))
        };
        if da == db {
            out.push(DIGITS[da]);
            i += 1;
            continue;
        }
        if db - da > 1 {
            out.push(DIGITS[(da + db) / 2]);
            return String::from_utf8(out).unwrap_or_default();
        }
        out.push(DIGITS[da]);
        open = true;
        i += 1;
    }
}

/// A key before `first` (or the first key ever).
pub fn before(first: Option<&str>) -> String {
    between(None, first)
}
