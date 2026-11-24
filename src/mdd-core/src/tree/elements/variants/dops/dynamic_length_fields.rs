/*
 * SPDX-License-Identifier: Apache-2.0
 * SPDX-FileCopyrightText: 2026 Copyright (c) Contributors to the Eclipse Foundation
 *
 * See the NOTICE file(s) distributed with this work for additional
 * information regarding copyright ownership.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Apache License Version 2.0 which is available at
 * https://www.apache.org/licenses/LICENSE-2.0
 */

use super::{kv_row, push_types_section};
use crate::tree::types::{CellType, DetailRow, DetailSectionData};

/// Build tabbed sections for `DynamicLengthField` DOP
/// Shows bit/byte position and data object prop ref (link), not DOP variant/short name
pub(super) fn build_dynamic_length_field_dop_tabs(
    dyn_field: &cda_database::datatypes::DynamicLengthDop<'_>,
    types_rows: &mut Vec<DetailRow>,
    sections: &mut Vec<DetailSectionData>,
) {
    // Remove DOP Variant and Short Name rows -- not wanted for dynamic length fields
    types_rows.retain(|row| {
        let key = row.cells.first().map(|c| c.text.as_str());
        !matches!(key, Some("DOP Variant" | "Short Name"))
    });

    let offset = dyn_field.offset();
    let det = dyn_field.determine_number_of_items();

    // Summary line explaining how the total length is derived at decode time.
    let display_name = |name: &str| {
        let display = super::parse_dop_name(name).display_name();
        if display.is_empty() {
            name.to_owned()
        } else {
            display
        }
    };
    let struct_name = dyn_field
        .field()
        .and_then(|f| f.basic_structure())
        .map_or_else(
            || "?".to_owned(),
            |b| display_name(b.short_name().unwrap_or("?")),
        );
    let count_source = det.as_ref().map_or_else(
        || "N = number of items (source unknown)".to_owned(),
        |d| {
            let dop_name = d.dop().map_or_else(
                || "?".to_owned(),
                |dop| display_name(dop.short_name().unwrap_or("?")),
            );
            format!(
                "N decoded via {dop_name} at byte {}, bit {} (relative to field start)",
                d.byte_position(),
                d.bit_position()
            )
        },
    );
    types_rows.push(kv_row(
        "Length",
        format!("{offset} + N \u{d7} sizeof({struct_name}); {count_source}"),
        CellType::Text,
        0,
    ));

    types_rows.push(kv_row(
        "First Item Offset (bytes)",
        offset.to_string(),
        CellType::NumericValue,
        0,
    ));

    if let Some(det) = det {
        types_rows.push(kv_row(
            "Item Count \u{203a} Byte Position",
            det.byte_position().to_string(),
            CellType::NumericValue,
            0,
        ));
        types_rows.push(kv_row(
            "Item Count \u{203a} Bit Position",
            det.bit_position().to_string(),
            CellType::NumericValue,
            0,
        ));

        if let Some(dop) = det.dop() {
            let dop_name = dop.short_name().unwrap_or("?").to_owned();
            types_rows.push(super::dop_ref_row("Item Count \u{203a} DOP", &dop_name));
        }
    }

    let mut extra_sections = Vec::new();
    super::push_basic_structure!(dyn_field.field(), types_rows, extra_sections);

    push_types_section(std::mem::take(types_rows), sections);
    sections.extend(extra_sections);
}
