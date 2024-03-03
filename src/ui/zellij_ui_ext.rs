use std::borrow::Borrow;

use zellij_tile::prelude::ui_components::*;

pub fn format_text(text: Text) -> String {
    format!("\u{1b}Pztext;{}\u{1b}\\", text.serialize())
}

pub fn format_text_with_coordinates(
    text: Text,
    x: usize,
    y: usize,
    width: Option<usize>,
    height: Option<usize>,
) -> String {
    let width = width.map(|w| w.to_string()).unwrap_or_default();
    let height = height.map(|h| h.to_string()).unwrap_or_default();
    format!(
        "\u{1b}Pztext;{}/{}/{}/{};{}\u{1b}\\",
        x,
        y,
        width,
        height,
        text.serialize()
    )
}

pub fn format_ribbon(text: &Text) -> String {
    format!("\u{1b}Pzribbon;{}\u{1b}\\", text.serialize())
}

pub fn format_ribbon_with_coordinates(
    text: &Text,
    x: usize,
    y: usize,
    width: Option<usize>,
    height: Option<usize>,
) -> String {
    let width = width.map(|w| w.to_string()).unwrap_or_default();
    let height = height.map(|h| h.to_string()).unwrap_or_default();

    format!(
        "\u{1b}Pzribbon;{}/{}/{}/{};{}\u{1b}\\",
        x,
        y,
        width,
        height,
        text.serialize()
    )
}

pub fn format_ribbon_line<I>(ribbons: I) -> String
where
    I: IntoIterator,
    I::Item: Borrow<Text>,
{
    ribbons
        .into_iter()
        .map(|r| format_ribbon(r.borrow()))
        .collect()
}

pub fn format_ribbon_line_with_coordinates<I>(
    ribbons: I,
    x: usize,
    y: usize,
    width: Option<usize>,
    height: Option<usize>,
) -> String
where
    I: IntoIterator,
    I::Item: Borrow<Text>,
{
    let mut ribbons = ribbons.into_iter();
    let Some(first) = ribbons.next() else {
        return String::new();
    };

    let mut result = format_ribbon_with_coordinates(first.borrow(), x, y, width, height);
    result.push_str(&format_ribbon_line(ribbons));
    result
}

pub fn format_background_until_line_end(s: String, color: u8) -> String {
    format!("{}\u{1b}[48;5;{}m\u{1b}[0K", s, color)
}

pub fn format_ribbon_full_line<I>(ribbons: I) -> String
where
    I: IntoIterator,
    I::Item: Borrow<Text>,
{
    format_background_until_line_end(
        format_ribbon_line(ribbons),
        0, // TODO: use same background as ribbon // FIXME: may not be generalizable as I did not manage to do the front of the line too
    )
}

pub fn format_ribbon_full_line_with_coordinates<I>(
    ribbons: I,
    x: usize,
    y: usize,
    width: Option<usize>,
    height: Option<usize>,
) -> String
where
    I: IntoIterator,
    I::Item: Borrow<Text>,
{
    format_background_until_line_end(
        format_ribbon_line_with_coordinates(ribbons, x, y, width, height),
        super::BLACK, // TODO: use same background as ribbon // FIXME: may not be generalizable as I did not manage to do the front of the line too
    )
}

pub fn format_nested_list(items: Vec<NestedListItem>) -> String {
    let items = items
        .into_iter()
        .map(|i| i.serialize())
        .collect::<Vec<_>>()
        .join(";");
    format!("\u{1b}Pznested_list;{}\u{1b}\\", items)
}

pub fn format_nested_list_with_coordinates(
    items: Vec<NestedListItem>,
    x: usize,
    y: usize,
    width: Option<usize>,
    height: Option<usize>,
) -> String {
    let width = width.map(|w| w.to_string()).unwrap_or_default();
    let height = height.map(|h| h.to_string()).unwrap_or_default();
    let items = items
        .into_iter()
        .map(|i| i.serialize())
        .collect::<Vec<_>>()
        .join(";");
    format!(
        "\u{1b}Pznested_list;{}/{}/{}/{};{}\u{1b}\\",
        x, y, width, height, items
    )
}

pub fn format_table(table: Table) -> String {
    format!("\u{1b}Pztable;{}", table.serialize())
}

pub fn format_table_with_coordinates(
    table: Table,
    x: usize,
    y: usize,
    width: Option<usize>,
    height: Option<usize>,
) -> String {
    let width = width.map(|w| w.to_string()).unwrap_or_default();
    let height = height.map(|h| h.to_string()).unwrap_or_default();
    format!(
        "\u{1b}Pztable;{}/{}/{}/{};{}\u{1b}\\",
        x,
        y,
        width,
        height,
        table.serialize()
    )
}
