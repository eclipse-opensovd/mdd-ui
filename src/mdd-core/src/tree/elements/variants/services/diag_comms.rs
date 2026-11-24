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

use cda_database::datatypes::{DiagComm, DiagLayer, DiagService, ParentRef};

use super::details::{
    build_diag_comm_details_with_parent, build_diag_comms_table_section,
    build_precondition_state_refs_from_diag_comm,
};
use crate::tree::{
    builder::TreeBuilder,
    elements::variants::format_service_display_name,
    types::{
        ColumnConstraint, DetailCell, DetailContent, DetailRow, DetailSectionData,
        DetailSectionType, NodeType,
    },
};

pub fn add_diag_comms<'a>(
    b: &mut TreeBuilder,
    layer: &DiagLayer<'a>,
    depth: usize,
    variant_parent_refs: Option<impl Iterator<Item = ParentRef<'a>> + 'a>,
) {
    let mut own_services: Vec<DiagService<'_>> = layer
        .diag_services()
        .map(|services| services.iter().map(DiagService).collect())
        .unwrap_or_default();

    own_services.sort_by_cached_key(|ds| {
        ds.diag_comm()
            .and_then(|dc| dc.short_name())
            .unwrap_or("")
            .to_lowercase()
    });

    let InheritedServices {
        inherited: mut parent_services,
        overridden,
    } = resolve_inherited_services(layer, variant_parent_refs);

    parent_services.sort_by_cached_key(|(ds, _)| {
        ds.diag_comm()
            .and_then(|dc| dc.short_name())
            .unwrap_or("")
            .to_lowercase()
    });

    let job_count = layer.single_ecu_jobs().map_or(0, |jobs| jobs.len());
    let service_count = own_services.len().saturating_add(parent_services.len());
    let total_count = service_count.saturating_add(job_count);

    if total_count > 0 {
        let mut job_names: Vec<String> = layer
            .single_ecu_jobs()
            .map(|jobs| {
                jobs.iter()
                    .map(|job| {
                        job.diag_comm()
                            .and_then(|dc| dc.short_name())
                            .unwrap_or("Unnamed")
                            .to_owned()
                    })
                    .collect()
            })
            .unwrap_or_default();

        job_names.sort_by_key(|name| name.to_lowercase());

        // Push header first (empty details -- patched below with indices).
        let header_idx = b.next_index();
        b.push_service_list_header(
            depth,
            format!("Diag-Comms ({service_count} services, {job_count} jobs)"),
            false,
            true,
            vec![],
            crate::tree::ServiceListType::DiagComms,
        );

        // Push children, collecting (short_name -> tree index).
        let mut node_indices = std::collections::HashMap::new();

        for ds in &own_services {
            let Some(display_name) = format_service_display_name(ds) else {
                continue;
            };
            let short_name = ds
                .diag_comm()
                .and_then(|dc| dc.short_name())
                .unwrap_or("?")
                .to_owned();
            node_indices.insert(short_name.clone(), b.next_index());
            let sections = build_diag_comm_details_with_parent(ds, None, None);

            b.push_service_node(
                depth.saturating_add(1),
                display_name.clone(),
                false,
                sections,
                NodeType::Service,
                short_name,
            );
        }

        for (ds, source_layer_name) in &parent_services {
            let Some(display_name) = format_service_display_name(ds) else {
                continue;
            };
            let short_name = ds
                .diag_comm()
                .and_then(|dc| dc.short_name())
                .unwrap_or("?")
                .to_owned();
            node_indices.insert(short_name.clone(), b.next_index());
            let parent_idx = b.find_container_index(source_layer_name);
            let sections = build_diag_comm_details_with_parent(
                ds,
                Some(source_layer_name.as_str()),
                parent_idx,
            );

            b.push_service_node(
                depth.saturating_add(1),
                display_name.clone(),
                false,
                sections,
                NodeType::ParentRefService,
                short_name,
            );
        }

        add_single_ecu_jobs(b, layer, depth, &mut node_indices);

        // Build table with tree-node indices and patch the header.
        let detail_section = build_diag_comms_table_section(
            &own_services,
            &parent_services,
            &job_names,
            &node_indices,
            &overridden,
        );
        b.set_detail_sections(header_idx, vec![detail_section]);
    }
}

