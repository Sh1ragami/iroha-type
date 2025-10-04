use unicode_width::UnicodeWidthStr;

pub fn display_width(s: &str) -> usize { UnicodeWidthStr::width(s) }

