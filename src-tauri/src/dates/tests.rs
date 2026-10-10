use super::*;

#[test]
fn times_are_24_hour_hh_mm() {
    for ok in ["00:00", "09:05", "23:59"] {
        assert!(is_time(ok), "{ok}");
    }
    for bad in ["24:00", "29:59", "9:05", "12:60", "12-30", "", "ab:cd"] {
        assert!(!is_time(bad), "{bad}");
    }
}

#[test]
fn dates_are_real_calendar_days() {
    for ok in ["2026-10-10", "2024-02-29", "2026-12-31"] {
        assert!(is_date(ok), "{ok}");
    }
    for bad in [
        "2026-02-29",
        "2026-02-31",
        "2026-13-01",
        "2026-1-01",
        "2026-10-1",
        "",
        "10/10/2026",
    ] {
        assert!(!is_date(bad), "{bad}");
    }
}
