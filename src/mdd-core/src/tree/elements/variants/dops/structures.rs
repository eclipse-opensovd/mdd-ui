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

use super::kv_row;
use crate::tree::types::{
    CellType, ColumnConstraint, DetailCell, DetailContent, DetailRow, DetailSectionData,
    DetailSectionType,
};

/// Build tabbed sections for Structure DOP: Overview + Params tabs
pub(super) fn build_structure_dop_tabs(
    structure: &cda_database::datatypes::StructureDop<'_>,
    types_rows: &mut Vec<DetailRow>,
    sections: &mut Vec<DetailSectionData>,
) {
    let mut overview_rows = vec![kv_row(
        "Is Visible",
        structure.is_visible().to_string(),
        CellType::Text,
        0,
    )];

    if let Some(byte_size) = structure.byte_size() {
        overview_rows.push(kv_row(
            "Byte Size",
            byte_size.to_string(),
            CellType::NumericValue,
            0,
        ));
    }

    if let Some(params) = structure.params() {
        overview_rows.push(kv_row(
            "Param Count",
            params.len().to_string(),
            CellType::NumericValue,
            0,
        ));
    }

    // Show the generic type info (Short Name, DOP Variant, ...) first in the overview
    let mut rows = std::mem::take(types_rows);
    rows.extend(overview_rows);
    let overview_rows = rows;

    let overview_header = DetailRow::header(vec![
        DetailCell::text("Property"),
        DetailCell::text("Value"),
    ]);

    sections.push(
        DetailSectionData::new(
            "Overview".to_owned(),
            DetailContent::Table {
                header: overview_header,
                rows: overview_rows,
                constraints: vec![
                    ColumnConstraint::Percentage(40),
                    ColumnConstraint::Percentage(60),
                ],
                use_row_selection: false,
            },
            false,
        )
        .with_type(DetailSectionType::Overview),
    );

    sections.push(build_params_section(structure));
}

pub(super) fn build_params_section(
    structure: &cda_database::datatypes::StructureDop<'_>,
) -> DetailSectionData {
    let Some(params) = structure.params() else {
        return DetailSectionData {
            title: "Params".to_owned(),
            render_as_header: false,
            section_type: DetailSectionType::Requests,
            content: DetailContent::PlainText(vec!["No params".to_owned()]),
            byte_pattern_rows: None,
        };
    };
    crate::tree::elements::variants::params::build_param_section(
        "Params",
        params.iter().map(cda_database::datatypes::Parameter),
        DetailSectionType::Requests,
    )
}