fn add_single_ecu_jobs(
    b: &mut TreeBuilder,
    layer: &DiagLayer<'_>,
    depth: usize,
    node_indices: &mut std::collections::HashMap<String, usize>,
) {
    let Some(jobs) = layer.single_ecu_jobs() else {
        return;
    };

    let mut sorted_jobs: Vec<_> = jobs.iter().collect();
    sorted_jobs.sort_by_cached_key(|job| {
        job.diag_comm()
            .and_then(|dc| dc.short_name())
            .unwrap_or("")
            .to_lowercase()
    });

    for job in sorted_jobs {
        let Some(dc) = job.diag_comm() else {
            continue;
        };

        let Some(short_name) = dc.short_name() else {
            continue;
        };

        let mut sections = Vec::new();

        sections.push(DetailSectionData {
            title: format!("Job - {short_name}"),
            render_as_header: true,
            content: DetailContent::PlainText(vec![]),
            section_type: DetailSectionType::Header,
            byte_pattern_rows: None,
        });

        let header = DetailRow::header(vec![
            DetailCell::text("Property"),
            DetailCell::text("Value"),
        ]);

        let mut rows = Vec::new();

        rows.extend(
            dc.short_name().map(|sn| {
                DetailRow::normal(vec![DetailCell::text("Job"), DetailCell::text(sn)], 0)
            }),
        );
        rows.extend(dc.semantic().map(|semantic| {
            DetailRow::normal(
                vec![DetailCell::text("Semantic"), DetailCell::text(semantic)],
                0,
            )
        }));

        sections.push(
            DetailSectionData::new(
                "Overview".to_owned(),
                DetailContent::Table {
                    header,
                    rows,
                    constraints: vec![
                        ColumnConstraint::Percentage(30),
                        ColumnConstraint::Percentage(70),
                    ],
                    use_row_selection: true,
                },
                false,
            )
            .with_type(DetailSectionType::Overview),
        );

        sections.push(build_precondition_state_refs_from_diag_comm(Some(
            DiagComm(dc),
        )));

        node_indices.insert(short_name.to_owned(), b.next_index());
        b.push_service_node(
            depth.saturating_add(1),
            short_name.to_owned(),
            false,
            sections,
            NodeType::Job,
            short_name.to_owned(),
        );
    }
}

/// Inherited services of a layer after applying ODX override semantics.
pub struct InheritedServices<'a> {
    /// Services effectively inherited (not overridden locally), with source layer name.
    pub inherited: Vec<(DiagService<'a>, String)>,
    /// Short names of local diag-comms that override an inherited one,
    /// mapped to the layer name of the overridden service.
    pub overridden: std::collections::HashMap<String, String>,
}

/// Resolve the services inherited by `layer` from `parent_refs`.
///
/// A local diag-comm (service or job) with the same short name as an inherited
/// service overrides it, so the inherited one is removed from the result and
/// recorded in [`InheritedServices::overridden`].
pub fn resolve_inherited_services<'a>(
    layer: &DiagLayer<'a>,
    parent_refs: Option<impl Iterator<Item = ParentRef<'a>>>,
) -> InheritedServices<'a> {
    let Some(parent_refs) = parent_refs else {
        return InheritedServices {
            inherited: Vec::new(),
            overridden: std::collections::HashMap::new(),
        };
    };

    let local_names: std::collections::HashSet<&str> = layer
        .diag_services()
        .into_iter()
        .flat_map(|s| s.iter().filter_map(|ds| ds.diag_comm()?.short_name()))
        .chain(
            layer
                .single_ecu_jobs()
                .into_iter()
                .flat_map(|j| j.iter().filter_map(|job| job.diag_comm()?.short_name())),
        )
        .collect();

    let mut overridden = std::collections::HashMap::new();
    let inherited = get_parent_ref_services_recursive(parent_refs)
        .into_iter()
        .filter(|(ds, source)| {
            let Some(name) = ds.diag_comm().and_then(|dc| dc.short_name()) else {
                return true;
            };
            if local_names.contains(name) {
                overridden.insert(name.to_owned(), source.clone());
                false
            } else {
                true
            }
        })
        .collect();

    InheritedServices {
        inherited,
        overridden,
    }
}

/// Text for the "Source" column of service list tables.
pub fn service_source_label(
    short_name: &str,
    inherited_from: Option<&str>,
    overridden: &std::collections::HashMap<String, String>,
) -> String {
    match inherited_from {
        Some(layer) => format!("Inherited ({layer})"),
        None => overridden.get(short_name).map_or_else(
            || "Local".to_owned(),
            |layer| format!("Local (overrides {layer})"),
        ),
    }
}

