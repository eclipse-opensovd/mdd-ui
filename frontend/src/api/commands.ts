// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Copyright (c) Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0

import { invoke } from "@tauri-apps/api/core";

// Auto-generated types from Rust - do not edit manually
import type {
  CellJumpTarget,
  CellJumpTargetType,
  ChatMessage,
  ChatResult,
  DetailCell,
  DetailContent,
  DetailRow,
  DetailSectionData,
  DeviceFlowStart,
  JumpTarget,
  JumpTargetType,
  LoadResult,
  MatchedService,
  NavigateResult,
  PollResult,
  RecentFile,
  RecentFilesResult,
  SearchResult,
  ServiceSchemaResult,
  SettingsUpdate,
  SettingsView,
  TabInfo,
  ToggleSortResult,
  UdsEncodeResult,
  UdsLookupResult,
  UiPrefs,
  VariantInfo,
  VisibleNode,
} from "./generated/index";

export type {
  CellJumpTarget,
  CellJumpTargetType,
  ChatMessage,
  ChatResult,
  DetailCell,
  DetailContent,
  DetailRow,
  DetailSectionData,
  DeviceFlowStart,
  JumpTarget,
  JumpTargetType,
  LoadResult,
  MatchedService,
  NavigateResult,
  PollResult,
  RecentFile,
  RecentFilesResult,
  SearchResult,
  ServiceSchemaResult,
  SettingsUpdate,
  SettingsView,
  TabInfo,
  ToggleSortResult,
  UdsEncodeResult,
  UdsLookupResult,
  UiPrefs,
  VariantInfo,
  VisibleNode,
};

/** @deprecated Use `DetailSectionData` instead. */
export type DetailSection = DetailSectionData;

export async function loadMdd(path: string): Promise<LoadResult> {
  return invoke<LoadResult>("load_mdd", { path });
}

export async function loadDiff(oldPath: string, newPath: string): Promise<LoadResult> {
  return invoke<LoadResult>("load_diff", { oldPath, newPath });
}

export async function getVisibleNodes(): Promise<VisibleNode[]> {
  return invoke<VisibleNode[]>("get_visible_nodes");
}

export async function getNodeDetail(index: number): Promise<DetailSection[]> {
  return invoke<DetailSection[]>("get_node_detail", { index });
}

export async function toggleExpand(index: number): Promise<VisibleNode[]> {
  return invoke<VisibleNode[]>("toggle_expand", { index });
}

export async function doSearch(query: string, op: "and" | "or" = "and"): Promise<SearchResult> {
  return invoke<SearchResult>("search", { query, op });
}

export async function clearSearch(): Promise<VisibleNode[]> {
  return invoke<VisibleNode[]>("clear_search");
}

export async function cycleSearchScope(): Promise<string> {
  return invoke<string>("cycle_search_scope");
}

export async function setSearchScope(scope: string): Promise<string> {
  return invoke<string>("set_search_scope", { scope });
}

export async function toggleSort(nodeIndex?: number): Promise<ToggleSortResult> {
  return invoke<ToggleSortResult>("toggle_sort", { nodeIndex: nodeIndex ?? null });
}

export async function expandAll(): Promise<VisibleNode[]> {
  return invoke<VisibleNode[]>("expand_all");
}

export async function expandFirstLevel(): Promise<VisibleNode[]> {
  return invoke<VisibleNode[]>("expand_first_level");
}

export async function collapseAll(): Promise<VisibleNode[]> {
  return invoke<VisibleNode[]>("collapse_all");
}

export async function toggleHideUnchanged(): Promise<VisibleNode[]> {
  return invoke<VisibleNode[]>("toggle_hide_unchanged");
}

export async function navigateTo(target: JumpTarget): Promise<NavigateResult> {
  return invoke<NavigateResult>("navigate_to", { target });
}

export async function getNodePath(index: number): Promise<string> {
  return invoke<string>("get_node_path", { index });
}

export async function getRecentFiles(): Promise<RecentFilesResult> {
  return invoke<RecentFilesResult>("get_recent_files");
}

export async function addRecentFile(path: string): Promise<void> {
  return invoke("add_recent_file", { path });
}

export async function clearRecentFiles(): Promise<void> {
  return invoke("clear_recent_files");
}

export async function clearAllCaches(): Promise<void> {
  return invoke("clear_all_caches");
}

export async function removeRecentFile(path: string): Promise<void> {
  return invoke("remove_recent_file", { path });
}

export async function getUiPrefs(): Promise<UiPrefs> {
  return invoke<UiPrefs>("get_ui_prefs");
}

export async function saveUiPrefs(prefs: UiPrefs): Promise<void> {
  return invoke("save_ui_prefs", { prefs });
}

export async function registerMddAssociation(): Promise<string> {
  return invoke<string>("register_mdd_association");
}

export async function getInitialFile(): Promise<string | null> {
  return invoke<string | null>("get_initial_file");
}

// Tab management

export async function switchTab(tabId: string): Promise<LoadResult> {
  return invoke<LoadResult>("switch_tab", { tabId });
}

export async function closeTab(tabId: string): Promise<LoadResult | null> {
  return invoke<LoadResult | null>("close_tab", { tabId });
}

export async function getOpenTabs(): Promise<TabInfo[]> {
  return invoke<TabInfo[]>("get_open_tabs");
}

// UDS translation

export async function udsLoad(path: string): Promise<void> {
  return invoke("uds_load", { path });
}

export async function udsListServices(): Promise<MatchedService[]> {
  return invoke<MatchedService[]>("uds_list_services");
}

export async function udsLookup(hex: string): Promise<UdsLookupResult> {
  return invoke<UdsLookupResult>("uds_lookup", { hex });
}

export async function udsEncode(
  serviceName: string,
  json: unknown,
  variantName?: string | null,
): Promise<UdsEncodeResult> {
  return invoke<UdsEncodeResult>("uds_encode", {
    serviceName,
    json,
    variantName: variantName ?? null,
  });
}

export async function udsListVariants(): Promise<VariantInfo[]> {
  return invoke<VariantInfo[]>("uds_list_variants");
}

export async function udsSelectVariant(variantName: string): Promise<VariantInfo> {
  return invoke<VariantInfo>("uds_select_variant", { variantName });
}

export async function getNodeVariant(index: number): Promise<string | null> {
  return invoke<string | null>("get_node_variant", { index });
}
