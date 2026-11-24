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

use cda_database::datatypes::{DiagService, Parameter};

use super::{
    dops::parse_dop_name,
    format_service_id,
    services::{extract_coded_value, extract_dop_badge, extract_dop_name},
};
use crate::tree::types::{
    BIT_POSITION_UNSET, CellJumpTarget, CellJumpTargetType, CellType, ColumnConstraint, DetailCell,
    DetailContent, DetailRow, DetailSectionData, DetailSectionType, param_type_label,
};

/// Build a key-value row for a DOP reference that is navigable via Enter.
/// Uses the parsed display name when available so the navigation target
/// matches the tree node label produced by `add_dops_section`.
fn dop_kv_row(dop_name: &str) -> DetailRow {
    let parsed = parse_dop_name(dop_name);
    let display = parsed.display_name();
    let nav_name = if display.is_empty() {
        dop_name.to_owned()
    } else {
        display
    };
    DetailRow::normal(
        vec![
            DetailCell::text("DOP"),
            DetailCell::new(nav_name.clone(), CellType::DopReference).with_jump(
                CellJumpTarget::new(CellJumpTargetType::Dop {
                    index: usize::MAX,
                    name: nav_name,
                }),
            ),
        ],
        0,
    )
}

/// Append detail rows for a `DiagCodedType` flatbuf value.  Implemented as a
/// macro because the underlying flatbuf types are crate-private and cannot be
/// named in function signatures.
macro_rules! append_dct_rows {
    ($dct:expr, $rows:expr) => {{
        let dct = &$dct;
        $rows.push(DetailRow::normal(
            vec![
                DetailCell::text("DCT Type"),
                DetailCell::text(format!("{:?}", dct.type_())),
            ],
            0,
        ));

        if let Some(enc) = dct.base_type_encoding() {
            $rows.push(DetailRow::normal(
                vec![
                    DetailCell::text("Base Type Encoding"),
                    DetailCell::text(enc),
                ],
                0,
            ));
        }

        $rows.push(DetailRow::normal(
            vec![
                DetailCell::text("Base Data Type"),
                DetailCell::text(format!("{:?}", dct.base_data_type())),
            ],
            0,
        ));

        $rows.push(DetailRow::normal(
            vec![
                DetailCell::text("High-Low Byte Order"),
                DetailCell::text(dct.is_high_low_byte_order().to_string()),
            ],
            0,
        ));

        let length_type = format!("{:?}", dct.specific_data_type());
        $rows.push(DetailRow::normal(
            vec![
                DetailCell::text("Length Type"),
                DetailCell::text(length_type),
            ],
            0,
        ));

        if let Some(slt) = dct.specific_data_as_standard_length_type() {
            $rows.push(DetailRow::normal(
                vec![
                    DetailCell::text("Bit Length"),
                    DetailCell::new(slt.bit_length().to_string(), CellType::NumericValue),
                ],
                0,
            ));
            let mask_str = slt
                .bit_mask()
                .map_or_else(|| "None".to_owned(), |m| format!("{m:?}"));
            $rows.push(DetailRow::normal(
                vec![DetailCell::text("Bit Mask"), DetailCell::text(mask_str)],
                0,
            ));
            $rows.push(DetailRow::normal(
                vec![
                    DetailCell::text("Condensed"),
                    DetailCell::text(slt.condensed().to_string()),
                ],
                0,
            ));
        } else if let Some(mml) = dct.specific_data_as_min_max_length_type() {
            $rows.push(DetailRow::normal(
                vec![
                    DetailCell::text("Min Length"),
                    DetailCell::new(mml.min_length().to_string(), CellType::NumericValue),
                ],
                0,
            ));
            let max = mml
                .max_length()
                .map_or_else(|| "None".to_owned(), |v| v.to_string());
            $rows.push(DetailRow::normal(
                vec![
                    DetailCell::text("Max Length"),
                    DetailCell::new(max, CellType::NumericValue),
                ],
                0,
            ));
            $rows.push(DetailRow::normal(
                vec![
                    DetailCell::text("Termination"),
                    DetailCell::text(format!("{:?}", mml.termination())),
                ],
                0,
            ));
        } else if let Some(lli) = dct.specific_data_as_leading_length_info_type() {
            $rows.push(DetailRow::normal(
                vec![
                    DetailCell::text("Bit Length"),
                    DetailCell::new(lli.bit_length().to_string(), CellType::NumericValue),
                ],
                0,
            ));
        } else if let Some(pli) = dct.specific_data_as_param_length_info_type() {
            let key_name = pli.length_key().and_then(|p| p.short_name()).unwrap_or("-");
            $rows.push(DetailRow::normal(
                vec![
                    DetailCell::text("Length Key Param"),
                    DetailCell::text(key_name),
                ],
                0,
            ));
        }
    }};
}

