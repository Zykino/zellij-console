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

pub fn format_ribbon_line(
    texts: Vec<Text>,
    y: usize,
    width: Option<usize>,
    height: Option<usize>,
) -> String {
    let x = 0;

    let (first, rest) = match texts.split_first() {
        Some(t) => t,
        None => return String::new(),
    };

    format!(
        "{}{}\u{1b}[48;5;{}m\u{1b}[0K",
        format_ribbon_with_coordinates(first, x, y, width, height),
        rest.iter().map(format_ribbon).collect::<String>(),
        crate::ui::BLACK // TODO: use same background as ribbon // FIXME: may not be generalizable as I did not manage to do the front of the line too
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
