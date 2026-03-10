#![allow(dead_code)]
use ratatui::style::{Color, Modifier, Style};

pub const PRIMARY: Color = Color::Rgb(255, 26, 70);
pub const ACCENT: Color = Color::Yellow;
pub const POSITIVE: Color = Color::Green;
pub const NEGATIVE: Color = Color::Red;
pub const MUTED: Color = Color::DarkGray;
pub const SELECTED_BG: Color = Color::Rgb(40, 40, 60);

pub fn header_style() -> Style {
    Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD)
}

pub fn tab_active() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(PRIMARY)
        .add_modifier(Modifier::BOLD)
}

pub fn tab_inactive() -> Style {
    Style::default().fg(Color::White)
}

pub fn selected_row() -> Style {
    Style::default().bg(SELECTED_BG).fg(ACCENT)
}

pub fn value_style() -> Style {
    Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

pub fn muted_style() -> Style {
    Style::default().fg(MUTED)
}

pub fn negative_style() -> Style {
    Style::default().fg(NEGATIVE)
}

pub fn format_usd(value: f64) -> String {
    if value < 0.0 {
        format!("-${}", format_number(-value, 2))
    } else {
        format!("${}", format_number(value, 2))
    }
}

pub fn format_number(value: f64, decimals: usize) -> String {
    let s = format!("{:.prec$}", value, prec = decimals);
    let parts: Vec<&str> = s.split('.').collect();
    let int_part = parts[0];

    let negative = int_part.starts_with('-');
    let digits = if negative { &int_part[1..] } else { int_part };

    let with_commas: String = digits
        .chars()
        .rev()
        .enumerate()
        .map(|(i, c)| {
            if i > 0 && i % 3 == 0 {
                format!("{},", c)
            } else {
                c.to_string()
            }
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    if parts.len() > 1 {
        if negative {
            format!("-{}.{}", with_commas, parts[1])
        } else {
            format!("{}.{}", with_commas, parts[1])
        }
    } else if negative {
        format!("-{}", with_commas)
    } else {
        with_commas
    }
}

pub fn format_balance(value: f64) -> String {
    if value.abs() >= 1000.0 {
        format_number(value, 2)
    } else if value.abs() >= 1.0 {
        format_number(value, 4)
    } else {
        format_number(value, 6)
    }
}

pub fn format_percentage(value: f64) -> String {
    format!("{:.1}%", value)
}