/// Build detail sections for a single parameter (Overview with key-value
/// properties).  Shared by both request and response parameter views.
pub fn build_param_detail_sections(param: &Parameter<'_>) -> Vec<DetailSectionData> {
    let mut sections = Vec::new();

    let param_name = param.short_name().unwrap_or("?");
    sections.push(DetailSectionData {
        title: format!("Parameter - {param_name}"),
        render_as_header: true,
        section_type: DetailSectionType::Header,
        content: DetailContent::PlainText(vec![]),
        byte_pattern_rows: None,
    });

    let mut overview_rows = Vec::new();
    overview_rows.push(DetailRow::normal(
        vec![
            DetailCell::text("ID"),
            DetailCell::new(param.id().to_string(), CellType::NumericValue),
        ],
        0,
    ));

    if let Some(short_name) = param.short_name() {
        overview_rows.push(DetailRow::normal(
            vec![DetailCell::text("Short Name"), DetailCell::text(short_name)],
            0,
        ));
    }

    if let Ok(param_type) = param.param_type() {
        overview_rows.push(DetailRow::normal(
            vec![
                DetailCell::text("Type"),
                DetailCell::text(param_type_label(&param_type)),
            ],
            0,
        ));
    }

    if let Some(semantic) = param.semantic() {
        overview_rows.push(DetailRow::normal(
            vec![DetailCell::text("Semantic"), DetailCell::text(semantic)],
            0,
        ));
    }

    overview_rows.push(DetailRow::normal(
        vec![
            DetailCell::text("Byte Position"),
            DetailCell::new(param.byte_position().to_string(), CellType::NumericValue),
        ],
        0,
    ));

    let bit_pos = param.bit_position();
    overview_rows.push(DetailRow::normal(
        vec![
            DetailCell::text("Bit Position"),
            DetailCell::new(
                if bit_pos == BIT_POSITION_UNSET {
                    "unset".to_owned()
                } else {
                    bit_pos.to_string()
                },
                CellType::NumericValue,
            ),
        ],
        0,
    ));

    if let Some(pdv) = param.physical_default_value() {
        overview_rows.push(DetailRow::normal(
            vec![
                DetailCell::text("Physical Default Value"),
                DetailCell::text(pdv),
            ],
            0,
        ));
    }

    let coded_value = extract_coded_value(param);
    if !coded_value.is_empty() {
        overview_rows.push(DetailRow::normal(
            vec![
                DetailCell::text("Coded Value"),
                DetailCell::new(coded_value, CellType::NumericValue),
            ],
            0,
        ));
    }

    let dop_name = extract_dop_name(param);
    if !dop_name.is_empty() {
        overview_rows.push(dop_kv_row(&dop_name));
    }

    let header = DetailRow::header(vec![
        DetailCell::text("Property"),
        DetailCell::text("Value"),
    ]);

    sections.push(
        DetailSectionData::new(
            "Overview".to_owned(),
            DetailContent::Table {
                header,
                rows: overview_rows,
                constraints: vec![
                    ColumnConstraint::Percentage(40),
                    ColumnConstraint::Percentage(60),
                ],
                use_row_selection: true,
            },
            false,
        )
        .with_type(DetailSectionType::Overview),
    );

    if let Some(specific_section) = build_specific_section(param) {
        sections.push(specific_section);
    }

    sections
}

