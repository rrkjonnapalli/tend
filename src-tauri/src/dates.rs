//! Local dates (`YYYY-MM-DD`) and times (`HH:MM`, 24-hour) as the database keeps them: the one
//! check for both, used by tasks (due) and people (birth).

#[cfg(test)]
mod tests;

fn is_leap(year: i64) -> bool {
    let every_fourth = year % 4 == 0 && year % 100 != 0;
    every_fourth || year % 400 == 0
}

/// Without a year, February allows the 29th.
pub fn days_in_month(year: Option<i64>, month: i64) -> i64 {
    match month {
        2 => match year {
            Some(y) if !is_leap(y) => 28,
            _ => 29,
        },
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// `HH:MM`, 24-hour: `17:30` yes, `29:59` or `7:30` no.
pub fn is_time(t: &str) -> bool {
    let shaped = t.len() == 5 && t.as_bytes()[2] == b':';
    if !shaped {
        return false;
    }
    let hour_ok = t[..2].parse::<u8>().is_ok_and(|h| h < 24);
    let minute_ok = t[3..].parse::<u8>().is_ok_and(|m| m < 60);
    hour_ok && minute_ok
}

/// A real calendar day as `YYYY-MM-DD`: `2026-02-31` no.
pub fn is_date(d: &str) -> bool {
    let parts: Vec<&str> = d.split('-').collect();
    let [y, m, day] = parts.as_slice() else {
        return false;
    };
    let shaped = y.len() == 4 && m.len() == 2 && day.len() == 2;
    let (Ok(year), Ok(month), Ok(dd)) = (y.parse::<i64>(), m.parse::<i64>(), day.parse::<i64>())
    else {
        return false;
    };
    let month_ok = (1..=12).contains(&month);
    shaped && year >= 1 && month_ok && (1..=days_in_month(Some(year), month)).contains(&dd)
}