/// Recursively resolve services from parent references, returning each
/// service paired with the name of the layer it was defined in.
///
/// For every parent ref the effective services of the parent layer are
/// collected: the parent's own services plus everything the parent itself
/// inherits (with the parent's local overrides applied). Afterwards the
/// ref's NOT-INHERITED-DIAG-COMMS short names are removed.
/// Deduplicates services by short name, keeping the first occurrence.
fn get_parent_ref_services_recursive<'a>(
    parent_refs: impl Iterator<Item = ParentRef<'a>>,
) -> Vec<(DiagService<'a>, String)> {
    /// Guard against malformed (cyclic) inheritance graphs.
    const MAX_DEPTH: usize = 16;

    fn short_name<'a>(ds: &DiagService<'a>) -> Option<&'a str> {
        ds.diag_comm().and_then(|dc| dc.short_name())
    }

    /// Resolve the parent layer and its own parent refs for one parent ref.
    fn resolve_ref<'a>(
        parent_ref: &ParentRef<'a>,
    ) -> Option<(DiagLayer<'a>, Option<Vec<ParentRef<'a>>>)> {
        use cda_database::datatypes::ParentRefType;
        match parent_ref.ref_type().try_into() {
            Ok(ParentRefType::EcuSharedData) => {
                let esd = parent_ref.ref__as_ecu_shared_data()?;
                Some((DiagLayer(esd.diag_layer()?), None))
            }
            Ok(ParentRefType::FunctionalGroup) => {
                let fg = parent_ref.ref__as_functional_group()?;
                Some((
                    DiagLayer(fg.diag_layer()?),
                    fg.parent_refs().map(|r| r.iter().map(ParentRef).collect()),
                ))
            }
            Ok(ParentRefType::Protocol) => {
                let protocol = parent_ref.ref__as_protocol()?;
                Some((
                    DiagLayer(protocol.diag_layer()?),
                    protocol
                        .parent_refs()
                        .map(|r| r.iter().map(ParentRef).collect()),
                ))
            }
            Ok(ParentRefType::Variant) => {
                let variant = parent_ref.ref__as_variant()?;
                Some((
                    DiagLayer(variant.diag_layer()?),
                    variant
                        .parent_refs()
                        .map(|r| r.iter().map(ParentRef).collect()),
                ))
            }
            Ok(ParentRefType::TableDop | ParentRefType::NONE) | Err(_) => None,
        }
    }

    fn find_services_recursive<'a>(
        parent_refs: impl Iterator<Item = ParentRef<'a>>,
        depth: usize,
    ) -> Vec<(DiagService<'a>, String)> {
        if depth > MAX_DEPTH {
            return Vec::new();
        }
        parent_refs
            .filter_map(|parent_ref| {
                let not_inherited_names: Vec<&str> = parent_ref
                    .not_inherited_diag_comm_short_names()
                    .map(|names| names.iter().collect())
                    .unwrap_or_default();

                let (layer, nested_refs) = resolve_ref(&parent_ref)?;
                let layer_name = layer.short_name().unwrap_or("?").to_owned();

                let own: Vec<(DiagService<'a>, String)> = layer
                    .diag_services()
                    .into_iter()
                    .flat_map(|s| s.iter().map(DiagService))
                    .map(|ds| (ds, layer_name.clone()))
                    .collect();
                let own_names: std::collections::HashSet<&str> =
                    own.iter().filter_map(|(ds, _)| short_name(ds)).collect();

                let inherited = nested_refs
                    .map(|refs| find_services_recursive(refs.into_iter(), depth.saturating_add(1)))
                    .unwrap_or_default()
                    .into_iter()
                    // the parent's own services override what it inherits
                    .filter(|(ds, _)| short_name(ds).is_none_or(|n| !own_names.contains(n)));

                Some(
                    own.into_iter()
                        .chain(inherited)
                        .filter(|(ds, _)| {
                            short_name(ds).is_none_or(|n| !not_inherited_names.contains(&n))
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .flatten()
            .collect()
    }

    let all_services = find_services_recursive(parent_refs, 0);

    // Deduplicate by service short name, keeping the first occurrence
    let mut seen = std::collections::HashSet::new();
    all_services
        .into_iter()
        .filter(|(service, _)| seen.insert(short_name(service).unwrap_or("").to_owned()))
        .collect()
}