fn build_specific_section(param: &Parameter<'_>) -> Option<DetailSectionData> {
    let specific_rows = build_specific_data_rows(param);
    if specific_rows.is_empty() {
        return None;
    }
    let title = param.param_type().map_or_else(
        |_| "Specific Data".to_owned(),
        |pt| param_type_label(&pt).to_owned(),
    );
    let header = DetailRow::header(vec![
        DetailCell::text("Property"),
        DetailCell::text("Value"),
    ]);
    Some(
        DetailSectionData::new(
            title,
            DetailContent::Table {
                header,
                rows: specific_rows,
                constraints: vec![
                    ColumnConstraint::Percentage(40),
                    ColumnConstraint::Percentage(60),
                ],
                use_row_selection: true,
            },
            false,
        )
        .with_type(DetailSectionType::Custom),
    )
}

/// Build rows for the type-specific data of a parameter.
fn build_specific_data_rows(param: &Parameter<'_>) -> Vec<DetailRow> {
    let mut rows = Vec::new();

    // CodedConst
    if let Some(cc) = param.specific_data_as_coded_const() {
        if let Some(cv) = cc.coded_value() {
            rows.push(DetailRow::normal(
                vec![DetailCell::text("Coded Value"), DetailCell::text(cv)],
                0,
            ));
        }
        if let Some(dct) = cc.diag_coded_type() {
            append_dct_rows!(dct, rows);
        }
        return rows;
    }

    // NrcConst
    if let Some(nrc) = param.specific_data_as_nrc_const() {
        if let Some(vals) = nrc.coded_values() {
            let values_str: Vec<&str> = vals.iter().collect();
            rows.push(DetailRow::normal(
                vec![
                    DetailCell::text("Coded Values"),
                    DetailCell::text(values_str.join(", ")),
                ],
                0,
            ));
        }
        if let Some(dct) = nrc.diag_coded_type() {
            append_dct_rows!(dct, rows);
        }
        return rows;
    }

    build_remaining_specific_rows(param)
}

/// Build type-specific rows for param types that do not use `DiagCodedType`.
fn build_remaining_specific_rows(param: &Parameter<'_>) -> Vec<DetailRow> {
    let mut rows = Vec::new();

    // MatchingRequestParam
    if let Some(mrp) = param.specific_data_as_matching_request_param() {
        rows.push(DetailRow::normal(
            vec![
                DetailCell::text("Request Byte Pos"),
                DetailCell::new(mrp.request_byte_pos().to_string(), CellType::NumericValue),
            ],
            0,
        ));
        rows.push(DetailRow::normal(
            vec![
                DetailCell::text("Byte Length"),
                DetailCell::new(mrp.byte_length().to_string(), CellType::NumericValue),
            ],
            0,
        ));
        return rows;
    }

    // PhysConst
    if let Some(pc) = param.specific_data_as_phys_const() {
        if let Some(v) = pc.phys_constant_value() {
            rows.push(DetailRow::normal(
                vec![DetailCell::text("Phys Constant Value"), DetailCell::text(v)],
                0,
            ));
        }
        if let Some(dop) = pc.dop() {
            rows.push(dop_kv_row(dop.short_name().unwrap_or("-")));
        }
        return rows;
    }

    // Reserved
    if let Some(res) = param.specific_data_as_reserved() {
        rows.push(DetailRow::normal(
            vec![
                DetailCell::text("Bit Length"),
                DetailCell::new(res.bit_length().to_string(), CellType::NumericValue),
            ],
            0,
        ));
        return rows;
    }

    // System
    if let Some(sys) = param.specific_data_as_system() {
        if let Some(sp) = sys.sys_param() {
            rows.push(DetailRow::normal(
                vec![DetailCell::text("Sys Param"), DetailCell::text(sp)],
                0,
            ));
        }
        if let Some(dop) = sys.dop() {
            rows.push(dop_kv_row(dop.short_name().unwrap_or("-")));
        }
        return rows;
    }

    // LengthKeyRef
    if let Some(lkr) = param.specific_data_as_length_key_ref() {
        if let Some(dop) = lkr.dop() {
            rows.push(dop_kv_row(dop.short_name().unwrap_or("-")));
        }
        return rows;
    }

    // TableEntry
    if let Some(te) = param.specific_data_as_table_entry() {
        rows.push(DetailRow::normal(
            vec![
                DetailCell::text("Target"),
                DetailCell::text(format!("{:?}", te.target())),
            ],
            0,
        ));
        if let Some(p) = te.param() {
            rows.push(DetailRow::normal(
                vec![
                    DetailCell::text("Entry Param"),
                    DetailCell::text(p.short_name().unwrap_or("-")),
                ],
                0,
            ));
        }
        if let Some(tr) = te.table_row()
            && let Some(sn) = tr.short_name()
        {
            rows.push(DetailRow::normal(
                vec![DetailCell::text("Table Row"), DetailCell::text(sn)],
                0,
            ));
        }
        return rows;
    }

    // TableStruct
    if let Some(ts) = param.specific_data_as_table_struct() {
        if let Some(key) = ts.table_key() {
            rows.push(DetailRow::normal(
                vec![
                    DetailCell::text("Table Key Param"),
                    DetailCell::text(key.short_name().unwrap_or("-")),
                ],
                0,
            ));
        }
        return rows;
    }

    rows
}

struct ParamInfo {
    name: String,
    byte_pos: u32,
    bit_pos: u32,
    bit_len: Option<u32>,
    value: String,
    dop_name: String,
    dop_badge: String,
    semantic: String,
    param_id: u32,
    /// Nesting depth (0 = top level, >0 = sub-element of a structure).
    depth: usize,
    /// Byte size of the referenced structure, if known.
    struct_byte_size: Option<u32>,
}

/// Maximum nesting depth when expanding structure sub-elements.
const MAX_STRUCT_DEPTH: usize = 8;

/// Return the structure DOP referenced by a `Value` parameter, if any.
/// For field DOPs (static / dynamic length / end-of-PDU) the basic
/// structure (one repeated item) is returned, flagged with `true`.
fn param_structure<'a>(
    param: &Parameter<'a>,
) -> Option<(cda_database::datatypes::StructureDop<'a>, bool)> {
    if !matches!(
        param.param_type(),
        Ok(cda_database::datatypes::ParamType::Value)
    ) {
        return None;
    }
    let dop = param.0.specific_data_as_value().and_then(|vd| vd.dop())?;
    if let Some(s) = dop.specific_data_as_structure() {
        return Some((cda_database::datatypes::StructureDop(s), false));
    }
    let field = dop
        .specific_data_as_end_of_pdu_field()
        .and_then(|f| f.field())
        .or_else(|| dop.specific_data_as_static_field().and_then(|f| f.field()))
        .or_else(|| {
            dop.specific_data_as_dynamic_length_field()
                .and_then(|f| f.field())
        })?;
    field
        .basic_structure()
        .and_then(|b| b.specific_data_as_structure())
        .map(|s| (cda_database::datatypes::StructureDop(s), true))
}

/// Collect parameter infos, recursively expanding structure DOPs so their
/// sub-elements appear as indented rows below the owning parameter.
/// Byte positions of sub-elements are shown absolute (relative to the PDU).
fn collect_param_infos(
    params: &[Parameter<'_>],
    base_byte: u32,
    depth: usize,
    out: &mut Vec<ParamInfo>,
) {
    for param in params {
        let byte_pos = base_byte.saturating_add(param.byte_position());
        let structure = param_structure(param);
        out.push(ParamInfo {
            name: param.short_name().unwrap_or("?").to_owned(),
            byte_pos,
            bit_pos: param.bit_position(),
            bit_len: crate::uds::byte_pattern::extract_bit_length(param),
            value: extract_coded_value(param),
            dop_name: extract_dop_name(param),
            dop_badge: extract_dop_badge(param).to_owned(),
            semantic: param.semantic().unwrap_or_default().to_owned(),
            param_id: param.id(),
            depth,
            struct_byte_size: structure
                .as_ref()
                .filter(|(_, is_field)| !is_field)
                .and_then(|(s, _)| s.byte_size()),
        });

        if depth < MAX_STRUCT_DEPTH
            && let Some(sub) = structure.as_ref().and_then(|(s, _)| s.params())
        {
            let sub: Vec<Parameter<'_>> = sub.iter().map(Parameter).collect();
            collect_param_infos(&sub, byte_pos, depth.saturating_add(1), out);
        }
    }
}

/// Find the next parameter on the same nesting level (stopping at the end of
/// the enclosing structure) for byte length estimation.
fn next_sibling(infos: &[ParamInfo], idx: usize) -> Option<&ParamInfo> {
    let depth = infos.get(idx)?.depth;
    infos
        .iter()
        .skip(idx.saturating_add(1))
        .take_while(|i| i.depth >= depth)
        .find(|i| i.depth == depth)
}

fn param_byte_len_text(info: &ParamInfo, next: Option<&ParamInfo>) -> String {
    if let Some(size) = info.struct_byte_size {
        return size.to_string();
    }
    if let Some(bl) = info.bit_len {
        let effective_start = if info.bit_pos == BIT_POSITION_UNSET {
            0
        } else {
            info.bit_pos
        };
        effective_start
            .saturating_add(bl)
            .saturating_add(7)
            .checked_div(8)
            .unwrap_or(1)
            .to_string()
    } else {
        next.map_or_else(
            || "-".to_owned(),
            |n| n.byte_pos.saturating_sub(info.byte_pos).to_string(),
        )
    }
}

/// Build a parameter table section (the column-based param list used by
/// request / response detail views).  `section_type` distinguishes Requests
/// from `PosResponses` / `NegResponses`.
pub fn build_param_section<'a, I>(
    title: &str,
    params: I,
    section_type: DetailSectionType,
) -> DetailSectionData
where
    I: IntoIterator<Item = Parameter<'a>>,
{
    let params: Vec<Parameter<'_>> = params.into_iter().collect();
    let byte_pattern_rows = if params.is_empty() {
        None
    } else {
        Some(crate::uds::byte_pattern::build_byte_pattern_rows(&params))
    };

    let header = DetailRow::header(vec![
        DetailCell::text("Short Name"),
        DetailCell::new("Byte", CellType::NumericValue),
        DetailCell::new("Bit", CellType::NumericValue),
        DetailCell::new("Bit\nLen", CellType::NumericValue),
        DetailCell::new("Byte\nLen", CellType::NumericValue),
        DetailCell::text("Value"),
        DetailCell::text("DOP"),
        DetailCell::text("Semantic"),
    ]);

    let mut infos: Vec<ParamInfo> = Vec::with_capacity(params.len());
    collect_param_infos(&params, 0, 0, &mut infos);

    let rows: Vec<DetailRow> = infos
        .iter()
        .enumerate()
        .map(|(idx, info)| {
            let has_dop = !info.dop_name.is_empty();

            let dop_jump = if has_dop {
                Some(CellJumpTarget::new(CellJumpTargetType::Dop {
                    index: usize::MAX,
                    name: info.dop_name.clone(),
                }))
            } else {
                None
            };

            let dop_cell_type = if has_dop {
                CellType::DopReference
            } else {
                CellType::Text
            };
            let dop_display = if has_dop {
                format!("{}{}", info.dop_badge, info.dop_name)
            } else {
                info.dop_name.clone()
            };
            let mut dop_cell = DetailCell::new(dop_display, dop_cell_type);
            if let Some(jump) = dop_jump {
                dop_cell = dop_cell.with_jump(jump);
            }

            let bit_len_text = info
                .bit_len
                .map_or_else(|| "-".to_owned(), |bl| bl.to_string());

            let byte_len_text = param_byte_len_text(info, next_sibling(&infos, idx));

            let mut row = DetailRow::normal(
                vec![
                    DetailCell::new(info.name.clone(), CellType::ParameterName).with_jump(
                        CellJumpTarget::new(CellJumpTargetType::Parameter {
                            param_id: info.param_id,
                        }),
                    ),
                    DetailCell::new(info.byte_pos.to_string(), CellType::NumericValue),
                    DetailCell::new(info.bit_pos.to_string(), CellType::NumericValue),
                    DetailCell::new(bit_len_text, CellType::NumericValue),
                    DetailCell::new(byte_len_text, CellType::NumericValue),
                    DetailCell::new(info.value.clone(), CellType::NumericValue),
                    dop_cell,
                    DetailCell::text(info.semantic.clone()),
                ],
                info.depth,
            );
            row.metadata = Some(crate::tree::RowMetadata::ParameterRow {
                param_id: info.param_id,
            });
            row
        })
        .collect();

    DetailSectionData {
        title: title.to_owned(),
        render_as_header: false,
        section_type,
        content: DetailContent::Table {
            header,
            rows,
            constraints: vec![
                ColumnConstraint::Percentage(45),
                ColumnConstraint::Fixed(4),
                ColumnConstraint::Fixed(3),
                ColumnConstraint::Fixed(4),
                ColumnConstraint::Fixed(5),
                ColumnConstraint::Percentage(15),
                ColumnConstraint::Percentage(15),
                ColumnConstraint::Percentage(25),
            ],
            use_row_selection: false,
        },
        byte_pattern_rows,
    }
}

/// Build a service-list table section (the header table showing all services
/// with ID / Short Name / Source columns).  Used by both the Requests and
/// Responses list headers.
///
/// `node_indices` maps each service short name to its tree-node index so
/// that table rows carry direct [`CellJumpTargetType::TreeNodeByIndex`]
/// targets for O(1) navigation.
pub fn build_service_list_table_section(
    own_services: &[DiagService<'_>],
    parent_services: &[(DiagService<'_>, String)],
    label: &str,
    section_type: DetailSectionType,
    node_indices: &std::collections::HashMap<String, usize>,
    overridden: &std::collections::HashMap<String, String>,
) -> DetailSectionData {
    let header = DetailRow::header(vec![
        DetailCell::text("ID"),
        DetailCell::text("Short Name"),
        DetailCell::text("Source"),
    ]);

    let build_row = |ds: &DiagService<'_>, inherited_from: Option<&str>| -> Option<DetailRow> {
        let name = ds.diag_comm()?.short_name().unwrap_or("?").to_owned();
        let source = super::services::service_source_label(&name, inherited_from, overridden);
        let id_str = format_service_id(ds);
        let id = if id_str.is_empty() {
            "-".to_owned()
        } else {
            id_str
        };
        let index = node_indices.get(&name).copied().unwrap_or(usize::MAX);
        let jump = CellJumpTarget::new(CellJumpTargetType::TreeNodeByIndex {
            index,
            short_name: name.clone(),
        });
        Some(DetailRow::normal(
            vec![
                DetailCell::text(id),
                DetailCell::new(name, CellType::ParameterName).with_jump(jump),
                DetailCell::text(source),
            ],
            0,
        ))
    };

    let mut rows = Vec::new();
    rows.extend(own_services.iter().filter_map(|ds| build_row(ds, None)));
    rows.extend(
        parent_services
            .iter()
            .filter_map(|(ds, layer)| build_row(ds, Some(layer.as_str()))),
    );

    let total_count = own_services.len().saturating_add(parent_services.len());

    DetailSectionData {
        title: format!("{label} ({total_count})"),
        render_as_header: false,
        section_type,
        content: DetailContent::Table {
            header,
            rows,
            constraints: vec![
                ColumnConstraint::Percentage(15),
                ColumnConstraint::Percentage(45),
                ColumnConstraint::Percentage(40),
            ],
            use_row_selection: true,
        },
        byte_pattern_rows: None,
    }
}
