//! Region/element filtering for SOM snapshots.
//!
//! Provides `apply_selector` to narrow a SOM down to specific regions or
//! elements by semantic role or HTML id. Used by both the CLI (`--selector`)
//! and MCP tools (`selector` parameter).

use super::types::{Element, ElementRole, RegionRole, ShadowRoot, Som};

/// Maximum selector size accepted by the shared SOM filter, in UTF-8 bytes.
pub const MAX_SELECTOR_BYTES: usize = 256;

/// Filter a SOM to a specific region or element by semantic selector.
///
/// Supported selectors:
/// - Region roles: `landmark`/`landmarks`/`landmark_region`/`landmark_regions`/`landmark-region`/`landmark region`/`landmark_inventory`/`landmark_inventories`/`landmark-inventory`/`landmark-inventories`/`landmark inventory`/`landmark inventories`/`landmark_region_inventory`/`landmark_region_inventories`/`landmark-region-inventory`/`landmark-region-inventories`/`landmark region inventory`/`landmark region inventories` (all compiled regions), `main`, `nav`/`navs`/`navigation`/`navigations`/`nav_region`/
///   `nav-region`/`nav region`/`nav_regions`/`nav-regions`/`nav regions`/
///   `navigation_region`/`navigation-region`/`navigation region`/
///   `navigation_regions`/`navigation-regions`/`navigation regions`/`search`/`searches`/
///   `search_nav`/`search-nav`/`search_navigation`/`search-navigation`/`search navigation`/
///   `search_navigations`/`search-navigations`/`search navigations`,
///   `aside`/`asides`/`sidebar`/`sidebars`/`complementary`/`complementaries`,
///   `header`/`headers`/`banner`/`banners`,
///   `footer`/`footers`/`contentinfo`/`content_info`/`content-info`, `form`/`forms`, `dialog`/`dialogs`/
///   `modal`/`modals`/`popup`/`popups`/`pop-up`/`pop-ups`,
///   `alertdialog`/`alertdialogs`/`alert-dialog`/`alert-dialogs`/`alert dialog`/
///   `alert dialogs`,
///   `content`/`contents`/`article`/`articles`/`main_content`/`main_contents`/
///   `main-content`/`main-contents`/`main content`/`main contents`/
///   `main-region`/`main region`/`main-regions`/`main regions`/
///   `main_content_region`/`main_content_regions`/`main-content-region`/
///   `main-content-regions`/`main content region`/`main content regions`/
///   `maincontent`/`maincontents`/
///   `primary`/`primary_content`/`primary_contents`/`primary-content`/
///   `primary-contents`/`primary content`/`primary contents`/
///   `primarycontent`/`primarycontents`/
///   `primary_region`/`primary_regions`/`primary-region`/`primary-regions`/
///   `primary region`/`primary regions`/`primary_content_region`/
///   `primary_content_regions`/`primary-content-region`/
///   `primary-content-regions`/`primary content region`/
///   `primary content regions`
///   `article_region`/`article_regions`/`article-region`/`article-regions`/
///   `article region`/`article regions`/
///   `region`/`regions`/`content-region`/`content region`/`content-regions`/
///   `content regions` (or `role=<region>` / `role:<region>`;
///   optional whitespace around the separator is accepted). The same `role=`
///   and `role:` forms also work for element roles.
/// - Element roles: `link` / `links` / `a` / `area` / `areas` / `anchor` /
///   `anchors` / `hyperlink` / `hyperlinks`, `button` / `buttons`, `text_input` /
///   `text_inputs` / `input` / `inputs` / `text_box` / `text_boxes` / `text_field` /
///   `text_fields` / `textboxes` / `textfields` / `textinput` / `textinputs` /
///   `textbox` / `searchbox` /
///   `slider` / `sliders` / `spinbutton` / `spinbuttons` / `spin button` /
///   `spin buttons`, HTML
///   text-input types such as `text`,
///   `search_input` / `search_inputs` / `search-input` / `search_control` /
///   `search_controls` / `search-control` / `search_field` / `search_fields` /
///   `search-field` / `query` / `queries` / `query_input` / `query_inputs` / `query-input` /
///   `query_field` / `query_fields` / `query-field` / `query_control` /
///   `query_controls` / `query-control` / `querybox` / `query-box` /
///   `query_boxes` / `query-boxes` / `search_box` / `search_boxes` / `search-box`,
///   `email`, `number`,
///   `password`, `tel`, `url`, `date`, `datetime` / `datetime-local` /
///   `date_time` / `date-time` / `date time`, `date_time_local` /
///   `date-time-local` / `date time local`, `month`,
///   `time` / `times` / `time_input` / `time_inputs` / `time-input` /
///   `time-inputs` / `time input` / `time inputs` / `timeinput` / `timeinputs` /
///   `timecontrol` / `timecontrols` / `time-control` / `time-controls` /
///   `time control` / `time controls` / `time_picker` / `time_pickers` /
///   `timepicker` / `timepickers` / `time-picker` / `time-pickers` / `time picker` /
///   `time pickers`, `week`, `hidden`, `color`, `range`, and `file`, `textarea` /
///   `textareas` / `text_area` / `text_areas`,
///   `select` / `selects` / `combobox` / `comboboxes` / `combo box` /
///   `combo boxes` / `combo-boxes` /
///   `listbox` / `listboxes` / `list_box` / `list_boxes` / `list box` / `list boxes`,
///   `checkbox` / `checkboxes` / `check box` /
///   `check boxes` / `switch` /
///   `switches` / `menuitemcheckbox` / `menuitemcheckboxes` /
///   `menu-item-checkbox` / `menu-item-checkboxes`, `radio` / `radios` /
///   `radiobutton` / `radiobuttons` / `radio-button` / `radio-buttons` /
///   `radio button` / `radio buttons` /
///   `menuitemradio` / `menuitemradios` / `menu-item-radio` /
///   `menu-item-radios`, `heading`, `image` / `images` / `image_item` /
///   `image-item` / `image item` / `imageitem` / `image_items` / `image-items` /
///   `image items` / `imageitems`, `list` / `lists`,
///   `table` / `tables` / `data_table` / `data_tables` / `data-table` / `data tables`,
///   `paragraph` / `p` / `abbr` / `address` / `cite` / `code` / `dfn` /
///   `figcaption` / `kbd` / `math` / `pre` / `q` / `ruby` / `samp` /
///   `small` / `var` / `s` / `ins` / `del` / `mark` / `subscript` /
///   `superscript` / `deletion` / `insertion` / `suggestion` / `definition` /
///   `term` / `definition-term` / `definition-terms` / `definition_term` /
///   `definition_terms` / `description` / `descriptions` /
///   `definition-description` / `definition-descriptions` /
///   `definition_description` / `definition_descriptions` /
///   `emphasis` / `strong` / `log` / `marquee` / `timer` /
///   `blockquote` / `quote` / `output` /
///   `outputs` / `alert` /
///   `alerts` / `status` / `statuses` / `tooltip` /
///   `tooltips`, `section` / `sections`,
///   plus common HTML tag aliases `a`/`anchor`, `p`, `ul`, `ol`, `dl`, and `menu`,
///   plus the ARIA collection aliases `feed` / `feeds` / `aria-feed` /
///   `aria-feeds` / `aria_feed` / `aria_feeds` / `ariafeed` / `ariafeeds`,
///   `aria feed` / `aria feeds`,
///   plus feed-entry aliases `feed_item` / `feed_items` / `feed-item` /
///   `feed-items` / `feed item` / `feed items` / `feeditem` / `feeditems`,
///   `aria-feed-item` / `aria-feed-items` / `aria_feed_item` /
///   `aria_feed_items` / `aria feed item` / `aria feed items` /
///   `ariafeeditem` / `ariafeeditems`,
///   plus feed-entry synonyms `feed_entry` / `feed_entries` / `feed-entry` /
///   `feed-entries` / `feed entry` / `feed entries` / `feedentry` /
///   `feedentries` and their `aria-*` equivalents,
///   and explicit list aliases `unordered_list`, `ordered_list`, and
///   `definition_list` / `definition_lists` / `definition-list` /
///   `definition-lists` / `definition list` / `definition lists`, plus ARIA
///   `feed` / `feeds`,
///   `group` / `groups` / `fieldset` / `fieldsets` / `figure` / `figures` /
///   `radiogroup` / `radiogroups` / `radio_group` / `radio_groups` /
///   `radio-group` / `radio-groups` / `radio group` / `radio groups` /
///   `progress` / `progressbar` /
///   `progress_bar` / `progress-bar` / `progress bar` / `progressbars` /
///   `meter` / `meters` /
///   `video` / `canvas`,
///   `grid` / `grids` / `data_grid` / `data_grids` / `datagrid` / `datagrids` /
///   `data-grid` / `data-grids` /
///   `treegrid` / `treegrids` / `tree-grid` / `tree-grids` / `table_grid` /
///   `tablegrid` / `tablegrids` / `table-grid` / `table grids` / `table-grids`,
///   `separator` / `separators` / `hr`, `details` / `detail` / `summary` /
///   `summaries`,
///   `iframe` / `iframes`, `audio` / `audios`, `video` / `videos`,
///   `canvas` / `canvases`
///   and the grouped media selector `media` / `media_item` / `media-item` /
///   `media item` / `mediaitem` / `media_items` / `media-items` / `media items` /
///   `mediaitems`
///   (images and media-capable groups).
///   `audio_item`/`audio-item`/`audio item`/`audio_items`/`audio-items`/
///   `audio items` and `video_item`/`video-item`/`video item`/
///   `video_items`/`video-items`/`video items` are convenient
///   media aliases (and match the grouped media behavior). The equivalent
///   `canvas_item`/`canvas-item`/`canvas item`/`canvas_items`/`canvas-items`/
///   `canvas items` forms are also supported.
///   Compound element-role aliases also accept spaces, such as `text input`,
///   `search box`, `menu item`, and `unordered list`.
/// - Heading levels: `h1` .. `h6` (or `heading1` .. `heading6`, including
///   hyphenated, underscored, and spaced forms such as `h-1`, `h_1`, and
///   `h 1`) match headings whose compiled `attrs.level` is that integer.
///   Missing or out-of-range levels are not invented.
/// - Action surfaces: `interactive` / `interactives` / `interactive element` /
///   `interactive-element` / `interactive elements` / `interactive_elements` /
///   `interactive-elements`, bare action names (`click`, `type`,
///   `type_text`/`type-text`/`type text`, `clear`/`clear_text`/`clear-text`/
///   `clear text`, `select`, `select_option`/`select-option`/`select option`,
///   `toggle`, `submit`, `reset`), or `action:<verb>` forms (including
///   `action : <verb>` and `action = <verb>`). The text and
///   select aliases mirror the corresponding MCP interaction tool names.
/// - Label/text search: `label:<text>` / `label=<text>` (also `text:<text>` /
///   `text=<text>`, `name:<text>` /
///   `name=<text>` and
///   `placeholder:<text>` / `placeholder=<text>` and `title:<text>` /
///   `title=<text>` and
///   `testid:<text>` / `testid=<text>` (also `test-id`, `test_id`,
///   `data-testid`, `datatestid`, `data-test-id`, `data_test_id`,
///   `data-test`, `data-qa`, `data-cy`, and `data-automation-id`). CSS presence
///   forms such as `[data-testid]`, `[aria-label]`, `[aria-description]`,
///   `[aria-expanded]`, `[aria-selected]`, `[aria-busy]`, `[aria-disabled]`,
///   `[aria-hidden]`, `[aria-modal]`, `[aria-activedescendant]`, `[aria-details]`,
///   `[aria-flowto]`, `[aria-keyshortcuts]`, `[aria-owns]`,
///   `[aria-roledescription]`, `[aria-braillelabel]`, `[aria-valuemax]`, `[aria-valuemin]`,
///   `[aria-valuenow]`, `[aria-valuetext]`, `[aria-sort]`, `[aria-relevant]`, `[role]`, `[inert]`,
///   `[aria-atomic]`, `[aria-errormessage]`, `[aria-placeholder]`, `[lang]`, `[dir]`, `[title]`,
///   and common HTML boolean attributes such as `[disabled]`, `[checked]`, and `[required]`
///   match. Any `data-*`, `data_*`, `aria-*`, or `aria_*` attribute can also be
///   selected by presence or value, including ARIA attributes introduced after
///   this parser. This covers application-specific test and state hooks. CSS
///   prefix (`^=`), suffix (`$=`), and substring (`*=`)
///   attribute matches are supported alongside exact (`=`) and token (`~=`)
///   matches;
///   compact `accesskey:<value>` / `accesskey=<value>` forms are also
///   supported for keyboard shortcut discovery;
///   `[id]` matches authored HTML ids (not generated SOM element ids),
///   value selectors such as `[role="button"]` match the normalized SOM role
///   elements exposing the corresponding attribute without requiring a
///   particular value, and
///   `aria-label:<text>` / `aria-label=<text>` and
///   `aria-braillelabel:<text>` / `aria-braillelabel=<text>` (also `braille`,
///   `braillelabel`, `braille-label`, and `braille label`) match an element's
///   braille label, case-insensitively, preserving parent context, and
///   `alt:<text>` / `alt=<text>` (also `alttext`, `alternative`, `alt-text`, `alt_text`,
///   `alt text`,
///   `alternative-text`, `alternative_text`, and `alternative text`) and
///   `accessible-name:<text>` / `accessible_name=<text>` and
///   `accessible-label:<text>` / `accessible_label=<text>`) matches a region or
///   element's accessible label, authored text, or `aria-label`/title/placeholder/
///   `alt`/`description`/`aria-description` fallbacks (including nested ARIA descriptions),
///   case-insensitively, preserving parent context.
/// - Id: `#some-id` - region id first, then SOM element `id` or `html_id`
///
/// Unrecognised selectors return the full SOM unchanged (with a warning to stderr).
/// If a recognised selector matches nothing, the full SOM is returned (with a
/// warning) so callers always get usable output.
pub fn apply_selector(som: &Som, selector: &str) -> Som {
    if selector.len() > MAX_SELECTOR_BYTES {
        eprintln!(
            "Warning: selector exceeds the {MAX_SELECTOR_BYTES}-byte limit - returning full SOM"
        );
        return som.clone();
    }
    let selector = selector.trim();
    // Accept the same quoted selector values that the role/action forms use.
    // This is convenient when selectors are passed through a shell or JSON
    // configuration layer that quotes every value uniformly.
    let selector = collapse_selector_whitespace(strip_selector_quotes(selector));
    let selector = selector.as_str();
    // Accept an explicitly named selector value when callers forward a
    // key/value option instead of passing only the selector itself.
    let selector = selector
        .get(..8)
        .filter(|prefix| prefix.eq_ignore_ascii_case("selector"))
        .and_then(|_| selector.get(8..))
        .and_then(|rest| {
            let trimmed = rest.trim_start();
            trimmed
                .strip_prefix(['=', ':'])
                .map(str::trim)
                .map(strip_selector_quotes)
                .or_else(|| {
                    (!rest.is_empty()
                        && rest.chars().next().is_some_and(char::is_whitespace))
                        .then_some(trimmed)
                })
        })
        .filter(|value| !value.is_empty())
        .unwrap_or(selector);
    // Keep the caller's spelling for id selectors. Roles and actions are
    // case-insensitive, but HTML/SOM ids are case-sensitive.
    let selector_for_id = selector;
    let selector_lower = selector.to_ascii_lowercase();
    let selector = selector_lower
        .strip_prefix("role")
        .and_then(|rest| {
            let trimmed = rest.trim_start();
            trimmed
                .strip_prefix(['=', ':'])
                .or_else(|| {
                    (!trimmed.is_empty()
                        && rest.chars().next().is_some_and(char::is_whitespace))
                        .then_some(trimmed)
                })
        })
        .map(str::trim)
        .map(strip_selector_quotes)
        .filter(|value| !value.is_empty())
        .unwrap_or(selector);

    // Return the page's compiled regions as a compact landmark inventory.
    // Unlike a specific region-role selector, this is useful when an agent
    // needs to discover the page structure before choosing a narrower target.
    if matches!(
        selector.to_ascii_lowercase().as_str(),
        "landmark"
            | "landmarks"
            | "role=landmark"
            | "role:landmark"
            | "role landmark"
            | "role=landmarks"
            | "role:landmarks"
            | "role landmarks"
            | "role:landmark_region"
            | "role:landmark-region"
            | "role:landmark region"
            | "role landmark-region"
            | "role=landmark_regions"
            | "role:landmark_regions"
            | "role=landmark-regions"
            | "role:landmark-regions"
            | "role:landmark regions"
            | "role landmark-regions"
            | "role=landmark_inventory"
            | "role:landmark_inventory"
            | "role=landmark-inventory"
            | "role:landmark-inventory"
            | "role=landmark inventory"
            | "role:landmark inventory"
            | "role landmark-inventory"
            | "role=landmark_inventories"
            | "role:landmark_inventories"
            | "role=landmark-inventories"
            | "role:landmark-inventories"
            | "role=landmark inventories"
            | "role:landmark inventories"
            | "role landmark-inventories"
            | "role=landmark_region_inventory"
            | "role:landmark_region_inventory"
            | "role=landmark-region-inventory"
            | "role:landmark-region-inventory"
            | "role=landmark region inventory"
            | "role:landmark region inventory"
            | "role landmark-region-inventory"
            | "role=landmark_region_inventories"
            | "role:landmark_region_inventories"
            | "role=landmark-region-inventories"
            | "role:landmark-region-inventories"
            | "role=landmark region inventories"
            | "role:landmark region inventories"
            | "role landmark-region-inventories"
            | "landmark_region"
            | "landmark_regions"
            | "landmark-region"
            | "landmark region"
            | "landmark regions"
            | "landmark_inventory"
            | "landmark_inventories"
            | "landmark-inventory"
            | "landmark-inventories"
            | "landmark inventory"
            | "landmark inventories"
            | "landmark_region_inventory"
            | "landmark_region_inventories"
            | "landmark-region-inventory"
            | "landmark-region-inventories"
            | "landmark region inventory"
            | "landmark region inventories"
    ) {
        if som.regions.is_empty() {
            eprintln!(
                "Warning: selector '{}' matched no landmarks - returning full SOM",
                selector
            );
            return som.clone();
        }
        let mut result = som.clone();
        for region in &mut result.regions {
            region.elements.clear();
        }
        return refresh_meta(result);
    }

    // Try to match a region role
    let role_opt: Option<RegionRole> = match selector.to_lowercase().as_str() {
        "main"
        | "main_content"
        | "main_contents"
        | "main-content"
        | "main-contents"
        | "main content"
        | "main contents"
        | "main_region"
        | "main-region"
        | "main region"
        | "main_regions"
        | "main-regions"
        | "main regions"
        | "main_content_region"
        | "main_content_regions"
        | "main-content-region"
        | "main-content-regions"
        | "main content region"
        | "main content regions"
        | "maincontent"
        | "maincontents"
        | "primary"
        | "primary_content"
        | "primary_contents"
        | "primary-content"
        | "primary-contents"
        | "primary content"
        | "primary contents"
        | "primarycontent"
        | "primarycontents"
        | "primary_region"
        | "primary_regions"
        | "primary-region"
        | "primary-regions"
        | "primary region"
        | "primary regions"
        | "primary_content_region"
        | "primary_content_regions"
        | "primary-content-region"
        | "primary-content-regions"
        | "primary content region"
        | "primary content regions" => Some(RegionRole::Main),
        "nav"
        | "navs"
        | "navigation"
        | "navigations"
        | "nav_region"
        | "nav-region"
        | "nav region"
        | "nav_regions"
        | "nav-regions"
        | "nav regions"
        | "navigation_region"
        | "navigation-region"
        | "navigation region"
        | "navigation_regions"
        | "navigation-regions"
        | "navigation regions"
        | "search"
        | "searches"
        | "search_nav"
        | "search-nav"
        | "search_navigation"
        | "search-navigation"
        | "search navigation"
        | "search_navigations"
        | "search-navigations"
        | "search navigations" =>
        {
            Some(RegionRole::Navigation)
        }
        "aside"
        | "asides"
        | "sidebar"
        | "sidebars"
        | "complementary"
        | "complementaries" => {
            Some(RegionRole::Aside)
        }
        "header" | "headers" | "banner" | "banners" => Some(RegionRole::Header),
        "footer"
        | "footers"
        | "contentinfo"
        | "contentinfos"
        | "content_info"
        | "content_infos"
        | "content-info"
        | "content-infos"
        | "content info"
        | "content infos" =>
        {
            Some(RegionRole::Footer)
        }
        "form" | "forms" => Some(RegionRole::Form),
        "dialog"
        | "dialogs"
        | "modal"
        | "modals"
        | "popup"
        | "popups"
        | "pop-up"
        | "pop-ups"
        | "alertdialog"
        | "alertdialogs"
        | "alert_dialog"
        | "alert_dialogs"
        | "alert-dialog"
        | "alert-dialogs"
        | "alert dialog"
        | "alert dialogs" => Some(RegionRole::Dialog),
        "content"
        | "contents"
        | "article"
        | "articles"
        | "article_region"
        | "article_regions"
        | "article-region"
        | "article-regions"
        | "article region"
        | "article regions"
        | "region"
        | "regions"
        | "content_region"
        | "content-region"
        | "content region"
        | "content_regions"
        | "content-regions"
        | "content regions" => {
            Some(RegionRole::Content)
        }
        _ => None,
    };

    if let Some(role) = role_opt {
        let filtered: Vec<_> = som
            .regions
            .iter()
            .filter(|r| r.role == role)
            .cloned()
            .collect();
        if filtered.is_empty() {
            // Some semantic names are also element-role aliases (for
            // example, `article`, `content`, and `region` compile to a
            // Section). Let those selectors continue through the element
            // matcher when no region matched.
            if parse_element_role(selector).is_none() {
                eprintln!(
                    "Warning: selector '{}' matched no regions - returning full SOM",
                    selector
                );
                return som.clone();
            }
        }
        if !filtered.is_empty() {
            let mut result = som.clone();
            result.regions = filtered;
            return refresh_meta(result);
        }
    }

    // Match element roles, preserving parent containers and shadow roots that
    // contain matching descendants.
    if matches!(
        selector.to_ascii_lowercase().as_str(),
        "media"
            | "media_item"
            | "media-item"
            | "media item"
            | "mediaitem"
            | "media_items"
            | "media-items"
            | "media items"
            | "mediaitems"
    ) {
        return filter_som_elements(som, selector, is_media_element);
    }

    // Native audio/video/canvas nodes compile to the grouped SOM role, so
    // their source-specific selectors must retain the source-role detail.
    if let Some(source_role) = media_source_role(selector) {
        return filter_som_elements(som, selector, |element| {
            is_source_role_element(element, source_role)
        });
    }

    // Native progress/meter controls and ARIA progressbar roles compile to
    // the generic Group role, but retain their source role in attrs. Prefer
    // that metadata when it is present so a progress selector does not also
    // return unrelated groups. Legacy or hand-built SOMs without the
    // metadata continue through the generic Group role matcher below.
    if let Some(source_role) = source_specific_group_role(selector) {
        if som_contains_source_role(som, source_role) {
            return filter_som_elements(som, selector, |element| {
                is_source_role_element(element, source_role)
            });
        }
    }

    // Match only actionable elements. This is useful when an agent needs a
    // compact menu of possible targets without full body text.
    if matches!(
        selector.to_ascii_lowercase().as_str(),
        "interactive"
            | "interactives"
            | "interactive element"
            | "interactive-element"
            | "interactive elements"
            | "interactive_elements"
            | "interactive-elements"
    ) {
        return filter_som_elements(som, selector, |element| element.role.is_interactive());
    }

    // `time` is both an HTML input type and a native text element. The SOM
    // uses TextInput for the former and Paragraph for the latter, so keep the
    // source-facing selector useful for both compiled forms without treating
    // every text input or paragraph as a time value.
    if selector.eq_ignore_ascii_case("time") || selector.eq_ignore_ascii_case("times") {
        return filter_som_elements(som, selector, |element| {
            match element.role {
                ElementRole::Paragraph => element.attrs.as_ref().is_some_and(|attrs| {
                    attrs
                        .get("source_tag")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|tag| tag.eq_ignore_ascii_case("time"))
                        || attrs
                            .get("source_role")
                            .and_then(serde_json::Value::as_str)
                            .is_some_and(|role| role.eq_ignore_ascii_case("time"))
                }),
                ElementRole::TextInput => element.attrs.as_ref().is_some_and(|attrs| {
                    attrs
                        .get("input_type")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|input_type| input_type.eq_ignore_ascii_case("time"))
                }),
                _ => false,
            }
        });
    }

    // ARIA live-region roles compile to Paragraph, but retain their source
    // role so a live-region selector does not also return ordinary text.
    if let Some(source_role) = live_region_source_role(selector) {
        if som_contains_source_role(som, source_role) {
            return filter_som_elements(som, selector, |element| {
                is_source_role_element(element, source_role)
            });
        }
    }

    // A generic live-region selector is useful when the caller does not care
    // whether the source role is `log`, `status`, or an application-specific
    // ARIA live region. Keep both flattened and nested ARIA metadata forms.
    if matches!(
        selector.to_ascii_lowercase().as_str(),
        "live" | "lives" | "live_region" | "live_regions" | "live-region" | "live-regions"
    ) {
        return filter_som_elements(som, selector, has_aria_live);
    }

    // Explicit time-input selectors are useful when a page contains both a
    // native <time> element and a time form control. Keep the bare `time`
    // selector's backwards-compatible mixed behavior above.
    if matches!(
        selector.to_ascii_lowercase().as_str(),
        "times"
            | "time_input"
            | "time_inputs"
            | "timeinput"
            | "timeinputs"
            | "timecontrol"
            | "timecontrols"
            | "time-control"
            | "time-controls"
            | "time control"
            | "time controls"
            | "time-input"
            | "time-inputs"
            | "time input"
            | "time inputs"
            | "time_picker"
            | "time_pickers"
            | "timepicker"
            | "timepickers"
            | "time-picker"
            | "time-pickers"
            | "time picker"
            | "time pickers"
    ) {
        return filter_som_elements(som, selector, |element| {
            element.role == ElementRole::TextInput
                && element.attrs.as_ref().is_some_and(|attrs| {
                    attrs
                        .get("input_type")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|input_type| input_type.eq_ignore_ascii_case("time"))
                })
        });
    }

    // Native output elements compile as Paragraph so their authored text and
    // nested links stay in the readable text path. Keep the source-facing
    // selector specific to output elements instead of returning every
    // paragraph in the document.
    if selector.eq_ignore_ascii_case("output") || selector.eq_ignore_ascii_case("outputs") {
        return filter_som_elements(som, selector, |element| {
            element.role == ElementRole::Paragraph
                && element.attrs.as_ref().is_some_and(|attrs| {
                    attrs
                        .get("source_tag")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|tag| tag.eq_ignore_ascii_case("output"))
                })
        });
    }

    // ARIA feed is preserved as source metadata because it does not map to
    // the generic SOM list role. Match that source role directly so `feed`
    // does not accidentally select ordinary lists.
    if matches!(
        selector.to_ascii_lowercase().as_str(),
        "feed"
            | "feeds"
            | "aria-feed"
            | "aria-feeds"
            | "aria_feed"
            | "aria_feeds"
            | "aria feed"
            | "aria feeds"
            | "ariafeed"
            | "ariafeeds"
    ) {
        return filter_som_elements(som, selector, |element| {
            has_source_role(element, "feed")
        });
    }

    // Feed entries are exposed as ARIA `article` roles inside a feed. Match
    // that source metadata directly so callers can request entries without
    // also selecting unrelated article landmarks.
    if matches!(
        selector.to_ascii_lowercase().as_str(),
        "feed_item"
            | "feed_items"
            | "feed-item"
            | "feed-items"
            | "feed item"
            | "feed items"
            | "feeditem"
            | "feeditems"
            | "aria-feed-item"
            | "aria-feed-items"
            | "aria_feed_item"
            | "aria_feed_items"
            | "aria feed item"
            | "aria feed items"
            | "ariafeeditem"
            | "ariafeeditems"
            | "feed_entry"
            | "feed_entries"
            | "feed-entry"
            | "feed-entries"
            | "feed entry"
            | "feed entries"
            | "feedentry"
            | "feedentries"
            | "aria-feed-entry"
            | "aria-feed-entries"
            | "aria_feed_entry"
            | "aria_feed_entries"
            | "aria feed entry"
            | "aria feed entries"
            | "ariafeedentry"
            | "ariafeedentries"
    ) {
        return filter_som_elements(som, selector, |element| {
            has_source_role(element, "article")
        });
    }

    // Definition-list terms and descriptions compile to readable paragraph
    // elements while retaining their authored ARIA/source role. Match that
    // metadata directly so callers can target one side of a definition list
    // without selecting every paragraph on the page.
    if matches!(
        selector.to_ascii_lowercase().as_str(),
        "term"
            | "terms"
            | "dt"
            | "definition-term"
            | "definition-terms"
            | "definition term"
            | "definition terms"
            | "definition_term"
            | "definition_terms"
            | "definition"
            | "definitions"
            | "dd"
            | "description"
            | "descriptions"
            | "definition-description"
            | "definition-descriptions"
            | "definition description"
            | "definition descriptions"
            | "definition_description"
            | "definition_descriptions"
    ) {
        let source_role = if matches!(
            selector.to_ascii_lowercase().as_str(),
            "term"
                | "terms"
                | "dt"
                | "definition-term"
                | "definition-terms"
                | "definition term"
                | "definition terms"
                | "definition_term"
                | "definition_terms"
        ) {
            "term"
        } else {
            "definition"
        };
        return filter_som_elements(som, selector, |element| {
            has_source_role(element, source_role)
        });
    }

    if let Some(role) = parse_element_role(selector) {
        return filter_som_elements(som, selector, |element| element.role == role);
    }

    if let Some(level) = parse_heading_level_selector(selector) {
        return filter_som_elements(som, selector, |element| {
            heading_compiled_level(element) == Some(level)
        });
    }

    // Accept the familiar CSS attribute form for the label/test-id selectors
    // as well as the documented `key:value` forms.
    if let Some(attribute) = parse_attribute_presence_selector(selector) {
        return filter_som_elements(som, selector, |element| {
            element_has_attribute_key(element, &attribute)
        });
    }

    if let Some((attribute, value, matcher)) = parse_attribute_selector(selector) {
        return filter_som_elements(som, selector, |element| {
            element_has_attribute_value(element, &attribute, &value, matcher)
        });
    }

    // Keep compact selectors consistent with the CSS-style form for arbitrary
    // data attributes, e.g. `data-state:open` or `data-state=open`.
    if let Some((attribute, value, token_match)) = parse_named_data_attribute_selector(selector) {
        return filter_som_elements(som, selector, |element| {
            element_has_attribute_value(
                element,
                &attribute,
                &value,
                if token_match { AttributeMatcher::Token } else { AttributeMatcher::Exact },
            )
        });
    }

    // Accept compact value selectors for ARIA attributes, e.g.
    // `aria-current:page` or `aria-pressed=true`, alongside the CSS form.
    if let Some((attribute, value, token_match)) = parse_named_aria_attribute_selector(selector) {
        return filter_som_elements(som, selector, |element| {
            element_has_attribute_value(
                element,
                &attribute,
                &value,
                if token_match { AttributeMatcher::Token } else { AttributeMatcher::Exact },
            )
        });
    }

    if let Some(value) = parse_named_accesskey_selector(selector) {
        return filter_som_elements(som, selector, |element| {
            // HTML accesskey is a space-separated list of shortcut tokens.
            // Keep the compact selector token-aware while CSS attribute
            // selectors below retain their exact-match semantics.
            element_has_attribute_value(element, "accesskey", &value, AttributeMatcher::Token)
        });
    }

    // Document metadata attributes have the same compact ergonomics as ARIA
    // attributes, e.g. `lang:en` and `dir:rtl`.
    if let Some((attribute, value)) = parse_named_document_attribute_selector(selector) {
        return filter_som_elements(som, selector, |element| {
            let matcher = if attribute == "lang" {
                AttributeMatcher::DashPrefix
            } else {
                AttributeMatcher::Exact
            };
            element_has_attribute_value(element, &attribute, &value, matcher)
        });
    }

    // Bare document, ARIA, and data attribute names mirror CSS presence
    // selectors without requiring brackets.
    if let Some(attribute) = parse_named_attribute_presence_selector(selector) {
        return filter_som_elements(som, selector, |element| {
            element_has_attribute_key(element, &attribute)
        });
    }

    // Accept the common CSS class shorthand in addition to the explicit
    // attribute form (`[class~="token"]`). Class matching is token-aware so
    // `.card` does not accidentally match `.card-header`.
    if let Some(class_names) = parse_class_selector(selector_for_id) {
        return filter_som_elements(som, selector, |element| {
            class_names.iter().all(|class_name| {
                element_has_attribute_value(element, "class", class_name, AttributeMatcher::Token)
            })
        });
    }

    if let Some(class_names) = parse_named_class_selector(selector_for_id) {
        return filter_som_elements(som, selector, |element| {
            class_names.iter().all(|class_name| {
                element_has_attribute_value(element, "class", class_name, AttributeMatcher::Token)
            })
        });
    }

    if let Some((matcher, value)) = parse_named_class_string_selector(selector_for_id) {
        return filter_som_elements(som, selector, |element| {
            element_has_attribute_value(element, "class", &value, matcher)
        });
    }

    if let Some(test_id) = parse_test_id_selector(selector) {
        return filter_som_elements(som, selector, |element| {
            element_has_test_id(element, &test_id)
        });
    }

    if let Some(alt_text) = parse_alt_selector(selector) {
        return filter_som_elements(som, selector, |element| {
            element
                .attrs
                .as_ref()
                .and_then(|attrs| attrs.get("alt"))
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| normalize_label_text(value).contains(&alt_text))
        });
    }

    if let Some(label) = parse_label_selector(selector) {
        return filter_som_by_label(som, selector, &label);
    }

    // Match elements that expose a specific action in their compact action list.
    let selector_lower = selector.to_ascii_lowercase();
    let action = selector_lower
        .strip_prefix("action")
        .and_then(|rest| {
            let trimmed = rest.trim_start();
            trimmed
                .strip_prefix(['=', ':'])
                .or_else(|| {
                    (!trimmed.is_empty()
                        && rest.chars().next().is_some_and(char::is_whitespace))
                        .then_some(trimmed)
                })
        })
        .map(str::trim)
        .map(strip_selector_quotes)
        .filter(|action| !action.is_empty())
        .or_else(|| {
            matches!(
                selector_lower.as_str(),
                "click"
                    | "type"
                    | "type_text"
                    | "type-text"
                    | "type text"
                    | "clear"
                    | "clear_text"
                    | "clear-text"
                    | "clear text"
                    | "select"
                    | "select_option"
                    | "select-option"
                    | "select option"
                    | "toggle"
                    | "submit"
                    | "reset"
            )
            .then_some(selector_lower.as_str())
        });
    if let Some(action) = action {
        let action = action.trim().to_ascii_lowercase();
        let action = match action.as_str() {
            "type_text" | "type-text" | "type text" => "type".to_string(),
            "clear_text" | "clear-text" | "clear text" => "clear".to_string(),
            "select_option" | "select-option" | "select option" => "select".to_string(),
            _ => action,
        };
        if !action.is_empty() {
            return filter_som_elements(som, selector, |element| {
                let advertises_action = element
                    .actions
                    .as_ref()
                    .map(|actions| actions.iter().any(|a| a.eq_ignore_ascii_case(&action)))
                    .unwrap_or(false);
                let is_submit_control = action == "submit"
                    && element
                        .attrs
                        .as_ref()
                        .and_then(|attrs| attrs.get("button_type"))
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|button_type| {
                            button_type.eq_ignore_ascii_case("submit")
                                || button_type.eq_ignore_ascii_case("image")
                        });
                let is_reset_control = action == "reset"
                    && element
                        .attrs
                        .as_ref()
                        .and_then(|attrs| attrs.get("button_type"))
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|button_type| button_type.eq_ignore_ascii_case("reset"));
                advertises_action || is_submit_control || is_reset_control
            });
        }
    }

    // Try id selectors: #my-id, id=my-id, and id:my-id. Prefer documented
    // region ids, then SOM element ids or HTML ids. If neither matches,
    // return the full SOM.
    if let Some(id) = parse_id_selector(selector_for_id) {
        let region_matches: Vec<_> = som.regions.iter().filter(|r| r.id == id).cloned().collect();
        if !region_matches.is_empty() {
            let mut result = som.clone();
            result.regions = region_matches;
            return refresh_meta(result);
        }

        let filtered_regions: Vec<_> = som
            .regions
            .iter()
            .filter_map(|r| {
                let els = filter_elements_by_id(&r.elements, id);
                if els.is_empty() {
                    None
                } else {
                    let mut region = r.clone();
                    region.elements = els;
                    Some(region)
                }
            })
            .collect();
        if filtered_regions.is_empty() {
            eprintln!("Warning: selector '#{id}' matched no elements - returning full SOM");
            return som.clone();
        }
        let mut result = som.clone();
        result.regions = filtered_regions;
        return refresh_meta(result);
    }

    eprintln!(
        "Warning: unrecognised selector '{}' - returning full SOM",
        selector
    );
    som.clone()
}

fn parse_id_selector(selector: &str) -> Option<&str> {
    if let Some(id) = selector.strip_prefix('#') {
        return Some(strip_selector_quotes(id.trim()));
    }
    let rest = selector
        .get(..2)
        .filter(|prefix| prefix.eq_ignore_ascii_case("id"))
        .and_then(|_| selector.get(2..))?;
    let trimmed = rest.trim_start();
    let id = trimmed
        .strip_prefix(['=', ':'])
        .map(str::trim)
        .or_else(|| {
            (!trimmed.is_empty() && rest.chars().next().is_some_and(char::is_whitespace))
                .then_some(trimmed)
        })?;
    let id = strip_selector_quotes(id);
    (!id.is_empty()).then_some(id)
}

fn parse_class_selector(selector: &str) -> Option<Vec<String>> {
    let class_names = selector
        .strip_prefix('.')?
        .split('.')
        .map(str::trim)
        .map(strip_selector_quotes)
        .collect::<Vec<_>>();
    (!class_names.is_empty()
        && class_names.iter().all(|class_name| {
            !class_name.is_empty() && !class_name.chars().any(char::is_whitespace)
        }))
        .then(|| {
            class_names
                .into_iter()
                .map(normalize_label_text)
                .collect()
        })
}

fn parse_named_class_selector(selector: &str) -> Option<Vec<String>> {
    let rest = selector
        .get(..5)
        .filter(|prefix| prefix.eq_ignore_ascii_case("class"))
        .and_then(|_| selector.get(5..))?;
    let trimmed = rest.trim_start();
    let value = trimmed
        .strip_prefix('~')
        .and_then(|rest| rest.trim_start().strip_prefix('=').map(str::trim))
        .or_else(|| trimmed.strip_prefix(['=', ':']))
        .map(str::trim)
        .or_else(|| {
            (!rest.trim().is_empty() && rest.chars().next().is_some_and(char::is_whitespace))
                .then_some(rest.trim())
        })
        .map(strip_selector_quotes)?;
    let class_names = strip_selector_quotes(value)
        .split('.')
        .map(str::trim)
        .map(strip_selector_quotes)
        .collect::<Vec<_>>();
    (!class_names.is_empty()
        && class_names.iter().all(|class_name| {
            !class_name.is_empty() && !class_name.chars().any(char::is_whitespace)
        }))
        .then(|| class_names.into_iter().map(normalize_label_text).collect())
}

fn parse_named_class_string_selector(selector: &str) -> Option<(AttributeMatcher, String)> {
    let rest = selector
        .get(..5)
        .filter(|prefix| prefix.eq_ignore_ascii_case("class"))
        .and_then(|_| selector.get(5..))?;
    let trimmed = rest.trim_start();
    let (operator, value) = ["^", "$", "*"]
        .into_iter()
        .find_map(|operator| {
            let after_operator = trimmed.strip_prefix(operator)?.trim_start();
            let value = after_operator.strip_prefix('=')?.trim();
            Some((operator, value))
        })?;
    let value = strip_selector_quotes(value);
    if value.is_empty() {
        return None;
    }
    let matcher = match operator {
        "^" => AttributeMatcher::Prefix,
        "$" => AttributeMatcher::Suffix,
        "*" => AttributeMatcher::Substring,
        _ => return None,
    };
    Some((matcher, normalize_label_text(value)))
}

fn strip_selector_quotes(value: &str) -> &str {
    let value = value.trim();
    match value.as_bytes().first().copied() {
        Some(quote @ (b'"' | b'\'' | b'`'))
            if value.as_bytes().last().copied() == Some(quote)
                && !is_escaped_delimiter(value) =>
        {
            value[1..value.len() - 1].trim()
        }
        _ => value,
    }
}

fn is_escaped_delimiter(value: &str) -> bool {
    let mut backslashes = 0;
    for character in value[..value.len() - 1].chars().rev() {
        if character == '\\' {
            backslashes += 1;
        } else {
            break;
        }
    }
    backslashes % 2 == 1
}

fn collapse_selector_whitespace(value: &str) -> String {
    let mut collapsed = String::with_capacity(value.len());
    let mut quote = None;
    let mut escaped = false;
    let mut pending_space = false;

    for character in value.chars() {
        if let Some(active_quote) = quote {
            collapsed.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == active_quote {
                quote = None;
            }
            continue;
        }

        if matches!(character, '\'' | '"' | '`') {
            if pending_space && !collapsed.is_empty() {
                collapsed.push(' ');
            }
            pending_space = false;
            quote = Some(character);
            escaped = false;
            collapsed.push(character);
        } else if character.is_whitespace() {
            pending_space = true;
        } else {
            if pending_space && !collapsed.is_empty() {
                collapsed.push(' ');
            }
            pending_space = false;
            collapsed.push(character);
        }
    }

    collapsed
}

fn is_media_element(element: &Element) -> bool {
    if element.role == ElementRole::Image {
        return true;
    }

    ["audio", "video", "canvas"]
        .into_iter()
        .any(|source_role| is_source_role_element(element, source_role))
}

fn media_source_role(selector: &str) -> Option<&'static str> {
    match selector.to_ascii_lowercase().as_str() {
        "audio"
        | "audios"
        | "audio_item"
        | "audio-item"
        | "audio item"
        | "audioitem"
        | "audio_items"
        | "audio-items"
        | "audio items"
        | "audioitems" => {
            Some("audio")
        }
        "video"
        | "videos"
        | "video_item"
        | "video-item"
        | "video item"
        | "videoitem"
        | "video_items"
        | "video-items"
        | "video items"
        | "videoitems" => {
            Some("video")
        }
        "canvas"
        | "canvases"
        | "canvas_item"
        | "canvas-item"
        | "canvas item"
        | "canvasitem"
        | "canvas_items"
        | "canvas-items"
        | "canvas items"
        | "canvasitems" => Some("canvas"),
        _ => None,
    }
}

fn live_region_source_role(selector: &str) -> Option<&'static str> {
    match selector.to_ascii_lowercase().as_str() {
        "alert" | "alerts" => Some("alert"),
        "log" | "logs" => Some("log"),
        "marquee" | "marquees" => Some("marquee"),
        "status" | "statuses" => Some("status"),
        "timer" | "timers" => Some("timer"),
        _ => None,
    }
}

fn source_specific_group_role(selector: &str) -> Option<&'static str> {
    match selector.to_ascii_lowercase().as_str() {
        "progress"
        | "progressbar"
        | "progress_bar"
        | "progress-bar"
        | "progress bar"
        | "progressbars"
        | "progress_bars"
        | "progress-bars"
        | "progress bars" => Some("progressbar"),
        "meter" | "meters" => Some("meter"),
        _ => None,
    }
}

fn som_contains_source_role(som: &Som, source_role: &str) -> bool {
    som.regions
        .iter()
        .any(|region| elements_contain_source_role(&region.elements, source_role))
}

fn elements_contain_source_role(elements: &[Element], source_role: &str) -> bool {
    elements.iter().any(|element| {
        is_source_role_element(element, source_role)
            || element
                .children
                .as_deref()
                .is_some_and(|children| elements_contain_source_role(children, source_role))
            || element.shadow.as_ref().is_some_and(|shadow| {
                elements_contain_source_role(&shadow.elements, source_role)
            })
    })
}

fn is_source_role_element(element: &Element, source_role: &str) -> bool {
    let compiled_live_region = element.role == ElementRole::Paragraph
        && matches!(
            source_role,
            "alert" | "log" | "marquee" | "status" | "timer"
        );
    (element.role == ElementRole::Group || compiled_live_region)
        && has_source_role(element, source_role)
}

fn has_source_role(element: &Element, source_role: &str) -> bool {
    element
        .attrs
        .as_ref()
        .and_then(|attrs| attrs.get("source_role"))
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value.eq_ignore_ascii_case(source_role))
}

fn has_aria_live(element: &Element) -> bool {
    element.attrs.as_ref().is_some_and(|attrs| {
        attrs.get("aria-live").is_some()
            || attrs.get("aria_live").is_some()
            || attrs
                .get("aria")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|aria| aria.contains_key("live"))
            || attrs
                .get("source_role")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|role| {
                    matches!(
                        role.to_ascii_lowercase().as_str(),
                        "alert" | "log" | "marquee" | "status" | "timer"
                    )
                })
    })
}

fn parse_heading_level_selector(selector: &str) -> Option<u8> {
    let normalized = selector.trim().to_ascii_lowercase();
    let suffix = normalized
        .strip_prefix("heading")
        .or_else(|| normalized.strip_prefix('h'))?;
    let level_suffix = suffix.trim_start_matches(['-', '_', ' ']);
    if level_suffix.len() > 1 && level_suffix.starts_with('0') {
        return None;
    }
    let level = level_suffix
        .parse::<u8>()
        .ok()?;
    (1..=6).contains(&level).then_some(level)
}

fn parse_label_selector(selector: &str) -> Option<String> {
    let selector_lower = selector.to_ascii_lowercase();
    let value = [
        "label",
        "text",
        "name",
        "placeholder",
        "title",
        "testid",
        "test-id",
        "test_id",
        "data-testid",
        "datatestid",
        "data-test-id",
        "data_test_id",
        "data-test",
        "data-qa",
        "data-cy",
        "data-automation-id",
        "aria-label",
        "aria_label",
        "aria label",
        "aria-braillelabel",
        "aria-braille-label",
        "aria_braillelabel",
        "aria_braille_label",
        "aria braillelabel",
        "aria braille label",
        "braillelabel",
        "braille-label",
        "braille label",
        "braille",
        "alt",
        "accessible-name",
        "accessible_name",
        "accessible name",
        "accessible-label",
        "accessible_label",
        "accessible label",
        "accessible-description",
        "accessible_description",
        "accessible description",
        "description",
        "aria-description",
        "aria_description",
        "aria description",
        "roledescription",
        "role-description",
        "role_description",
        "role description",
        "aria-roledescription",
        "aria_roledescription",
        "aria roledescription",
        "live",
        "aria-live",
        "aria_live",
        "aria live",
    ]
        .into_iter()
        .find_map(|prefix| {
            selector_lower
                .strip_prefix(prefix)
                .and_then(|rest| rest.trim_start().strip_prefix(['=', ':']))
        })
        .map(str::trim)
        .map(strip_selector_quotes)
        .filter(|value| !value.is_empty())?;
    Some(normalize_label_text(value))
}

#[derive(Clone, Copy)]
enum AttributeMatcher {
    Exact,
    Token,
    DashPrefix,
    Prefix,
    Suffix,
    Substring,
}

fn parse_attribute_selector(selector: &str) -> Option<(String, String, AttributeMatcher)> {
    let inner = selector.strip_prefix('[')?.strip_suffix(']')?.trim();
    let (index, operator) = find_attribute_operator(inner)?;
    let key = inner[..index].trim();
    let value = inner[index + operator.len()..].trim();
    // CSS permits `i`/`s` modifiers after the value for ASCII-insensitive or
    // sensitive matching, e.g. `[data-state="OPEN" i]`. SOM attribute
    // matching is normalized consistently, so discard either modifier rather
    // than treating it as part of the requested value.
    let value = value
        .strip_suffix(['i', 'I', 's', 'S'])
        .filter(|value| {
            value.chars().last().is_some_and(|character| {
                character.is_ascii_whitespace() || matches!(character, '\'' | '"')
            })
        })
        .map(str::trim_end)
        .unwrap_or(value);
    let operator_kind = operator
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect::<String>();
    let matcher = match operator_kind.as_str() {
        "~=" => AttributeMatcher::Token,
        "|=" => AttributeMatcher::DashPrefix,
        "^=" => AttributeMatcher::Prefix,
        "$=" => AttributeMatcher::Suffix,
        "*=" => AttributeMatcher::Substring,
        "=" => AttributeMatcher::Exact,
        _ => return None,
    };
    let key = key.trim().to_ascii_lowercase();
    let supported = key.starts_with("data-")
        || key.starts_with("data_")
        // Keep CSS attribute selectors forward-compatible with new ARIA
        // attributes instead of requiring a parser change for every addition.
        || key.starts_with("aria-")
        || key.starts_with("aria_")
        || matches!(
            key.as_str(),
        "aria-label"
            | "aria_label"
            | "aria-labelledby"
            | "aria_labelledby"
            | "aria-describedby"
            | "aria_describedby"
            | "aria-description"
            | "aria_description"
            | "aria-controls"
            | "aria_controls"
            | "aria-current"
            | "aria_current"
            | "aria-expanded"
            | "aria_expanded"
            | "aria-busy"
            | "aria_busy"
            | "aria-disabled"
            | "aria_disabled"
            | "aria-hidden"
            | "aria_hidden"
            | "aria-modal"
            | "aria_modal"
            | "aria-haspopup"
            | "aria_haspopup"
            | "aria-checked"
            | "aria_checked"
            | "aria-pressed"
            | "aria_pressed"
            | "aria-selected"
            | "aria_selected"
            | "aria-invalid"
            | "aria_invalid"
            | "aria-required"
            | "aria_required"
            | "aria-activedescendant"
            | "aria_activedescendant"
            | "aria-details"
            | "aria_details"
            | "aria-flowto"
            | "aria_flowto"
            | "aria-keyshortcuts"
            | "aria_keyshortcuts"
            | "aria-owns"
            | "aria_owns"
            | "aria-roledescription"
            | "aria_roledescription"
            | "aria-valuemax"
            | "aria_valuemax"
            | "aria-valuemin"
            | "aria_valuemin"
            | "aria-valuenow"
            | "aria_valuenow"
            | "aria-valuetext"
            | "aria_valuetext"
            | "aria-sort"
            | "aria_sort"
            | "aria-relevant"
            | "aria_relevant"
            | "aria-atomic"
            | "aria_atomic"
            | "aria-errormessage"
            | "aria_errormessage"
            | "aria-placeholder"
            | "aria_placeholder"
            | "aria-live"
            | "aria_live"
            | "role"
            | "id"
            | "inert"
            | "title"
            | "placeholder"
            | "alt"
            | "accesskey"
            | "lang"
            | "dir"
            | "class"
            | "allowfullscreen"
            | "async"
            | "autofocus"
            | "autoplay"
            | "checked"
            | "controls"
            | "default"
            | "defer"
            | "disabled"
            | "formnovalidate"
            | "hidden"
            | "ismap"
            | "itemscope"
            | "loop"
            | "multiple"
            | "muted"
            | "nomodule"
            | "novalidate"
            | "open"
            | "playsinline"
            | "readonly"
            | "required"
            | "reversed"
            | "selected"
            | "testid"
            | "data-testid"
            | "data-test-id"
            | "data_test_id"
            | "data-test"
            | "data-qa"
            | "data-cy"
            | "data-automation-id"
        );
    supported
        .then(|| unescape_selector_value(strip_selector_quotes(value.trim())))
        .filter(|value| !value.is_empty())
        .map(|value| (key, normalize_label_text(&value), matcher))
}

fn find_attribute_operator(inner: &str) -> Option<(usize, &str)> {
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in inner.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if quote.is_some() && character == '\\' {
            escaped = true;
            continue;
        }
        match character {
            '\'' | '"' if quote == Some(character) => quote = None,
            '\'' | '"' if quote.is_none() => quote = Some(character),
            _ if quote.is_some() => continue,
            '~' | '|' | '^' | '$' | '*' => {
                let after_operator = index + character.len_utf8();
                let equals_index = inner[after_operator..]
                    .char_indices()
                    .find_map(|(offset, next)| {
                        (!next.is_ascii_whitespace()).then_some((offset, next))
                    })
                    .and_then(|(offset, next)| {
                        (next == '=').then_some(after_operator + offset)
                    });
                if let Some(equals_index) = equals_index {
                    return Some((index, &inner[index..=equals_index]));
                }
            }
            '=' => return Some((index, "=")),
            _ => {}
        }
    }
    None
}

fn unescape_selector_value(value: &str) -> String {
    let mut unescaped = String::with_capacity(value.len());
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            unescaped.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else {
            unescaped.push(character);
        }
    }
    if escaped {
        unescaped.push('\\');
    }
    unescaped
}

fn parse_attribute_presence_selector(selector: &str) -> Option<String> {
    let key = selector.strip_prefix('[')?.strip_suffix(']')?.trim();
    if key.contains('=') || key.contains('~') {
        return None;
    }
    let key = key.to_ascii_lowercase();
    (key.starts_with("data-")
        || key.starts_with("data_")
        // Keep presence selectors forward-compatible with new ARIA
        // attributes instead of requiring a parser change for every addition.
        || key.starts_with("aria-")
        || key.starts_with("aria_")
        || matches!(
            key.as_str(),
        "aria-label"
            | "aria_label"
            | "aria-labelledby"
            | "aria_labelledby"
            | "aria-describedby"
            | "aria_describedby"
            | "aria-description"
            | "aria_description"
            | "aria-controls"
            | "aria_controls"
            | "aria-current"
            | "aria_current"
            | "aria-expanded"
            | "aria_expanded"
            | "aria-busy"
            | "aria_busy"
            | "aria-disabled"
            | "aria_disabled"
            | "aria-hidden"
            | "aria_hidden"
            | "aria-modal"
            | "aria_modal"
            | "aria-haspopup"
            | "aria_haspopup"
            | "aria-checked"
            | "aria_checked"
            | "aria-pressed"
            | "aria_pressed"
            | "aria-selected"
            | "aria_selected"
            | "aria-invalid"
            | "aria_invalid"
            | "aria-required"
            | "aria_required"
            | "aria-activedescendant"
            | "aria_activedescendant"
            | "aria-details"
            | "aria_details"
            | "aria-flowto"
            | "aria_flowto"
            | "aria-keyshortcuts"
            | "aria_keyshortcuts"
            | "aria-owns"
            | "aria_owns"
            | "aria-roledescription"
            | "aria_roledescription"
            | "aria-valuemax"
            | "aria_valuemax"
            | "aria-valuemin"
            | "aria_valuemin"
            | "aria-valuenow"
            | "aria_valuenow"
            | "aria-valuetext"
            | "aria_valuetext"
            | "aria-sort"
            | "aria_sort"
            | "aria-relevant"
            | "aria_relevant"
            | "aria-atomic"
            | "aria_atomic"
            | "aria-errormessage"
            | "aria_errormessage"
            | "aria-placeholder"
            | "aria_placeholder"
            | "aria-live"
            | "aria_live"
            | "role"
            | "id"
            | "inert"
            | "title"
            | "placeholder"
            | "alt"
            | "accesskey"
            | "lang"
            | "dir"
            | "class"
            | "allowfullscreen"
            | "async"
            | "autofocus"
            | "autoplay"
            | "checked"
            | "controls"
            | "default"
            | "defer"
            | "disabled"
            | "formnovalidate"
            | "hidden"
            | "ismap"
            | "itemscope"
            | "loop"
            | "multiple"
            | "muted"
            | "nomodule"
            | "novalidate"
            | "open"
            | "playsinline"
            | "readonly"
            | "required"
            | "reversed"
            | "selected"
            | "testid"
            | "test-id"
            | "test_id"
            | "data-testid"
            | "datatestid"
            | "data-test-id"
            | "data_test_id"
            | "data-test"
            | "data-qa"
            | "data-cy"
            | "data-automation-id"
        ))
    .then_some(key)
}

fn parse_named_data_attribute_selector(selector: &str) -> Option<(String, String, bool)> {
    let (key, value, token_match) = if let Some((key, value)) = selector.split_once('~') {
        (key, value.strip_prefix('=')?, true)
    } else if let Some((key, value)) = selector.split_once(':') {
        (key, value, false)
    } else {
        let (key, value) = selector.split_once('=')?;
        (key, value, false)
    };
    let key = key.trim().to_ascii_lowercase();
    let value = unescape_selector_value(strip_selector_quotes(value.trim()));
    (key.starts_with("data-") || key.starts_with("data_"))
        .then_some((key, normalize_label_text(&value), token_match))
        .filter(|(_, value, _)| !value.is_empty())
}

fn parse_named_aria_attribute_selector(selector: &str) -> Option<(String, String, bool)> {
    let (key, value, token_match) = if let Some(index) = selector.find('~') {
        let (key, rest) = selector.split_at(index);
        let value = rest.strip_prefix('~')?.trim_start().strip_prefix('=')?;
        (key, value, true)
    } else if let Some((key, value)) = selector.split_once(':') {
        (key, value, false)
    } else {
        let (key, value) = selector.split_once('=')?;
        (key, value, false)
    };
    let key = key.trim().to_ascii_lowercase();
    let value = unescape_selector_value(strip_selector_quotes(value.trim()));
    (key.starts_with("aria-") || key.starts_with("aria_"))
        .then_some((key, normalize_label_text(&value), token_match))
        .filter(|(_, value, _)| !value.is_empty())
}

fn parse_named_accesskey_selector(selector: &str) -> Option<String> {
    let (key, value) = selector
        .split_once(':')
        .or_else(|| selector.split_once('='))?;
    if !key.trim().eq_ignore_ascii_case("accesskey") {
        return None;
    }
    let value = unescape_selector_value(strip_selector_quotes(value.trim()));
    (!value.is_empty()).then(|| normalize_label_text(&value))
}

fn parse_named_document_attribute_selector(selector: &str) -> Option<(String, String)> {
    let (key, value) = if let Some((key, value)) = selector.split_once("|=") {
        (key, value)
    } else {
        selector
            .split_once(':')
            .or_else(|| selector.split_once('='))
            .or_else(|| selector.split_once(char::is_whitespace))?
    };
    let key = key.trim().to_ascii_lowercase();
    if !matches!(key.as_str(), "lang" | "dir") {
        return None;
    }
    let value = unescape_selector_value(strip_selector_quotes(value.trim()));
    let value = normalize_label_text(&value);
    (!value.is_empty()).then_some((key, value))
}

fn parse_named_attribute_presence_selector(selector: &str) -> Option<String> {
    let key = selector.trim().to_ascii_lowercase();
    (matches!(
        key.as_str(),
        "lang"
            | "dir"
            | "accesskey"
            | "title"
            | "inert"
            | "allowfullscreen"
            | "async"
            | "autofocus"
            | "autoplay"
            | "checked"
            | "controls"
            | "defer"
            | "disabled"
            | "formnovalidate"
            | "hidden"
            | "itemscope"
            | "ismap"
            | "loop"
            | "multiple"
            | "muted"
            | "nomodule"
            | "novalidate"
            | "open"
            | "playsinline"
            | "readonly"
            | "required"
            | "reversed"
            | "selected"
    )
        || key.starts_with("aria-")
        || key.starts_with("aria_")
        || key.starts_with("data-")
        || key.starts_with("data_"))
        .then_some(key)
}

fn element_has_attribute_key(element: &Element, key: &str) -> bool {
    let normalize = |value: &str| {
        value
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect::<String>()
    };
    let normalized_key = normalize(key);
    if normalized_key == "role" {
        return true;
    }
    if normalized_key == "id" {
        return element.html_id.is_some();
    }
    let is_test_hook = |key: &str| {
        matches!(
            key,
            "testid"
                | "datatestid"
                | "datatest"
                | "dataqa"
                | "datacy"
                | "dataautomationid"
        )
    };

    element
        .attrs
        .as_ref()
        .and_then(serde_json::Value::as_object)
        .is_some_and(|attrs| {
            attrs.keys().any(|attribute| {
                let normalized_attribute = normalize(attribute);
                normalized_attribute == normalized_key
                    || (is_test_hook(&normalized_attribute) && is_test_hook(&normalized_key))
            }) || attrs
                .get("aria")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|aria| {
                    let nested_key = normalized_key
                        .strip_prefix("aria")
                        .unwrap_or(&normalized_key);
                    aria.keys().any(|attribute| normalize(attribute) == nested_key)
                })
        })
}

fn element_has_attribute_value(
    element: &Element,
    key: &str,
    expected: &str,
    matcher: AttributeMatcher,
) -> bool {
    let normalize_key = |value: &str| {
        value
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect::<String>()
    };
    let normalize_value = |value: &serde_json::Value| match value {
        serde_json::Value::String(value) => normalize_label_text(value),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        _ => String::new(),
    };
    let normalized_key = normalize_key(key);
    let case_insensitive_value = matches!(normalized_key.as_str(), "lang" | "dir");
    let matches_key = |candidate: &str| normalize_key(candidate) == normalized_key;
    let matches_value = |value: &serde_json::Value| {
        let mut value = normalize_value(value);
        let mut expected = expected.to_string();
        if case_insensitive_value {
            value.make_ascii_lowercase();
            expected.make_ascii_lowercase();
        }
        match matcher {
            AttributeMatcher::Token => value.split_whitespace().any(|token| token == expected),
            AttributeMatcher::DashPrefix => {
                value == expected || value.starts_with(&format!("{expected}-"))
            }
            AttributeMatcher::Prefix => value.starts_with(&expected),
            AttributeMatcher::Suffix => value.ends_with(&expected),
            AttributeMatcher::Substring => value.contains(&expected),
            AttributeMatcher::Exact => value == expected,
        }
    };

    if normalized_key == "role" {
        let role = normalize_label_text(element.role.as_str());
        return match matcher {
            AttributeMatcher::Token => role.split_whitespace().any(|token| token == expected),
            AttributeMatcher::DashPrefix => {
                role == expected || role.starts_with(&format!("{expected}-"))
            }
            AttributeMatcher::Prefix => role.starts_with(expected),
            AttributeMatcher::Suffix => role.ends_with(expected),
            AttributeMatcher::Substring => role.contains(expected),
            AttributeMatcher::Exact => role == expected,
        };
    }
    if normalized_key == "id" {
        return element
            .html_id
            .as_deref()
            .is_some_and(|value| {
                let value = normalize_label_text(value);
                match matcher {
                    AttributeMatcher::Token => value.split_whitespace().any(|token| token == expected),
                    AttributeMatcher::DashPrefix => {
                        value == expected || value.starts_with(&format!("{expected}-"))
                    }
                    AttributeMatcher::Prefix => value.starts_with(expected),
                    AttributeMatcher::Suffix => value.ends_with(expected),
                    AttributeMatcher::Substring => value.contains(expected),
                    AttributeMatcher::Exact => value == expected,
                }
            });
    }

    element
        .attrs
        .as_ref()
        .and_then(serde_json::Value::as_object)
        .is_some_and(|attrs| {
            attrs.iter().any(|(attribute, value)| {
                matches_key(attribute) && matches_value(value)
            }) || attrs
                .get("aria")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|aria| {
                    let nested_key = normalized_key.strip_prefix("aria").unwrap_or(&normalized_key);
                    aria.iter().any(|(attribute, value)| {
                        normalize_key(attribute) == nested_key && matches_value(value)
                    })
                })
        })
}

fn parse_test_id_selector(selector: &str) -> Option<String> {
    let selector_lower = selector.to_ascii_lowercase();
    [
        "testid",
        "test-id",
        "test_id",
        "data-testid",
        "datatestid",
        "data-test-id",
        "data_test_id",
        "data-test",
        "data-qa",
        "data-cy",
        "data-automation-id",
    ]
        .into_iter()
        .find_map(|prefix| {
            selector_lower
                .strip_prefix(prefix)
                .and_then(|rest| rest.trim_start().strip_prefix(['=', ':']))
        })
        .map(str::trim)
        .map(strip_selector_quotes)
        .filter(|value| !value.is_empty())
        .map(normalize_label_text)
}

fn element_has_test_id(element: &Element, test_id: &str) -> bool {
    let normalized_key = |key: &str| {
        key.chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect::<String>()
    };

    element
        .attrs
        .as_ref()
        .and_then(serde_json::Value::as_object)
        .is_some_and(|attrs| {
            attrs.iter().any(|(key, value)| {
                matches!(
                    normalized_key(key).as_str(),
                    "testid"
                        | "datatestid"
                        | "datatest"
                        | "dataqa"
                        | "datacy"
                        | "dataautomationid"
                ) && value
                    .as_str()
                    .is_some_and(|value| normalize_label_text(value).contains(test_id))
            })
        })
}

fn parse_alt_selector(selector: &str) -> Option<String> {
    let selector_lower = selector.to_ascii_lowercase();
    [
        "alt",
        "alttext",
        "alternative",
        "alt-text",
        "alt_text",
        "alt text",
        "alternative-text",
        "alternative_text",
        "alternative text",
    ]
        .into_iter()
        .find_map(|prefix| {
            selector_lower
                .strip_prefix(prefix)
                .and_then(|rest| rest.trim_start().strip_prefix(['=', ':']))
        })
    .map(str::trim)
    .map(strip_selector_quotes)
    .filter(|value| !value.is_empty())
    .map(normalize_label_text)
}

fn element_matches_label(element: &Element, label: &str) -> bool {
    let matches = |value: &str| normalize_label_text(value).contains(label);
    let normalized_key = |key: &str| {
        key.chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect::<String>()
    };

    if element
        .label
        .as_deref()
        .is_some_and(matches)
        || element.text.as_deref().is_some_and(matches)
    {
        return true;
    }

    element.attrs.as_ref().and_then(serde_json::Value::as_object).is_some_and(|attrs| {
        attrs.iter().any(|(key, value)| {
            matches!(
                normalized_key(key).as_str(),
                "arialabel"
                    | "labelledby"
                    | "arialabelledby"
                    | "title"
                    | "placeholder"
                    | "alt"
                    | "testid"
                    | "datatestid"
                    | "datatest"
                    | "dataqa"
                    | "datacy"
                    | "dataautomationid"
                    | "description"
                    | "describedby"
                    | "ariadescription"
                    | "ariadescribedby"
                    | "ariabraillelabel"
            | "ariacontrols"
            | "ariacurrent"
            | "ariaexpanded"
            | "ariabusy"
            | "ariadisabled"
            | "ariahidden"
            | "ariahaspopup"
            | "ariapressed"
            | "ariaselected"
            | "ariainvalid"
            | "ariarequired"
            | "ariaactivedescendant"
            | "ariadetails"
            | "ariaflowto"
            | "ariakeyshortcuts"
            | "ariaowns"
                    | "ariaroledescription"
                    | "arialive"
            | "ariavaluemax"
            | "ariavaluemin"
            | "ariavaluenow"
            | "ariavaluetext"
            ) && value.as_str().is_some_and(matches)
        }) || attrs
            .get("aria")
            .and_then(serde_json::Value::as_object)
            .is_some_and(|aria| {
                aria.iter().any(|(key, value)| {
                    matches!(
                        normalized_key(key).as_str(),
                        "label"
                            | "description"
                            | "arialabel"
                            | "labelledby"
                            | "arialabelledby"
                            | "describedby"
                            | "ariadescription"
                            | "ariadescribedby"
                            | "ariabraillelabel"
                        | "ariacontrols"
                        | "ariacurrent"
                        | "ariaexpanded"
                        | "ariabusy"
                        | "ariadisabled"
                        | "ariahidden"
                        | "ariahaspopup"
                        | "ariapressed"
                        | "ariaselected"
                        | "ariainvalid"
                        | "ariarequired"
                        | "active_descendant"
                        | "activedescendant"
                        | "details"
                        | "flowto"
                        | "keyshortcuts"
                        | "owns"
                        | "roledescription"
                        | "live"
                        | "valuemax"
                        | "valuemin"
                        | "valuenow"
                        | "valuetext"
                    )
                        && value.as_str().is_some_and(matches)
                })
            })
    })
}

fn filter_som_by_label(som: &Som, selector: &str, label: &str) -> Som {
    let filtered_regions: Vec<_> = som
        .regions
        .iter()
        .filter_map(|region| {
            if region
                .label
                .as_deref()
                .is_some_and(|value| normalize_label_text(value).contains(label))
            {
                return Some(region.clone());
            }

            let elements = filter_elements_by(&region.elements, &|element| {
                element_matches_label(element, label)
            });
            if elements.is_empty() {
                None
            } else {
                let mut region = region.clone();
                region.elements = elements;
                Some(region)
            }
        })
        .collect();

    if filtered_regions.is_empty() {
        eprintln!(
            "Warning: selector '{}' matched no labels - returning full SOM",
            selector
        );
        return som.clone();
    }

    let mut result = som.clone();
    result.regions = filtered_regions;
    refresh_meta(result)
}

fn normalize_label_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

fn heading_compiled_level(element: &Element) -> Option<u8> {
    if element.role != ElementRole::Heading {
        return None;
    }
    element
        .attrs
        .as_ref()
        .and_then(|attrs| attrs.get("level"))
        .and_then(|value| value.as_u64())
        .and_then(|level| u8::try_from(level).ok())
        .filter(|level| (1..=6).contains(level))
}

fn parse_element_role(selector: &str) -> Option<ElementRole> {
    match selector
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .replace(['-', ' '], "_")
        .as_str()
    {
        "link"
        | "links"
        | "a"
        | "area"
        | "areas"
        | "anchor"
        | "anchors"
        | "hyperlink"
        | "hyperlinks" => {
            Some(ElementRole::Link)
        }
        // These ARIA widget roles compile to the SOM's button role, so expose
        // the same selector aliases instead of making callers know the
        // normalized representation.
        "button" | "buttons" | "menuitem" | "menuitems" | "menu_item" | "menu_items"
        | "option" | "options" | "tab" | "tabs" | "treeitem" | "treeitems"
        | "tree_item" | "tree_items" => {
            Some(ElementRole::Button)
        }
        // HTML input types compile to the same SOM role. Accept their names
        // here so callers can target the controls using the source markup's
        // vocabulary without knowing the normalized role.
        "text_input"
        | "text_inputs"
        | "textbox"
        | "textboxes"
        | "text_box"
        | "text_boxes"
        | "text_field"
        | "text_fields"
        | "textinput"
        | "textinputs"
        | "textfields"
        | "searchbox"
        | "searchboxes"
        | "slider"
        | "sliders"
        | "spinbutton"
        | "spinbuttons"
        | "spin_button"
        | "spin_buttons"
        | "input"
        | "text"
        | "search"
        | "search_input"
        | "search_inputs"
        | "search_control"
        | "search_controls"
        | "search_field"
        | "search_fields"
        | "search_box"
        | "search_boxes"
        | "query_input"
        | "query_inputs"
        | "queries"
        | "query"
        | "query_field"
        | "query_fields"
        | "query_control"
        | "query_controls"
        | "querybox"
        | "query_box"
        | "queryboxes"
        | "query_boxes"
        | "inputs"
        | "email"
        | "number"
        | "password"
        | "tel"
        | "url"
        | "date"
        | "datetime"
        | "datetime_local"
        | "date_time"
        | "date_time_local"
        | "month"
        | "time"
        | "times"
        | "time_input"
        | "time_inputs"
        | "timeinput"
        | "timeinputs"
        | "timecontrol"
        | "timecontrols"
        | "time-control"
        | "time-controls"
        | "time control"
        | "time controls"
        | "time_picker"
        | "time_pickers"
        | "timepicker"
        | "timepickers"
        | "week"
        | "color"
        | "range"
        | "hidden"
        | "file" => Some(ElementRole::TextInput),
        "textarea" | "textareas" | "text_area" | "text_areas" => Some(ElementRole::Textarea),
        "select"
        | "selects"
        | "combobox"
        | "comboboxes"
        | "combo_box"
        | "combo_boxes"
        | "listbox"
        | "listboxes"
        | "list_box"
        | "list_boxes" => Some(ElementRole::Select),
        "checkbox"
        | "checkboxes"
        | "check_box"
        | "check_boxes"
        | "switch"
        | "switches"
        | "menuitemcheckbox"
        | "menuitemcheckboxes"
        | "menuitem_checkbox"
        | "menuitem_checkboxes"
        | "menu-item-checkbox"
        | "menu-item-checkboxes"
        | "menu_item_checkbox"
        | "menu_item_checkboxes" => {
            Some(ElementRole::Checkbox)
        }
        "radio"
        | "radios"
        | "radiobutton"
        | "radiobuttons"
        | "radio-button"
        | "radio-buttons"
        | "radio_button"
        | "radio_buttons"
        | "menuitemradio"
        | "menuitemradios"
        | "menuitem_radio"
        | "menuitem_radios"
        | "menu-item-radio"
        | "menu-item-radios"
        | "menu_item_radio"
        | "menu_item_radios" => Some(ElementRole::Radio),
        "heading" | "headings" => Some(ElementRole::Heading),
        "image"
        | "images"
        | "img"
        | "picture"
        | "pictures"
        | "image_item"
        | "image-item"
        | "image item"
        | "imageitem"
        | "image_items"
        | "image-items"
        | "image items"
        | "imageitems" => Some(ElementRole::Image),
        "list"
        | "lists"
        | "ul"
        | "ol"
        | "dl"
        | "menu"
        | "feed"
        | "feeds"
        | "unordered_list"
        | "ordered_list"
        | "definition_list"
        | "definition_lists"
        | "definition-list"
        | "definition-lists"
        | "definition list"
        | "definition lists" => Some(ElementRole::List),
        "table"
        | "tables"
        | "data_table"
        | "data_tables"
        | "data-table"
        | "data-tables"
        | "data table"
        | "data tables"
        | "grid"
        | "grids"
        | "data_grid"
        | "data_grids"
        | "datagrid"
        | "datagrids"
        | "data-grid"
        | "data-grids"
        | "data grid"
        | "data grids"
        | "table_grid"
        | "table_grids"
        | "tablegrid"
        | "tablegrids"
        | "table-grid"
        | "table-grids"
        | "table grid"
        | "table grids"
        | "tree_grid"
        | "tree_grids"
        | "tree-grid"
        | "tree-grids"
        | "tree grid"
        | "tree grids"
        | "treegrid"
        | "treegrids" => Some(ElementRole::Table),
        "paragraph" | "paragraphs" | "p" | "abbr" | "address" | "cite" | "code" | "dfn"
        | "figcaption" | "kbd" | "math" | "pre" | "q" | "ruby" | "samp" | "small" | "var"
        | "s" | "ins" | "del" | "mark" | "subscript" | "superscript" | "deletion"
        | "insertion" | "suggestion" | "definition" | "definitions" | "term" | "terms"
        | "emphasis" | "emphases" | "strong" | "strongs" | "blockquote" | "blockquotes" | "quote" | "quotes"
        | "output" | "outputs"
        | "alert" | "alerts" | "status" | "statuses" | "tooltip"
        | "tooltips" | "log" | "logs" | "marquee" | "marquees" | "timer"
        | "timers" => {
            Some(ElementRole::Paragraph)
        }
        "article" | "articles" | "content" | "contents" | "region" | "regions"
        | "content_region" | "content_regions" | "section" | "sections" => {
            Some(ElementRole::Section)
        }
        "group"
        | "groups"
        | "div"
        | "fieldset"
        | "fieldsets"
        | "figure"
        | "figures"
        | "radiogroup"
        | "radiogroups"
        | "radio_group"
        | "radio_groups"
        | "radio-group"
        | "radio-groups"
        | "radio group"
        | "radio groups"
        | "progress"
        | "progressbar"
        | "progress_bar"
        | "progress-bar"
        | "progress bar"
        | "progressbars"
        | "progress_bars"
        | "progress-bars"
        | "progress bars"
        | "meter"
        | "meters"
        | "audio"
        | "audios"
        | "audio_item"
        | "audio-item"
        | "audio_items"
        | "audio-items"
        | "video"
        | "videos"
        | "video_item"
        | "video-item"
        | "video_items"
        | "video-items"
        | "canvas"
        | "canvases" => {
            Some(ElementRole::Group)
        }
        "separator" | "separators" | "hr" => Some(ElementRole::Separator),
        "detail" | "details" | "summary" | "summaries" => Some(ElementRole::Details),
        "iframe" | "iframes" => Some(ElementRole::Iframe),
        _ => None,
    }
}

fn filter_som_elements<F>(som: &Som, selector: &str, matches: F) -> Som
where
    F: Fn(&Element) -> bool,
{
    let filtered_regions: Vec<_> = som
        .regions
        .iter()
        .filter_map(|region| {
            let elements = filter_elements_by(&region.elements, &matches);
            if elements.is_empty() {
                None
            } else {
                let mut region = region.clone();
                region.elements = elements;
                Some(region)
            }
        })
        .collect();

    if filtered_regions.is_empty() {
        eprintln!(
            "Warning: selector '{}' matched no elements - returning full SOM",
            selector
        );
        return som.clone();
    }

    let mut result = som.clone();
    result.regions = filtered_regions;
    refresh_meta(result)
}

pub fn refresh_meta(mut som: Som) -> Som {
    let mut element_count = 0;
    let mut interactive_count = 0;
    for region in &som.regions {
        count_elements(&region.elements, &mut element_count, &mut interactive_count);
    }
    som.meta.element_count = element_count;
    som.meta.interactive_count = interactive_count;

    // `som_bytes` describes the serialized snapshot, so it must be refreshed
    // after narrowing the regions. Iterate until stable because the metadata
    // value itself is part of the serialized payload and its digit count can
    // affect length.
    for _ in 0..8 {
        let serialized_len = serde_json::to_string(&som)
            .map(|json| json.len())
            .unwrap_or(0);
        if som.meta.som_bytes == serialized_len {
            break;
        }
        som.meta.som_bytes = serialized_len;
    }
    som
}

fn count_elements(elements: &[Element], element_count: &mut usize, interactive_count: &mut usize) {
    for element in elements {
        *element_count += 1;
        if element.role.is_interactive() {
            *interactive_count += 1;
        }
        if let Some(children) = &element.children {
            count_elements(children, element_count, interactive_count);
        }
        if let Some(shadow) = &element.shadow {
            count_elements(&shadow.elements, element_count, interactive_count);
        }
    }
}

fn filter_elements_by<F>(elements: &[Element], matches: &F) -> Vec<Element>
where
    F: Fn(&Element) -> bool,
{
    elements
        .iter()
        .filter_map(|element| {
            let mut cloned = element.clone();
            if let Some(children) = &element.children {
                let filtered_children = filter_elements_by(children, matches);
                cloned.children = if filtered_children.is_empty() {
                    None
                } else {
                    Some(filtered_children)
                };
            }

            let shadow_match = if let Some(shadow) = &element.shadow {
                let filtered_shadow_elements = filter_elements_by(&shadow.elements, matches);
                if filtered_shadow_elements.is_empty() {
                    cloned.shadow = None;
                    false
                } else {
                    cloned.shadow = Some(ShadowRoot {
                        mode: shadow.mode.clone(),
                        elements: filtered_shadow_elements,
                    });
                    true
                }
            } else {
                false
            };

            let matches_self = matches(element);
            if matches_self {
                // A selector targets the element, not just its role marker.
                // Keep the matched element's ordinary authored children so
                // selecting a button, section, or other container does not
                // erase its nested label/content. Shadow-root content stays
                // selector-filtered so unrelated encapsulated content is not
                // exposed just because its host matched.
                cloned.children = element.children.clone();
            }

            if matches_self || cloned.children.is_some() || shadow_match {
                Some(cloned)
            } else {
                None
            }
        })
        .collect()
}

fn filter_elements_by_id(elements: &[Element], id: &str) -> Vec<Element> {
    elements
        .iter()
        .filter_map(|element| {
            let mut cloned = element.clone();
            let matches_self = element.id == id || element.html_id.as_deref() == Some(id);
            if let Some(children) = &element.children {
                let filtered_children = filter_elements_by_id(children, id);
                cloned.children = if filtered_children.is_empty() {
                    None
                } else {
                    Some(filtered_children)
                };
            }

            let shadow_match = if let Some(shadow) = &element.shadow {
                let filtered_shadow_elements = filter_elements_by_id(&shadow.elements, id);
                if filtered_shadow_elements.is_empty() {
                    cloned.shadow = None;
                    false
                } else {
                    cloned.shadow = Some(ShadowRoot {
                        mode: shadow.mode.clone(),
                        elements: filtered_shadow_elements,
                    });
                    true
                }
            } else {
                false
            };

            if matches_self {
                // An id selector targets the element itself. Preserve its
                // authored children just as role selectors do; descendants
                // remain filtered only when the match is found below it.
                cloned.children = element.children.clone();
            }

            if matches_self || cloned.children.is_some() || shadow_match {
                Some(cloned)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::som::types::*;

    fn make_test_som() -> Som {
        Som {
            som_version: "1".to_string(),
            url: "https://example.com".to_string(),
            title: "Test".to_string(),
            lang: "en".to_string(),
            regions: vec![
                Region {
                    id: "r1".to_string(),
                    role: RegionRole::Navigation,
                    label: None,
                    action: None,
                    method: None,
                    target: None,
                    enctype: None,
                    novalidate: None,
                    accept_charset: None,
                    autocomplete: None,
                    elements: vec![Element {
                        id: "e1".to_string(),
                        role: ElementRole::Link,
                        html_id: None,
                        text: Some("Home".to_string()),
                        label: None,
                        actions: Some(vec!["click".to_string()]),
                        attrs: None,
                        children: None,
                        hints: None,
                        shadow: None,
                    }],
                },
                Region {
                    id: "r2".to_string(),
                    role: RegionRole::Main,
                    label: None,
                    action: None,
                    method: None,
                    target: None,
                    enctype: None,
                    novalidate: None,
                    accept_charset: None,
                    autocomplete: None,
                    elements: vec![
                        Element {
                            id: "e2".to_string(),
                            role: ElementRole::Paragraph,
                            html_id: Some("intro".to_string()),
                            text: Some("Hello world".to_string()),
                            label: None,
                            actions: None,
                            attrs: None,
                            children: None,
                            hints: None,
                            shadow: None,
                        },
                        Element {
                            id: "e3".to_string(),
                            role: ElementRole::Button,
                            html_id: Some("save".to_string()),
                            text: Some("Save".to_string()),
                            label: None,
                            actions: Some(vec!["click".to_string()]),
                            attrs: None,
                            children: None,
                            hints: None,
                            shadow: None,
                        },
                    ],
                },
            ],
            meta: SomMeta {
                html_bytes: 100,
                som_bytes: 50,
                element_count: 3,
                interactive_count: 2,
            },
            structured_data: None,
        }
    }

    #[test]
    fn test_selector_main() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "main");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].role, RegionRole::Main);
    }

    #[test]
    fn test_selector_landmark_inventory_role_aliases() {
        let som = make_test_som();

        for selector in [
            "role=landmark_inventory",
            "role:landmark-inventories",
            "role landmark",
            "role landmarks",
            "role landmark-region",
            "role landmark-regions",
            "role landmark-inventory",
            "role landmark-inventories",
            "role landmark-region-inventory",
            "role=landmark region inventory",
            "role:landmark region inventories",
            "role landmark-region-inventories",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), som.regions.len(), "selector: {selector}");
            assert!(filtered.regions.iter().all(|region| region.elements.is_empty()));
        }
    }

    #[test]
    fn test_selector_main_content_aliases() {
        let som = make_test_som();
        for selector in [
            "main_content",
            "main_contents",
            "main-content",
            "main-contents",
            "main content",
            "main contents",
            "main_content_region",
            "main_content_regions",
            "main-content-region",
            "main-content-regions",
            "main content region",
            "main content regions",
            "maincontent",
            "maincontents",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].role, RegionRole::Main);
        }
    }

    #[test]
    fn test_selector_primary_content_aliases() {
        let som = make_test_som();
        for selector in [
            "primary",
            "primary_content",
            "primary_contents",
            "primary-content",
            "primary-contents",
            "primary content",
            "primary contents",
            "primary_region",
            "primary_regions",
            "primary-region",
            "primary-regions",
            "primary region",
            "primary regions",
            "primary_content_region",
            "primary_content_regions",
            "primary-content-region",
            "primary-content-regions",
            "primary content region",
            "primary content regions",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].role, RegionRole::Main);
        }
    }

    #[test]
    fn test_selector_accepts_quoted_bare_values() {
        let som = make_test_som();

        for selector in [
            "\"main\"",
            "'main'",
            "`main`",
            "\" button \"",
            "' button '",
            "` button `",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].role, RegionRole::Main);
        }
    }

    #[test]
    fn selector_whitespace_collapse_preserves_quoted_values() {
        assert_eq!(
            collapse_selector_whitespace("  id = \"account  primary\"  "),
            "id = \"account  primary\""
        );
        assert_eq!(
            collapse_selector_whitespace("label: 'New   York'"),
            "label: 'New   York'"
        );
        assert_eq!(
            collapse_selector_whitespace(r#"label: "New \" York"  =  value"#),
            r#"label: "New \" York" = value"#
        );
    }

    #[test]
    fn selector_quote_stripping_keeps_escaped_final_delimiters() {
        assert_eq!(strip_selector_quotes(r#""account\""#), r#""account\""#);
        assert_eq!(strip_selector_quotes(r#"'account\\'"#), r#"account\\"#);
        assert_eq!(strip_selector_quotes(r#"`account`"#), "account");
    }

    #[test]
    fn test_selector_accepts_plural_aria_paragraph_aliases() {
        let som = make_test_som();

        for selector in ["alerts", "statuses", "tooltips"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Paragraph);
        }
    }

    #[test]
    fn test_selector_matches_accessible_labels_and_text() {
        let mut som = make_test_som();
        som.regions[1].elements[0].label = Some("Billing\n  address".to_string());
        som.regions[1].elements[1].text = Some("Save\tchanges".to_string());

        for selector in [
            "label:billing",
            "label=\"BILLING ADDRESS\"",
            "text:billing address",
            "text=\"BILLING ADDRESS\"",
            "aria-label:billing address",
            "aria_label=\"BILLING ADDRESS\"",
            "accessible-name:billing address",
            "accessible_name=\"BILLING ADDRESS\"",
            "accessible-label:billing address",
            "accessible_label=\"BILLING ADDRESS\"",
            "name:billing address",
            "name=\"BILLING ADDRESS\"",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].html_id.as_deref(), Some("intro"));
        }

        let filtered = apply_selector(&som, "label:save");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].html_id.as_deref(), Some("save"));
    }

    #[test]
    fn test_selector_matches_aria_braille_label_aliases() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-braille-label".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"aria-braillelabel": "Braille Save"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "aria-braillelabel:braille save",
            "aria-braille-label:braille save",
            "aria_braillelabel=BRAILLE SAVE",
            "aria_braille_label=BRAILLE SAVE",
            "aria braillelabel:braille save",
            "aria braille label:braille save",
            "braillelabel:braille save",
            "braille-label=BRAILLE SAVE",
            "braille label:braille save",
            "braille:braille save",
            "braille:'BRAILLE SAVE'",
            "selector=braille:\"braille save\"",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-braille-label");
        }

        let filtered = apply_selector(&som, "[aria-braillelabel]");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-braille-label");
    }

    #[test]
    fn test_selector_matches_legacy_label_fallback_attributes() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-placeholder".to_string(),
            role: ElementRole::TextInput,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"placeholder": "Search products"})),
            children: None,
            hints: None,
            shadow: None,
        });

        let filtered = apply_selector(&som, "label:SEARCH PRODUCTS");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-placeholder");
    }

    #[test]
    fn test_selector_matches_title_alias() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-tooltip".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"title": "Open settings"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in ["title:settings", "title=OPEN SETTINGS"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-tooltip");
        }
    }

    #[test]
    fn test_selector_matches_stable_test_id_aliases() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-test-hook".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"test_id": "save-action"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "testid:action",
            "test-id=SAVE-ACTION",
            "data-testid:save",
            "data-test-id=SAVE-ACTION",
            "data_test_id:save",
            "datatestid=SAVE-ACTION",
            "data-cy=SAVE-ACTION",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-test-hook");
        }
    }

    #[test]
    fn test_selector_matches_css_attribute_aliases() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-css-hook".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"data-testid": "save-action"})),
            children: None,
            hints: None,
            shadow: None,
        });

        let filtered = apply_selector(&som, "[data-testid=\"SAVE-ACTION\"]");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-css-hook");
    }

    #[test]
    fn test_selector_matches_css_attribute_string_operators() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-css-string-operators".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"data-testid": "save-primary-action"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "[data-testid^='save-']",
            "[data-testid$='-action']",
            "[data-testid*='primary']",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-css-string-operators");
        }
    }

    #[test]
    fn test_selector_accepts_css_attribute_sensitivity_flags() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-css-flag".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"data-state": "open"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in ["[data-state='OPEN' i]", "[data-state='OPEN' s]"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-css-flag");
        }
    }

    #[test]
    fn test_selector_matches_css_class_tokens() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-card".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"class": "card featured"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [".card", "[class~='FEATURED']", "[class=card featured]"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-card");
        }

        let filtered = apply_selector(&som, ".card.featured");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-card");

        for selector in [
            "class:card",
            "class=FEATURED",
            "CLASS card",
            "class~=FEATURED",
            "CLASS ~ = 'card'",
            "class:card.featured",
            "class~=card.featured",
            "class='card.featured'",
            "CLASS ~ = \"card.featured\"",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-card");
        }

        let prefix_collision = apply_selector(&som, ".car");
        assert_eq!(prefix_collision.regions.len(), som.regions.len());
        assert_eq!(prefix_collision.regions[1].elements.len(), som.regions[1].elements.len());
    }

    #[test]
    fn test_selector_matches_compact_class_string_operators() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-card-prefix".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"class": "card featured"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "class^=card",
            "class$=featured",
            "class*=feat",
            "class ^ = 'card'",
            "class $ = \"featured\"",
            "class * = feat",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-card-prefix");
        }
    }

    #[test]
    fn test_selector_rejects_values_over_max_bytes() {
        let som = make_test_som();
        let selector = "a".repeat(MAX_SELECTOR_BYTES + 1);

        let filtered = apply_selector(&som, &selector);

        assert_eq!(
            serde_json::to_value(filtered).unwrap(),
            serde_json::to_value(som).unwrap()
        );
    }

    #[test]
    fn test_selector_matches_css_attribute_presence_aliases() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-css-presence-hook".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"data-cy": "save-action"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "[testid]",
            "[test-id]",
            "[test_id]",
            "[data-testid]",
            "[datatestid]",
            "[data-test-id]",
            "[data_test_id]",
            "[data-test]",
            "[data-qa]",
            "[data-cy]",
            "[data-automation-id]",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-css-presence-hook");
        }
    }

    #[test]
    fn test_selector_matches_css_boolean_attribute_presence() {
        let mut som = make_test_som();
        som.regions[1].elements[0].attrs = Some(serde_json::json!({
            "disabled": true,
            "required": true,
            "checked": true,
        }));

        for selector in ["[disabled]", "[required]", "[checked]"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e2");
        }
    }

    #[test]
    fn test_selector_matches_additional_html_boolean_attribute_presence() {
        let mut som = make_test_som();
        som.regions[1].elements[0].attrs = Some(serde_json::json!({
            "default": true,
            "formnovalidate": true,
            "hidden": true,
            "ismap": true,
            "playsinline": true,
        }));

        for selector in [
            "[default]",
            "[formnovalidate]",
            "[hidden]",
            "[ismap]",
            "[playsinline]",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e2");
        }
    }

    #[test]
    fn test_selector_matches_css_role_presence() {
        let mut som = make_test_som();
        for region in &mut som.regions {
            region.elements.clear();
        }
        som.regions[1].elements.push(Element {
            id: "e-role-presence".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        });

        let filtered = apply_selector(&som, "[role]");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-role-presence");
    }

    #[test]
    fn test_selector_matches_authored_html_id_presence_and_value() {
        let mut som = make_test_som();
        for region in &mut som.regions {
            for element in &mut region.elements {
                element.html_id = None;
            }
        }
        som.regions[1].elements.push(Element {
            id: "e-generated-id".to_string(),
            role: ElementRole::Button,
            html_id: Some("save-action".to_string()),
            text: None,
            label: None,
            actions: None,
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in ["[id]", "[id=save-action]"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-generated-id");
        }

        let case_mismatch = apply_selector(&som, "[id=SAVE-ACTION]");
        assert_eq!(case_mismatch.regions.len(), som.regions.len());
        assert_eq!(case_mismatch.regions[1].elements.len(), 1);
    }

    #[test]
    fn test_selector_matches_aria_live_presence_aliases() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-live-region".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: Some("Saved".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"aria": {"live": "polite"}})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in ["[aria-live]", "[aria_live]"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-live-region");
        }
    }

    #[test]
    fn test_selector_matches_css_accessible_attribute_presence() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-accessible-presence-hook".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({
                "aria-label": "Save",
                "aria-labelledby": "save-label",
                "aria-describedby": "save-description",
                "aria-description": "Save this document",
                "title": "Save",
                "placeholder": "Save",
                "alt": "Save"
            })),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "[aria-label]",
            "[aria_label]",
            "[aria-labelledby]",
            "[aria_labelledby]",
            "[aria-describedby]",
            "[aria_describedby]",
            "[aria-description]",
            "[aria_description]",
            "[title]",
            "[placeholder]",
            "[alt]",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-accessible-presence-hook");
        }
    }

    #[test]
    fn test_selector_matches_css_aria_state_presence() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-aria-state-presence-hook".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({
                "aria-controls": "menu",
                "aria-current": "page",
                "aria-expanded": "false",
                "aria-haspopup": "menu",
                "aria-pressed": "false",
                "aria-selected": "true",
                "aria-sort": "ascending"
            })),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "[aria-controls]",
            "[aria_controls]",
            "[aria-current]",
            "[aria-expanded]",
            "[aria_haspopup]",
            "[aria-pressed]",
            "[aria-selected]",
            "[aria-sort]",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-aria-state-presence-hook");
        }
    }

    #[test]
    fn test_selector_matches_css_aria_relevant_presence_and_value() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-aria-relevant".to_string(),
            role: ElementRole::Paragraph,
            html_id: None,
            text: Some("Updates".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"aria-relevant": "additions text"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in ["[aria-relevant]", "[aria-relevant~=text]"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-aria-relevant");
        }
    }

    #[test]
    fn test_selector_matches_css_aria_atomic_presence_and_value() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-aria-atomic".to_string(),
            role: ElementRole::Paragraph,
            html_id: None,
            text: Some("Atomic update".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"aria-atomic": "true"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in ["[aria-atomic]", "[aria-atomic=true]"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-aria-atomic");
        }
    }

    #[test]
    fn test_selector_matches_css_aria_error_and_placeholder_attributes() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-aria-error-placeholder".to_string(),
            role: ElementRole::TextInput,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({
                "aria-errormessage": "email-error",
                "aria-placeholder": "name@example.com"
            })),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "[aria-errormessage]",
            "[aria-errormessage=email-error]",
            "[aria-placeholder]",
            "[aria-placeholder='name@example.com']",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-aria-error-placeholder");
        }
    }

    #[test]
    fn test_selector_matches_css_dash_prefix_values() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-language".to_string(),
            role: ElementRole::Paragraph,
            html_id: None,
            text: Some("Localized content".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"lang": "en-US"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "[lang|=en]",
            "[lang|='en']",
            "lang:en",
            "lang|=en",
            "lang:en-US",
            "lang=en-US",
            "lang en-US",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-language");
        }

        let filtered = apply_selector(&som, "[lang|=fr]");
        assert_eq!(
            serde_json::to_value(filtered).unwrap(),
            serde_json::to_value(som).unwrap()
        );
    }

    #[test]
    fn test_selector_matches_direction_attribute() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-rtl".to_string(),
            role: ElementRole::Paragraph,
            html_id: None,
            text: Some("مرحبا".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"dir": "rtl"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "[dir]",
            "[dir=rtl]",
            "[dir=\"rtl\"]",
            "dir",
            "dir:rtl",
            "dir=rtl",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-rtl");
        }

        som.regions[1].elements[0].attrs = Some(serde_json::json!({"dir": "RTL"}));
        let filtered = apply_selector(&som, "dir=rtl");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-rtl");
    }

    #[test]
    fn test_selector_matches_css_aria_sort_value() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-aria-sort-value".to_string(),
            role: ElementRole::Paragraph,
            html_id: None,
            text: Some("Name".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"aria-sort": "ascending"})),
            children: None,
            hints: None,
            shadow: None,
        });

        let filtered = apply_selector(&som, "[aria-sort=ascending]");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-aria-sort-value");
    }

    #[test]
    fn test_selector_matches_arbitrary_data_attributes() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-data-state".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: Some("Save".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"data-state": "open", "data_test_variant": "primary"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "[data-state]",
            "[data-state=open]",
            "[data-state=\"OPEN\" i]",
            "[data_test_variant=primary]",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-data-state", "selector: {selector}");
        }
    }

    #[test]
    fn test_selector_matches_compact_arbitrary_data_attributes() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-compact-data-state".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: Some("Save".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"data-state": "open"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in ["data-state:open", "data-state=OPEN"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-compact-data-state");
        }
    }

    #[test]
    fn test_selector_matches_compact_data_attribute_tokens() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-compact-data-tags".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: Some("Save".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"data-tags": "primary compact"})),
            children: None,
            hints: None,
            shadow: None,
        });

        let filtered = apply_selector(&som, "data-tags~=PRIMARY");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-compact-data-tags");
    }

    #[test]
    fn test_selector_matches_css_common_aria_state_presence() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-common-aria-state-presence-hook".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({
                "aria-busy": "true",
                "aria-checked": "true",
                "aria-disabled": "false",
                "aria-hidden": "false",
                "aria-modal": "true",
                "inert": ""
            })),
            children: None,
            hints: None,
            shadow: None,
        });
        som.regions[1].elements.push(Element {
            id: "e-common-aria-state-without-checked".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"aria-disabled": "false"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "[aria-busy]",
            "[aria-checked]",
            "[aria_disabled]",
            "[aria-hidden]",
            "[aria-modal]",
            "[aria_modal]",
            "[inert]",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(
                filtered.regions[0].elements[0].id,
                "e-common-aria-state-presence-hook"
            );
        }
    }

    #[test]
    fn test_selector_matches_css_aria_state_values_and_validation_presence() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-aria-value-hook".to_string(),
            role: ElementRole::TextInput,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({
                "aria-expanded": "false",
                "aria-invalid": "true",
                "aria-required": "true"
            })),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "[aria-expanded='false']",
            "[aria-invalid]",
            "[aria_invalid='true']",
            "[aria-required]",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-aria-value-hook");
        }
    }

    #[test]
    fn test_selector_matches_css_attribute_values_on_the_named_attribute() {
        let mut som = make_test_som();
        som.regions[1].elements.extend([
            Element {
                id: "e-modal-false-label-true".to_string(),
                role: ElementRole::Button,
                html_id: None,
                text: None,
                label: Some("true".to_string()),
                actions: None,
                attrs: Some(serde_json::json!({"aria-modal": "false"})),
                children: None,
                hints: None,
                shadow: None,
            },
            Element {
                id: "e-modal-true".to_string(),
                role: ElementRole::Button,
                html_id: None,
                text: None,
                label: Some("dialog".to_string()),
                actions: None,
                attrs: Some(serde_json::json!({"aria-modal": "true"})),
                children: None,
                hints: None,
                shadow: None,
            },
        ]);

        let filtered = apply_selector(&som, "[aria-modal='true']");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-modal-true");
    }

    #[test]
    fn test_selector_matches_accesskey_attribute() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-accesskey".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"accesskey": "s"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "[accesskey]",
            "[accesskey='s']",
            "accesskey:s",
            "accesskey='s'",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-accesskey");
        }

        som.regions[1].elements[2].attrs = Some(serde_json::json!({"accesskey": "x s"}));
        let filtered = apply_selector(&som, "accesskey:s");
        assert_eq!(filtered.regions[0].elements[0].id, "e-accesskey");
    }

    #[test]
    fn test_selector_matches_css_role_values() {
        let mut som = make_test_som();
        som.regions[1].elements.extend([
            Element {
                id: "e-role-button".to_string(),
                role: ElementRole::Button,
                html_id: None,
                text: Some("Save".to_string()),
                label: None,
                actions: None,
                attrs: None,
                children: None,
                hints: None,
                shadow: None,
            },
            Element {
                id: "e-role-link".to_string(),
                role: ElementRole::Link,
                html_id: None,
                text: Some("Cancel".to_string()),
                label: None,
                actions: None,
                attrs: None,
                children: None,
                hints: None,
                shadow: None,
            },
        ]);

        let filtered = apply_selector(&som, "[ROLE='BUTTON']");

        let ids: Vec<_> = filtered.regions[0]
            .elements
            .iter()
            .map(|element| element.id.as_str())
            .collect();
        assert!(ids.contains(&"e-role-button"));
        assert!(!ids.contains(&"e-role-link"));
    }

    #[test]
    fn test_selector_matches_css_aria_range_value_attributes() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-aria-range-hook".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({
                "aria": {
                    "valuemax": "100",
                    "valuemin": "0",
                    "valuenow": "50",
                    "valuetext": "Half"
                }
            })),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "[aria-valuemax]",
            "[aria_valuemin]",
            "[aria-valuenow='50']",
            "[aria-valuetext='Half']",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-aria-range-hook");
        }
    }

    #[test]
    fn test_selector_matches_nested_aria_reference_attributes() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-nested-aria-reference-hook".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({
                "aria": {
                    "active_descendant": "option-1",
                    "details": "save-details",
                    "flowto": "next-step",
                    "keyshortcuts": "Meta+S",
                    "owns": "menu",
                    "roledescription": "primary action"
                }
            })),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "[aria-activedescendant]",
            "[aria-details]",
            "[aria-flowto]",
            "[aria-keyshortcuts]",
            "[aria-owns]",
            "[aria-roledescription]",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-nested-aria-reference-hook");
        }

        let filtered = apply_selector(&som, "[aria-roledescription='primary action']");
        assert_eq!(filtered.regions[0].elements[0].id, "e-nested-aria-reference-hook");
    }

    #[test]
    fn test_selector_matches_css_data_test_alias() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-css-data-test".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"data-test": "save-action"})),
            children: None,
            hints: None,
            shadow: None,
        });

        let filtered = apply_selector(&som, "[data-test='SAVE-ACTION']");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-css-data-test");
    }

    #[test]
    fn test_selector_matches_css_id_operators_case_insensitively() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-css-id".to_string(),
            role: ElementRole::Button,
            html_id: Some("SavePrimary".to_string()),
            text: None,
            label: None,
            actions: None,
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in ["[id^='save']", "[id$='PRIMARY']", "[id*='vepri']"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-css-id");
        }
    }

    #[test]
    fn test_selector_matches_raw_test_id_attributes() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-raw-test-hook".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"data-testid": "save-action"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in ["data-testid:save", "data-cy=SAVE-ACTION"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-raw-test-hook");
        }
    }

    #[test]
    fn test_selector_matches_data_automation_id_aliases() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-automation-hook".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"data-automation-id": "save-action"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "data-automation-id:save-action",
            "[data-automation-id='SAVE-ACTION']",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-automation-hook");
        }
    }

    #[test]
    fn test_selector_matches_aria_label_fallback_attribute() {
        let mut som = make_test_som();
        som.regions[1].elements[1].label = None;
        som.regions[1].elements[1].text = None;
        som.regions[1].elements[1].attrs = Some(serde_json::json!({
            "aria-label": "Search products"
        }));

        let filtered = apply_selector(&som, "label:products");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e3");
    }

    #[test]
    fn test_label_selector_matches_data_cy_attribute() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-cy-label".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"data-cy": "save-action"})),
            children: None,
            hints: None,
            shadow: None,
        });

        let filtered = apply_selector(&som, "label:save-action");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-cy-label");
    }

    #[test]
    fn test_selector_matches_compiled_aria_label_metadata() {
        let mut som = make_test_som();
        som.regions[1].elements[1].label = None;
        som.regions[1].elements[1].text = None;
        som.regions[1].elements[1].attrs = Some(serde_json::json!({
            "aria": {"label": "Search products"}
        }));

        let filtered = apply_selector(&som, "label:products");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e3");
    }

    #[test]
    fn test_selector_matches_compiled_description_metadata() {
        let mut som = make_test_som();
        som.regions[1].elements[1].label = None;
        som.regions[1].elements[1].text = None;
        som.regions[1].elements[1].attrs = Some(serde_json::json!({
            "description": "Password must be at least 12 characters"
        }));

        let filtered = apply_selector(&som, "label:password");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e3");
    }

    #[test]
    fn test_selector_matches_nested_aria_description_metadata() {
        let mut som = make_test_som();
        som.regions[1].elements[1].label = None;
        som.regions[1].elements[1].text = None;
        som.regions[1].elements[1].attrs = Some(serde_json::json!({
            "aria": {"description": "Password requirements"}
        }));

        let filtered = apply_selector(&som, "label:requirements");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e3");
    }

    #[test]
    fn test_selector_matches_flat_aria_description_fallback() {
        let mut som = make_test_som();
        som.regions[1].elements[1].label = None;
        som.regions[1].elements[1].text = None;
        som.regions[1].elements[1].attrs = Some(serde_json::json!({
            "aria-description": "Password requirements"
        }));

        let filtered = apply_selector(&som, "label:requirements");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e3");
    }

    #[test]
    fn test_selector_matches_named_aria_roledescription() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-roledescription".to_string(),
            role: ElementRole::Group,
            html_id: None,
            text: Some("Upload".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"aria-roledescription": "drop zone"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in ["roledescription:drop zone", "aria-roledescription=DROP ZONE"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-roledescription");
        }
    }

    #[test]
    fn test_selector_matches_named_aria_live_values() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-live-region".to_string(),
            role: ElementRole::Group,
            html_id: None,
            text: Some("Updated".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"aria-live": "polite"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in ["live:polite", "aria-live=POLITE", "aria live:polite"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-live-region");
        }
    }

    #[test]
    fn test_selector_matches_compact_attribute_presence() {
        let mut som = make_test_som();
        som.regions[1].elements[0].attrs = Some(serde_json::json!({
            "aria-label": "Introduction",
            "data-testid": "intro",
            "title": "Intro",
            "inert": ""
        }));

        for selector in ["aria-label", "data-testid", "title", "inert"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e2");
        }
    }

    #[test]
    fn test_selector_matches_compact_boolean_attribute_presence() {
        let mut som = make_test_som();
        som.regions[1].elements[0].attrs = Some(serde_json::json!({
            "disabled": true,
            "required": "required",
            "data-state": "open",
            "hidden": true,
            "formnovalidate": true,
            "ismap": true,
            "itemscope": true,
            "playsinline": true
        }));

        for selector in [
            "disabled",
            "required",
            "hidden",
            "formnovalidate",
            "ismap",
            "itemscope",
            "playsinline",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e2");
        }
    }

    #[test]
    fn test_selector_matches_generic_aria_live_regions() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-live-region".to_string(),
            role: ElementRole::Paragraph,
            html_id: None,
            text: Some("Updated".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"aria": {"live": "polite"}})),
            children: None,
            hints: None,
            shadow: None,
        });
        som.regions[1].elements.push(Element {
            id: "e-status-region".to_string(),
            role: ElementRole::Paragraph,
            html_id: None,
            text: Some("Saved".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"source_role": "status"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in ["live", "live-region", "live_regions"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 2, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-live-region");
            assert_eq!(filtered.regions[0].elements[1].id, "e-status-region");
        }
    }

    #[test]
    fn test_selector_matches_implicit_live_roles_without_other_paragraphs() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-status-region".to_string(),
            role: ElementRole::Paragraph,
            html_id: None,
            text: Some("Saved".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"source_role": "status"})),
            children: None,
            hints: None,
            shadow: None,
        });
        som.regions[1].elements.push(Element {
            id: "e-alert-region".to_string(),
            role: ElementRole::Paragraph,
            html_id: None,
            text: Some("Failed".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"source_role": "alert"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in ["status", "role=status", "alert"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            if selector == "alert" {
                assert_eq!(filtered.regions[0].elements[0].id, "e-alert-region");
            } else {
                assert_eq!(filtered.regions[0].elements[0].id, "e-status-region");
            }
        }
    }

    #[test]
    fn test_selector_matches_aria_labelledby_and_describedby_fallbacks() {
        let mut som = make_test_som();
        som.regions[1].elements[1].label = None;
        som.regions[1].elements[1].text = None;
        som.regions[1].elements[1].attrs = Some(serde_json::json!({
            "aria-labelledby": "Search products",
            "aria-describedby": "Search suggestions"
        }));

        for selector in ["label:products", "description:suggestions"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e3");
        }
    }

    #[test]
    fn test_selector_matches_nested_aria_labelledby_and_describedby_fallbacks() {
        let mut som = make_test_som();
        som.regions[1].elements[1].label = None;
        som.regions[1].elements[1].text = None;
        som.regions[1].elements[1].attrs = Some(serde_json::json!({
            "aria": {
                "aria-labelledby": "Search products",
                "aria-describedby": "Search suggestions"
            }
        }));

        for selector in ["label:products", "description:suggestions"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e3");
        }
    }

    #[test]
    fn test_selector_matches_compiled_nested_aria_reference_keys() {
        let mut som = make_test_som();
        som.regions[1].elements[1].label = None;
        som.regions[1].elements[1].text = None;
        som.regions[1].elements[1].attrs = Some(serde_json::json!({
            "aria": {
                "labelledby": "Search products",
                "describedby": "Search suggestions"
            }
        }));

        for selector in ["label:products", "description:suggestions"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e3");
        }
    }

    #[test]
    fn test_selector_accepts_description_aliases() {
        let mut som = make_test_som();
        som.regions[1].elements[1].label = None;
        som.regions[1].elements[1].text = None;
        som.regions[1].elements[1].attrs = Some(serde_json::json!({
            "aria-description": "Password requirements"
        }));

        for selector in [
            "description:requirements",
            "aria-description=requirements",
            "aria_description:requirements",
            "aria description=requirements",
            "accessible-description:requirements",
            "accessible_description=requirements",
            "accessible description:requirements",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e3");
        }
    }

    #[test]
    fn test_selector_checks_all_label_fallback_attributes() {
        let mut som = make_test_som();
        som.regions[1].elements[1].label = None;
        som.regions[1].elements[1].text = None;
        som.regions[1].elements[1].attrs = Some(serde_json::json!({
            "aria-label": "Close",
            "title": "Billing details",
            "placeholder": "Search products"
        }));

        let filtered = apply_selector(&som, "label:billing");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e3");
    }

    #[test]
    fn test_selector_matches_image_alt_fallback_attribute() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-image".to_string(),
            role: ElementRole::Image,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"alt": "Product preview"})),
            children: None,
            hints: None,
            shadow: None,
        });

        let filtered = apply_selector(&som, "label:preview");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-image");
    }

    #[test]
    fn test_selector_matches_image_alt_alias() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-image".to_string(),
            role: ElementRole::Image,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"alt": "Product preview"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "alt:preview",
            "alt=PRODUCT",
            "alt-text:preview",
            "alt_text=PRODUCT",
            "alttext:PRODUCT",
            "alt text:preview",
            "alternative=PRODUCT",
            "alternative-text:preview",
            "alternative_text=PRODUCT",
            "alternative text:preview",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-image");
        }
    }

    #[test]
    fn test_selector_alt_alias_ignores_unrelated_accessible_text() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-image".to_string(),
            role: ElementRole::Image,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"alt": "Product preview"})),
            children: None,
            hints: None,
            shadow: None,
        });
        som.regions[1].elements.push(Element {
            id: "e-title".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"title": "Product preview"})),
            children: None,
            hints: None,
            shadow: None,
        });

        let filtered = apply_selector(&som, "alt:preview");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-image");
    }

    #[test]
    fn test_selector_matches_region_labels() {
        let mut som = make_test_som();
        som.regions[1].label = Some("Account\n  filters".to_string());

        let filtered = apply_selector(&som, "label=ACCOUNT FILTERS");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].label.as_deref(), Some("Account\n  filters"));
        assert_eq!(filtered.regions[0].elements.len(), 2);
    }

    #[test]
    fn test_selector_role_aliases() {
        let som = make_test_som();

        for selector in ["role=main", "ROLE:MAIN", "role = main", "role : main"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].role, RegionRole::Main);
        }
    }

    #[test]
    fn test_selector_plural_region_aliases() {
        for (selector, role) in [
            ("navigations", RegionRole::Navigation),
            ("asides", RegionRole::Aside),
            ("sidebar", RegionRole::Aside),
            ("sidebars", RegionRole::Aside),
            ("complementaries", RegionRole::Aside),
            ("role=sidebar", RegionRole::Aside),
            ("role:sidebars", RegionRole::Aside),
            ("headers", RegionRole::Header),
            ("banners", RegionRole::Header),
            ("footers", RegionRole::Footer),
            ("contents", RegionRole::Content),
            ("content-info", RegionRole::Footer),
            ("content_info", RegionRole::Footer),
            ("content-infos", RegionRole::Footer),
            ("content_infos", RegionRole::Footer),
        ] {
            let mut som = make_test_som();
            som.regions[0].role = role.clone();
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].role, role, "selector: {selector}");
        }
    }

    #[test]
    fn test_selector_natural_region_aliases() {
        for (selector, role) in [
            ("main region", RegionRole::Main),
            ("main-region", RegionRole::Main),
            ("main regions", RegionRole::Main),
            ("main-regions", RegionRole::Main),
            ("main_region", RegionRole::Main),
            ("main_regions", RegionRole::Main),
            ("content region", RegionRole::Content),
            ("content-regions", RegionRole::Content),
            ("content_region", RegionRole::Content),
            ("content_regions", RegionRole::Content),
        ] {
            let mut som = make_test_som();
            som.regions[0].role = role.clone();
            som.regions[1].role = match role {
                RegionRole::Main => RegionRole::Content,
                RegionRole::Content => RegionRole::Main,
                _ => unreachable!("test only covers main/content aliases"),
            };
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].role, role, "selector: {selector}");
        }
    }

    #[test]
    fn test_selector_navigation_region_aliases() {
        for selector in [
            "nav region",
            "nav-region",
            "nav_regions",
            "navigation region",
            "navigation-region",
            "navigation regions",
        ] {
            let som = make_test_som();
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].role, RegionRole::Navigation);
        }
    }

    #[test]
    fn test_selector_search_navigation_aliases() {
        for selector in [
            "search",
            "search_nav",
            "search-nav",
            "search_navigation",
            "search-navigation",
            "search navigation",
            "search_navigations",
            "search-navigations",
            "search navigations",
            "search   navigation",
            "  search\tnavigation  ",
        ] {
            let som = make_test_som();
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].role, RegionRole::Navigation);
        }
    }

    #[test]
    fn test_selector_role_aliases_for_element_roles() {
        let som = make_test_som();

        for selector in [
            "role=button",
            "ROLE:BUTTON",
            "role = button",
            "role : button",
            "role button",
            " ROLE\tBUTTON ",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1);
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Button);
        }
    }

    #[test]
    fn test_selector_accepts_anchor_aliases_for_links() {
        let som = make_test_som();

        for selector in [
            "anchor",
            "anchors",
            "role=anchor",
            "role:anchors",
            "hyperlink",
            "hyperlinks",
            "role=hyperlink",
            "role:hyperlinks",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(
                filtered.regions[0].elements.len(),
                1,
                "selector: {selector}"
            );
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Link);
        }
    }

    #[test]
    fn test_selector_accepts_area_aliases_for_links() {
        let som = make_test_som();

        for selector in ["area", "areas", "role=area", "role:areas"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Link);
        }
    }

    #[test]
    fn test_selector_accepts_inline_text_tag_aliases_for_paragraphs() {
        let som = make_test_som();

        for selector in ["s", "role=s", "ins", "role=ins", "del", "role:del", "mark"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Paragraph);
        }
    }

    #[test]
    fn test_selector_time_matches_native_time_and_time_input() {
        let mut som = make_test_som();
        som.regions[1].elements[0].attrs = Some(serde_json::json!({"source_tag": "time"}));
        som.regions[1].elements.push(Element {
            id: "e-time-input".to_string(),
            role: ElementRole::TextInput,
            html_id: None,
            text: None,
            label: Some("Start time".to_string()),
            actions: Some(vec!["type".to_string()]),
            attrs: Some(serde_json::json!({"input_type": "time"})),
            children: None,
            shadow: None,
            hints: None,
        });

        for selector in ["time", "times", "TIME", "role=time", "role=times"] {
            let filtered = apply_selector(&som, selector);
            let ids = filtered.regions[0]
                .elements
                .iter()
                .map(|element| element.id.as_str())
                .collect::<Vec<_>>();
            assert_eq!(ids, vec!["e2", "e-time-input"], "selector: {selector}");
        }
    }

    #[test]
    fn test_selector_time_excludes_other_paragraphs_and_inputs() {
        let mut som = make_test_som();
        som.regions[1].elements[0].attrs = Some(serde_json::json!({"source_tag": "time"}));
        som.regions[1].elements.push(Element {
            id: "e-text-input".to_string(),
            role: ElementRole::TextInput,
            html_id: None,
            text: None,
            label: Some("Name".to_string()),
            actions: Some(vec!["type".to_string()]),
            attrs: Some(serde_json::json!({"input_type": "text"})),
            children: None,
            shadow: None,
            hints: None,
        });

        let filtered = apply_selector(&som, "time");
        let ids = filtered.regions[0]
            .elements
            .iter()
            .map(|element| element.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["e2"]);
    }

    #[test]
    fn test_selector_time_input_targets_only_time_controls() {
        let mut som = make_test_som();
        som.regions[1].elements[0].attrs = Some(serde_json::json!({"source_tag": "time"}));
        som.regions[1].elements.push(Element {
            id: "e-time-input".to_string(),
            role: ElementRole::TextInput,
            html_id: None,
            text: None,
            label: Some("Start time".to_string()),
            actions: Some(vec!["type".to_string()]),
            attrs: Some(serde_json::json!({"input_type": "time"})),
            children: None,
            shadow: None,
            hints: None,
        });

        for selector in [
            "time_input",
            "time_inputs",
            "timeinput",
            "timeinputs",
            "timecontrol",
            "timecontrols",
            "time-input",
            "time-inputs",
            "time input",
            "time inputs",
            "time_picker",
            "time_pickers",
            "timepicker",
            "timepickers",
            "time-picker",
            "time-pickers",
            "time picker",
            "time pickers",
        ] {
            let filtered = apply_selector(&som, selector);
            let ids = filtered.regions[0]
                .elements
                .iter()
                .map(|element| element.id.as_str())
                .collect::<Vec<_>>();
            assert_eq!(ids, vec!["e-time-input"], "selector: {selector}");
        }
    }

    #[test]
    fn test_selector_accepts_native_paragraph_tag_aliases() {
        let som = make_test_som();

        for selector in [
            "abbr", "address", "cite", "code", "dfn", "figcaption", "kbd", "math", "pre",
            "q", "ruby", "samp", "small", "var", "role=code", "role:var",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Paragraph);
        }
    }

    #[test]
    fn test_selector_accepts_aria_text_role_aliases_for_paragraphs() {
        let som = make_test_som();

        for selector in [
            "subscript",
            "role=subscript",
            "superscript",
            "role:superscript",
            "deletion",
            "insertion",
            "suggestion",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Paragraph);
        }
    }

    #[test]
    fn test_selector_accepts_remaining_compiled_text_aliases() {
        let som = make_test_som();

        for selector in [
            "definition",
            "role:definition",
            "term",
            "terms",
            "emphasis",
            "emphases",
            "strong",
            "strongs",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Paragraph);
        }
    }

    #[test]
    fn test_selector_role_interactive_alias() {
        let som = make_test_som();

        for selector in [
            "role=interactive",
            "ROLE:INTERACTIVE",
            "role = interactive",
            "interactives",
            "interactive element",
            "interactive-element",
            "interactive elements",
            "interactive_elements",
            "interactive-elements",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 2, "selector: {selector}");
            assert_eq!(
                filtered
                    .regions
                    .iter()
                    .flat_map(|region| region.elements.iter())
                    .count(),
                2,
                "selector: {selector}"
            );
            assert!(filtered.regions.iter().all(|region| {
                region.elements.iter().all(|element| element.role.is_interactive())
            }));
        }
    }

    #[test]
    fn test_selector_plural_text_role_aliases() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::Paragraph;

        for selector in ["paragraphs", "blockquotes", "quotes"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(
                filtered
                    .regions
                    .iter()
                    .flat_map(|region| region.elements.iter())
                    .filter(|element| element.role == ElementRole::Paragraph)
                    .count(),
                2,
                "selector: {selector}"
            );
        }
    }

    #[test]
    fn test_selector_plural_button_role_aliases() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::Button;

        for selector in ["menuitems", "options", "tabs"] {
            let filtered = apply_selector(&som, selector);
            assert!(!filtered.regions.is_empty(), "selector: {selector}");
            assert!(filtered.regions.iter().all(|region| {
                !region.elements.is_empty()
                    && region
                        .elements
                        .iter()
                        .all(|element| element.role == ElementRole::Button)
            }));
        }
    }

    #[test]
    fn test_selector_search_box_aliases() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::TextInput;

        for selector in ["search_box", "search-box", "search box"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1);
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::TextInput);
        }
    }

    #[test]
    fn test_selector_text_box_aliases() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::TextInput;

        for selector in [
            "text_box",
            "text-box",
            "text box",
            "text_boxes",
            "text-boxes",
            "text boxes",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::TextInput);
        }
    }

    #[test]
    fn test_selector_query_box_plural_aliases() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::TextInput;

        for selector in ["queryboxes", "query_boxes", "query-boxes", "query boxes"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::TextInput);
        }
    }

    #[test]
    fn test_selector_select_aliases_accept_plural_and_spaced_forms() {
        for selector in [
            "selects",
            "comboboxes",
            "combo box",
            "combo boxes",
            "combo-boxes",
            "listboxes",
            "list_box",
            "list_boxes",
            "list box",
            "list boxes",
        ] {
            let mut som = make_test_som();
            som.regions[0].elements[0].role = ElementRole::Select;
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Select);
        }
    }

    #[test]
    fn test_selector_accepts_spaced_compound_element_roles() {
        let mut som = make_test_som();
        for element in som.regions.iter_mut().flat_map(|region| region.elements.iter_mut()) {
            element.role = ElementRole::Paragraph;
        }
        som.regions[0].elements[0].role = ElementRole::TextInput;

        for (selector, role) in [
            ("text input", ElementRole::TextInput),
            ("text  input", ElementRole::TextInput),
            ("menu\titem", ElementRole::Button),
            ("unordered list", ElementRole::List),
        ] {
            som.regions[0].elements[0].role = role.clone();
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, role, "selector: {selector}");
        }
    }

    #[test]
    fn test_selector_structural_aliases() {
        for (selector, role) in [
            ("lists", ElementRole::List),
            ("tables", ElementRole::Table),
            ("data_tables", ElementRole::Table),
            ("data-tables", ElementRole::Table),
            ("data tables", ElementRole::Table),
            ("data-grid", ElementRole::Table),
            ("datagrid", ElementRole::Table),
            ("datagrids", ElementRole::Table),
            ("data grids", ElementRole::Table),
            ("table-grid", ElementRole::Table),
            ("tablegrid", ElementRole::Table),
            ("tablegrids", ElementRole::Table),
            ("table grids", ElementRole::Table),
            ("tree-grid", ElementRole::Table),
            ("tree grids", ElementRole::Table),
            ("separators", ElementRole::Separator),
            ("detail", ElementRole::Details),
            ("iframes", ElementRole::Iframe),
            ("groups", ElementRole::Group),
            ("fieldsets", ElementRole::Group),
            ("figures", ElementRole::Group),
            ("radiogroups", ElementRole::Group),
            ("progressbars", ElementRole::Group),
            ("meters", ElementRole::Group),
        ] {
            let mut som = make_test_som();
            som.regions[0].elements[0].role = role.clone();
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, role);
        }
    }

    #[test]
    fn test_selector_media_source_item_aliases_match_grouped_media() {
        for selector in [
            "audio_item",
            "audio-item",
            "audio item",
            "audioitem",
            "audio_items",
            "audio-items",
            "audio items",
            "audioitems",
            "video_item",
            "video-item",
            "video item",
            "videoitem",
            "video_items",
            "video-items",
            "video items",
            "videoitems",
            "canvas_item",
            "canvas-item",
            "canvas item",
            "canvasitem",
            "canvas_items",
            "canvas-items",
            "canvas items",
            "canvasitems",
        ] {
            let mut som = make_test_som();
            som.regions[0].elements[0].role = ElementRole::Group;
            let source_role = if selector.starts_with("audio") {
                "audio"
            } else if selector.starts_with("video") {
                "video"
            } else {
                "canvas"
            };
            som.regions[0].elements[0].attrs =
                Some(serde_json::json!({"source_role": source_role}));
            som.regions[1].elements[0].role = ElementRole::Group;
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Group);
        }
    }

    #[test]
    fn test_selector_plural_section_alias() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::Section;

        let filtered = apply_selector(&som, "sections");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Section);
    }

    #[test]
    fn test_selector_role_and_action_aliases_accept_quoted_values() {
        let som = make_test_som();

        for selector in [
            "role=\"button\"",
            "role='button'",
            "role=`button`",
            "role=\" button \"",
            "role=' button '",
            "role=` button `",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1);
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Button);
        }

        for selector in [
            "action=\"click\"",
            "action='click'",
            "action=`click`",
            "action=\" click \"",
            "action=' click '",
            "action=` click `",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 2, "selector: {selector}");
            assert!(filtered
                .regions
                .iter()
                .flat_map(|region| &region.elements)
                .all(|element| element
                    .actions
                    .as_ref()
                    .is_some_and(|actions| actions.iter().any(|action| action == "click"))));
        }
    }

    #[test]
    fn test_selector_id_accepts_quoted_values() {
        let mut som = make_test_som();
        som.regions[0].id = "hero".to_string();

        for selector in [
            "#hero",
            "#\"hero\"",
            "#'hero'",
            "\"#hero\"",
            "'#hero'",
            "#\" hero \"",
            "#' hero '",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].id, "hero");
        }
    }

    #[test]
    fn test_selector_role_aliases_for_actions() {
        let som = make_test_som();

        for selector in ["role=click", "ROLE:CLICK", "role = click", "role : click"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 2, "selector: {selector}");
            assert!(filtered
                .regions
                .iter()
                .flat_map(|region| &region.elements)
                .all(|element| element
                    .actions
                    .as_ref()
                    .is_some_and(|actions| actions.iter().any(|action| action == "click"))));
        }
    }

    #[test]
    fn test_selector_heading_name_aliases() {
        for selector in ["h1", "heading1", "HEADING1"] {
            assert_eq!(
                parse_heading_level_selector(selector),
                Some(1),
                "selector: {selector}"
            );
        }
        assert_eq!(parse_heading_level_selector("heading6"), Some(6));
        assert_eq!(parse_heading_level_selector("h-6"), Some(6));
        assert_eq!(parse_heading_level_selector("h_6"), Some(6));
        assert_eq!(parse_heading_level_selector("h 6"), Some(6));
        assert_eq!(parse_heading_level_selector("heading-6"), Some(6));
        assert_eq!(parse_heading_level_selector("heading_6"), Some(6));
        assert_eq!(parse_heading_level_selector("heading 6"), Some(6));
        assert_eq!(parse_heading_level_selector("heading7"), None);
        assert_eq!(parse_heading_level_selector("h01"), None);
        assert_eq!(parse_heading_level_selector("heading-01"), None);
    }

    #[test]
    fn test_selector_headings_plural_alias() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::Heading;

        let filtered = apply_selector(&som, "headings");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Heading);
    }

    #[test]
    fn test_selector_images_plural_alias() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::Image;

        let filtered = apply_selector(&som, "images");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Image);
    }

    #[test]
    fn test_selector_feed_aliases_match_list_elements() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::List;

        for selector in ["feed", "feeds"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::List);
        }
    }

    #[test]
    fn test_selector_picture_tag_aliases() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::Image;

        for selector in ["picture", "pictures"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Image);
        }
    }

    #[test]
    fn test_selector_image_item_aliases() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::Image;

        for selector in [
            "image_item",
            "image-item",
            "image item",
            "imageitem",
            "image_items",
            "image-items",
            "image items",
            "imageitems",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Image);
        }
    }

    #[test]
    fn test_selector_media_item_aliases_match_grouped_media() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::Image;

        for selector in ["media", "media_items", "media-items", "mediaitems"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Image);
        }
    }

    #[test]
    fn test_selector_media_item_singular_aliases_match_grouped_media() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::Image;

        for selector in ["media_item", "media-item", "media item", "mediaitem"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Image);
        }
    }

    #[test]
    fn test_selector_media_item_plural_spaced_alias_matches_grouped_media() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::Image;

        let filtered = apply_selector(&som, "media items");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Image);
    }

    #[test]
    fn test_selector_plural_control_aliases_match_roles() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::TextInput;
        som.regions[1].elements[0].role = ElementRole::Textarea;
        som.regions[1].elements.push(Element {
            id: "e-select".to_string(),
            role: ElementRole::Select,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        });

        for (selector, role) in [
            ("inputs", ElementRole::TextInput),
            ("text_inputs", ElementRole::TextInput),
            ("text_fields", ElementRole::TextInput),
            ("text-field", ElementRole::TextInput),
            ("text field", ElementRole::TextInput),
            ("textareas", ElementRole::Textarea),
            ("text_areas", ElementRole::Textarea),
            ("text-area", ElementRole::Textarea),
            ("text area", ElementRole::Textarea),
            ("selects", ElementRole::Select),
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.iter().flat_map(|r| &r.elements).count(), 1);
            assert_eq!(filtered.regions[0].elements[0].role, role, "selector: {selector}");
        }
    }

    #[test]
    fn test_selector_aria_widget_role_aliases() {
        for selector in [
            "menuitem",
            "menu-item",
            "menu_item",
            "menu items",
            "menu-items",
            "menu_items",
            "option",
            "options",
            "tab",
            "tabs",
            "treeitem",
            "treeitems",
            "tree-item",
            "tree_item",
            "tree items",
            "tree-items",
            "tree_items",
        ] {
            assert_eq!(parse_element_role(selector), Some(ElementRole::Button));
        }
        for selector in [
            "switch",
            "switches",
            "check box",
            "check boxes",
            "menuitemcheckbox",
            "menuitemcheckboxes",
            "menu-item-checkbox",
            "menu-item-checkboxes",
        ] {
            assert_eq!(parse_element_role(selector), Some(ElementRole::Checkbox));
        }
        for selector in [
            "menuitemradio",
            "menuitemradios",
            "radiobutton",
            "radiobuttons",
            "radio-button",
            "radio-buttons",
            "radio button",
            "radio buttons",
            "menu-item-radio",
            "menu-item-radios",
        ] {
            assert_eq!(parse_element_role(selector), Some(ElementRole::Radio));
        }
    }

    #[test]
    fn test_selector_live_region_role_aliases_match_compiled_paragraphs() {
        for selector in ["log", "logs", "marquee", "marquees", "timer", "timers"] {
            let mut som = make_test_som();
            som.regions[0].elements[0].role = ElementRole::Paragraph;
            som.regions[0].elements[0].attrs = Some(serde_json::json!({
                "source_role": live_region_source_role(selector).unwrap()
            }));
            som.regions[1].elements[0].role = ElementRole::Paragraph;
            som.regions[1].elements[0].attrs = Some(serde_json::json!({"source_role": "plain"}));

            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Paragraph);
        }
    }

    #[test]
    fn test_selector_plural_aria_toggle_aliases_filter_compiled_roles() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::Checkbox;
        som.regions[1].elements[0].role = ElementRole::Radio;

        for selector in [
            "switches",
            "check box",
            "check boxes",
            "menuitemcheckboxes",
            "menu-item-checkboxes",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Checkbox);
        }

        for selector in [
            "radiobutton",
            "radiobuttons",
            "radio-button",
            "radio-buttons",
            "radio button",
            "radio buttons",
            "menuitemradios",
            "menu-item-radios",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Radio);
        }
    }

    #[test]
    fn test_selector_aria_content_role_aliases_match_compiled_paragraph() {
        let mut som = make_test_som();
        som.regions[1].elements[0].role = ElementRole::Paragraph;

        for selector in ["alert", "status", "tooltip"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Paragraph);
        }
    }

    #[test]
    fn test_selector_blockquote_alias_matches_compiled_paragraph() {
        let som = make_test_som();

        let filtered = apply_selector(&som, "blockquote");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Paragraph);
    }

    #[test]
    fn test_selector_nav() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "nav");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].role, RegionRole::Navigation);
    }

    #[test]
    fn test_selector_html_id() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "#intro");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(
            filtered.regions[0].elements[0].html_id.as_deref(),
            Some("intro")
        );
    }

    #[test]
    fn test_selector_html_id_preserves_case() {
        let mut som = make_test_som();
        som.regions[1].elements[0].html_id = Some("IntroPanel".to_string());

        let filtered = apply_selector(&som, "#IntroPanel");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].html_id.as_deref(), Some("IntroPanel"));
        assert_eq!(apply_selector(&som, "#intropanel").regions.len(), 2);
    }

    #[test]
    fn test_selector_html_id_preserves_matched_element_children() {
        let mut som = make_test_som();
        som.regions[1].elements[0].html_id = Some("panel".to_string());
        som.regions[1].elements[0].children = Some(vec![Element {
            id: "panel-label".to_string(),
            role: ElementRole::Paragraph,
            html_id: None,
            text: Some("Panel content".to_string()),
            label: None,
            actions: None,
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        }]);

        let filtered = apply_selector(&som, "#panel");
        let children = filtered.regions[0].elements[0]
            .children
            .as_ref()
            .expect("matched element should retain authored children");
        assert_eq!(children[0].text.as_deref(), Some("Panel content"));
    }

    #[test]
    fn test_selector_element_role() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "button");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Button);
        assert_eq!(filtered.meta.element_count, 1);
        assert_eq!(filtered.meta.interactive_count, 1);
        assert_eq!(
            filtered.meta.som_bytes,
            serde_json::to_string(&filtered).unwrap().len()
        );
    }

    #[test]
    fn test_selector_common_html_tag_aliases() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::Link;
        som.regions[1].elements[0].role = ElementRole::Paragraph;
        som.regions[1].elements.push(Element {
            id: "e-list".to_string(),
            role: ElementRole::List,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        });

        for (selector, expected_role) in [
            ("a", ElementRole::Link),
            ("p", ElementRole::Paragraph),
            ("ul", ElementRole::List),
            ("ol", ElementRole::List),
            ("dl", ElementRole::List),
            ("menu", ElementRole::List),
            ("unordered_list", ElementRole::List),
            ("ordered_list", ElementRole::List),
            ("definition_list", ElementRole::List),
            ("definition_lists", ElementRole::List),
            ("definition-list", ElementRole::List),
            ("definition-lists", ElementRole::List),
            ("definition list", ElementRole::List),
            ("definition lists", ElementRole::List),
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert!(filtered.regions[0]
                .elements
                .iter()
                .all(|element| element.role == expected_role));
        }
    }

    #[test]
    fn test_selector_aria_feed_aliases_accept_role_forms() {
        let mut som = make_test_som();
        som.regions[0].elements[0].attrs = Some(serde_json::json!({"source_role": "feed"}));

        for selector in [
            "feed",
            "feeds",
            "aria-feed",
            "aria-feeds",
            "aria_feed",
            "ariafeed",
            "ariafeeds",
            "aria_feeds",
            "aria feed",
            "aria feeds",
            "role=feed",
            "role:feeds",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(
                filtered.regions[0].elements[0]
                    .attrs
                    .as_ref()
                    .and_then(|attrs| attrs.get("source_role"))
                    .and_then(serde_json::Value::as_str),
                Some("feed")
            );
        }
    }

    #[test]
    fn test_selector_feed_does_not_match_plain_lists() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "feed");
        assert_eq!(filtered.regions.len(), som.regions.len());
        assert_eq!(filtered.regions[0].elements.len(), som.regions[0].elements.len());
    }

    #[test]
    fn test_selector_feed_item_aliases_match_aria_articles() {
        let mut som = make_test_som();
        som.regions[0].elements[0].attrs = Some(serde_json::json!({"source_role": "article"}));

        for selector in [
            "feed_item",
            "feed-items",
            "feed items",
            "feeditem",
            "aria-feed-item",
            "aria_feed_items",
            "aria feed item",
            "ariafeeditems",
            "feed_entry",
            "feed-entries",
            "feed entries",
            "feedentry",
            "aria-feed-entry",
            "aria_feed_entries",
            "aria feed entries",
            "ariafeedentries",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(
                filtered.regions[0].elements[0]
                    .attrs
                    .as_ref()
                    .and_then(|attrs| attrs.get("source_role"))
                    .and_then(serde_json::Value::as_str),
                Some("article")
            );
        }
    }

    #[test]
    fn test_selector_definition_list_source_role_aliases() {
        let mut som = make_test_som();
        som.regions[0].elements[0].attrs = Some(serde_json::json!({"source_role": "term"}));
        som.regions[0].elements.push(Element {
            id: "e-definition".to_string(),
            role: ElementRole::Paragraph,
            html_id: None,
            text: Some("Meaning".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"source_role": "definition"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for (selector, source_role) in [
            ("term", "term"),
            ("terms", "term"),
            ("dt", "term"),
            ("definition-term", "term"),
            ("definition-terms", "term"),
            ("definition term", "term"),
            ("definition terms", "term"),
            ("definition_term", "term"),
            ("definition_terms", "term"),
            ("definition", "definition"),
            ("definitions", "definition"),
            ("dd", "definition"),
            ("description", "definition"),
            ("descriptions", "definition"),
            ("definition-description", "definition"),
            ("definition-descriptions", "definition"),
            ("definition description", "definition"),
            ("definition descriptions", "definition"),
            ("definition_description", "definition"),
            ("definition_descriptions", "definition"),
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(
                filtered.regions[0].elements[0]
                    .attrs
                    .as_ref()
                    .and_then(|attrs| attrs.get("source_role"))
                    .and_then(serde_json::Value::as_str),
                Some(source_role),
                "selector: {selector}"
            );
        }
    }

    #[test]
    fn test_selector_aria_group_aliases() {
        for selector in [
            "figure",
            "radiogroup",
            "radio_group",
            "radio-group",
            "radio group",
            "radio groups",
            "progress",
            "progressbar",
            "progress_bar",
            "progress-bar",
            "progress bar",
            "progress_bars",
            "progress-bars",
            "progress bars",
            "meter",
            "audio",
            "video",
            "canvas",
        ] {
            assert_eq!(
                parse_element_role(selector),
                Some(ElementRole::Group),
                "selector: {selector}"
            );
        }
    }

    #[test]
    fn test_selector_output_matches_only_native_output_elements() {
        let mut som = make_test_som();
        som.regions[1].elements[0].role = ElementRole::Paragraph;
        som.regions[1].elements[0].attrs = Some(serde_json::json!({"source_tag": "output"}));
        som.regions[1].elements.push(Element {
            id: "e-paragraph".to_string(),
            role: ElementRole::Paragraph,
            html_id: None,
            text: Some("A paragraph".to_string()),
            label: None,
            actions: None,
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in ["output", "outputs", "role=output"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e2", "selector: {selector}");
        }
    }

    #[test]
    fn test_selector_progress_plural_aliases_preserve_source_specific_matching() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::Group;
        som.regions[0].elements[0].attrs = Some(serde_json::json!({
            "source_role": "progressbar"
        }));
        som.regions[1].elements[0].role = ElementRole::Group;

        for selector in ["progress_bars", "progress-bars", "progress bars"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(
                filtered.regions[0].elements[0]
                    .attrs
                    .as_ref()
                    .and_then(|attrs| attrs.get("source_role"))
                    .and_then(serde_json::Value::as_str),
                Some("progressbar"),
                "selector: {selector}"
            );
        }
    }

    #[test]
    fn test_selector_structural_html_tag_aliases() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::Group;
        som.regions[1].elements[0].role = ElementRole::Separator;
        som.regions[1].elements.push(Element {
            id: "e-details".to_string(),
            role: ElementRole::Details,
            html_id: None,
            text: None,
            label: None,
            actions: None,
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        });

        for (selector, expected_role) in [
            ("div", ElementRole::Group),
            ("fieldset", ElementRole::Group),
            ("hr", ElementRole::Separator),
            ("summary", ElementRole::Details),
            ("summaries", ElementRole::Details),
        ] {
            assert_eq!(
                parse_element_role(selector),
                Some(expected_role.clone()),
                "selector: {selector}"
            );
            let filtered = apply_selector(&som, selector);
            assert!(
                filtered
                    .regions
                    .iter()
                    .flat_map(|region| region.elements.iter())
                    .any(|element| element.role == expected_role),
                "selector should filter matching SOM elements: {selector}"
            );
        }
    }

    #[test]
    fn test_selector_data_table_aliases_match_tables() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::Table;

        for selector in [
            "data_table",
            "data-table",
            "data_tables",
            "data-tables",
            "data table",
            "data tables",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Table);
        }
    }

    #[test]
    fn test_selector_time_input_aliases_match_time_inputs() {
        let mut som = make_test_som();
        som.regions[0].elements[0].role = ElementRole::TextInput;
        som.regions[0].elements[0].attrs = Some(serde_json::json!({"input_type": "time"}));

        for selector in [
            "times",
            "time_input",
            "time-input",
            "time input",
            "timeinput",
            "time_picker",
            "time-picker",
            "time picker",
            "timecontrol",
            "time-control",
            "time control",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::TextInput);
        }
    }

    #[test]
    fn test_selector_aria_landmark_aliases() {
        let mut som = make_test_som();
        som.regions[0].role = RegionRole::Header;
        som.regions[1].role = RegionRole::Footer;

        let mut content = som.regions[0].clone();
        content.id = "r-content".to_string();
        content.role = RegionRole::Content;
        som.regions.push(content);

        let aside = super::super::types::Region {
            id: "r-aside".to_string(),
            role: RegionRole::Aside,
            label: None,
            action: None,
            method: None,
            target: None,
            enctype: None,
            novalidate: None,
            accept_charset: None,
            autocomplete: None,
            elements: vec![],
        };
        som.regions.push(aside);

        let mut search = som.regions[0].clone();
        search.id = "r-search".to_string();
        search.role = RegionRole::Navigation;
        som.regions.push(search);

        let mut form = som.regions[0].clone();
        form.id = "r-form".to_string();
        form.role = RegionRole::Form;
        som.regions.push(form);

        let mut dialog = som.regions[0].clone();
        dialog.id = "r-dialog".to_string();
        dialog.role = RegionRole::Dialog;
        som.regions.push(dialog);

        for (selector, expected_role) in [
            ("banner", RegionRole::Header),
            ("contentinfo", RegionRole::Footer),
            ("content-info", RegionRole::Footer),
            ("complementary", RegionRole::Aside),
            ("search", RegionRole::Navigation),
            ("forms", RegionRole::Form),
            ("dialogs", RegionRole::Dialog),
            ("modal", RegionRole::Dialog),
            ("modals", RegionRole::Dialog),
            ("popup", RegionRole::Dialog),
            ("popups", RegionRole::Dialog),
            ("pop-up", RegionRole::Dialog),
            ("pop-ups", RegionRole::Dialog),
            ("alertdialog", RegionRole::Dialog),
            ("alertdialogs", RegionRole::Dialog),
            ("alert_dialog", RegionRole::Dialog),
            ("alert_dialogs", RegionRole::Dialog),
            ("alert-dialog", RegionRole::Dialog),
            ("alert-dialogs", RegionRole::Dialog),
            ("alert dialog", RegionRole::Dialog),
            ("alert dialogs", RegionRole::Dialog),
            ("region", RegionRole::Content),
            ("role=region", RegionRole::Content),
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].role, expected_role);
        }
    }

    #[test]
    fn test_selector_element_role_preserves_matched_subtree() {
        let mut som = make_test_som();
        som.regions[1].elements[1].children = Some(vec![Element {
            id: "e3-label".to_string(),
            role: ElementRole::Paragraph,
            html_id: None,
            text: Some("Save this report".to_string()),
            label: None,
            actions: None,
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        }]);

        let filtered = apply_selector(&som, "button");
        let button = &filtered.regions[0].elements[0];
        assert_eq!(button.id, "e3");
        assert_eq!(
            button
                .children
                .as_ref()
                .map(|children| children[0].id.as_str()),
            Some("e3-label")
        );
        assert_eq!(filtered.meta.element_count, 2);
    }

    #[test]
    fn test_selector_searchbox_matches_compiled_text_input() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-search".to_string(),
            role: ElementRole::TextInput,
            html_id: None,
            text: None,
            label: Some("Search".to_string()),
            actions: Some(vec!["type".to_string(), "clear".to_string()]),
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        });

        let filtered = apply_selector(&som, "searchbox");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-search");
        assert_eq!(filtered.regions[0].elements[0].role, ElementRole::TextInput);
        assert_eq!(
            apply_selector(&som, "textbox").regions[0].elements[0].id,
            "e-search"
        );
    }

    #[test]
    fn test_selector_plural_text_control_aliases_match_text_input() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-search".to_string(),
            role: ElementRole::TextInput,
            html_id: None,
            text: None,
            label: Some("Search".to_string()),
            actions: None,
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "textboxes",
            "textfields",
            "textinput",
            "textinputs",
            "searchboxes",
            "queryboxes",
            "search_inputs",
            "search_controls",
            "search_fields",
            "search_boxes",
            "query_inputs",
            "query_fields",
            "query_controls",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-search");
        }
    }

    #[test]
    fn test_selector_media_groups_visual_and_media_capable_elements() {
        let mut som = make_test_som();
        som.regions[1].elements.extend([
            Element {
                id: "e-image".to_string(),
                role: ElementRole::Image,
                html_id: None,
                text: None,
                label: Some("Cover".to_string()),
                actions: None,
                attrs: None,
                children: None,
                hints: None,
                shadow: None,
            },
            Element {
                id: "e-video".to_string(),
                role: ElementRole::Group,
                html_id: None,
                text: None,
                label: Some("Preview".to_string()),
                actions: None,
                attrs: Some(serde_json::json!({"source_role": "video"})),
                children: None,
                hints: None,
                shadow: None,
            },
            Element {
                id: "e-audio".to_string(),
                role: ElementRole::Group,
                html_id: None,
                text: None,
                label: Some("Podcast".to_string()),
                actions: None,
                attrs: Some(serde_json::json!({"source_role": "audio"})),
                children: None,
                hints: None,
                shadow: None,
            },
            Element {
                id: "e-fieldset".to_string(),
                role: ElementRole::Group,
                html_id: None,
                text: None,
                label: Some("Filters".to_string()),
                actions: None,
                attrs: Some(serde_json::json!({"source_role": "fieldset"})),
                children: None,
                hints: None,
                shadow: None,
            },
        ]);

        let filtered = apply_selector(&som, "media");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 3);
        assert_eq!(
            filtered.regions[0]
                .elements
                .iter()
                .map(|element| element.id.as_str())
                .collect::<Vec<_>>(),
            vec!["e-image", "e-video", "e-audio"]
        );
    }

    #[test]
    fn test_selector_aria_text_input_aliases_match_compiled_text_input() {
        let mut som = make_test_som();
        for id in ["e-slider", "e-spinbutton"] {
            som.regions[1].elements.push(Element {
                id: id.to_string(),
                role: ElementRole::TextInput,
                html_id: None,
                text: None,
                label: None,
                actions: Some(vec!["type".to_string()]),
                attrs: None,
                children: None,
                shadow: None,
                hints: None,
            });
        }

        for (selector, expected_id) in [
            ("slider", "e-slider"),
            ("sliders", "e-slider"),
            ("spinbutton", "e-spinbutton"),
            ("spinbuttons", "e-spinbutton"),
            ("spin button", "e-spinbutton"),
            ("spin buttons", "e-spinbutton"),
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 2, "selector: {selector}");
            assert!(filtered.regions[0]
                .elements
                .iter()
                .any(|element| element.id == expected_id));
            assert!(filtered.regions[0]
                .elements
                .iter()
                .all(|element| element.role == ElementRole::TextInput));
        }
    }

    #[test]
    fn test_selector_hyphenated_menu_item_control_aliases() {
        let mut som = make_test_som();
        som.regions[1].elements.extend([
            Element {
                id: "e-menu-checkbox".to_string(),
                role: ElementRole::Checkbox,
                html_id: None,
                text: None,
                label: Some("Enable alerts".to_string()),
                actions: None,
                attrs: None,
                children: None,
                shadow: None,
                hints: None,
            },
            Element {
                id: "e-menu-radio".to_string(),
                role: ElementRole::Radio,
                html_id: None,
                text: None,
                label: Some("Daily".to_string()),
                actions: None,
                attrs: None,
                children: None,
                shadow: None,
                hints: None,
            },
        ]);

        for (selector, expected_id) in [
            ("menuitem-checkbox", "e-menu-checkbox"),
            ("menuitem-checkboxes", "e-menu-checkbox"),
            ("menuitem-radio", "e-menu-radio"),
            ("menuitem-radios", "e-menu-radio"),
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, expected_id);
        }
    }

    #[test]
    fn test_selector_input_type_aliases_match_compiled_text_input() {
        for selector in [
            "text",
            "search",
            "search_input",
            "search-input",
            "search_control",
            "search-control",
            "search_field",
            "search-field",
            "query_field",
            "query-field",
            "querybox",
            "query-box",
            "email",
            "number",
            "password",
            "tel",
            "url",
            "date",
            "datetime",
            "datetime-local",
            "date_time",
            "date-time",
            "date time",
            "date_time_local",
            "date-time-local",
            "date time local",
            "month",
            "time",
            "week",
            "color",
            "range",
            "hidden",
            "file",
        ] {
            assert_eq!(
                parse_element_role(selector),
                Some(ElementRole::TextInput),
                "selector: {selector}"
            );
        }
    }

    #[test]
    fn test_selector_datetime_natural_aliases_filter_text_input() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-datetime".to_string(),
            role: ElementRole::TextInput,
            html_id: None,
            text: None,
            label: Some("Appointment".to_string()),
            actions: Some(vec!["type".to_string()]),
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "date_time",
            "date-time",
            "date time",
            "date_time_local",
            "date-time-local",
            "date time local",
            "role=date-time",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-datetime");
        }
    }

    #[test]
    fn test_selector_search_input_alias_filters_text_input() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-search".to_string(),
            role: ElementRole::TextInput,
            html_id: None,
            text: None,
            label: Some("Search".to_string()),
            actions: Some(vec!["type".to_string()]),
            attrs: None,
            children: None,
            shadow: None,
            hints: None,
        });

        for selector in [
            "search_input",
            "search-input",
            "search_control",
            "search-control",
            "search_field",
            "search-field",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-search");
        }
    }

    #[test]
    fn test_selector_query_input_alias_filters_text_input() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-query".to_string(),
            role: ElementRole::TextInput,
            html_id: None,
            text: None,
            label: Some("Query".to_string()),
            actions: Some(vec!["type".to_string()]),
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "query",
            "query_input",
            "queries",
            "query-input",
            "query_field",
            "query-field",
            "query_control",
            "query-control",
            "querybox",
            "query-box",
            "role=query_input",
            "role:query-input",
            "role=query_field",
            "role:query-field",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1);
            assert_eq!(filtered.regions[0].elements[0].id, "e-query");
        }
    }

    #[test]
    fn test_selector_combobox_matches_compiled_select() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-combo".to_string(),
            role: ElementRole::Select,
            html_id: None,
            text: None,
            label: Some("Country".to_string()),
            actions: Some(vec!["select".to_string()]),
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        });

        let filtered = apply_selector(&som, "combobox");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-combo");
        assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Select);
        assert_eq!(
            apply_selector(&som, "listbox").regions[0].elements[0].id,
            "e-combo"
        );
        assert_eq!(
            apply_selector(&som, "select").regions[0].elements[0].id,
            "e-combo"
        );
    }

    #[test]
    fn test_selector_label_matches_hyphenated_nested_aria_description() {
        let mut som = make_test_som();
        som.regions[1].elements[0].attrs = Some(serde_json::json!({
            "aria": { "aria-description": "Choose a country" }
        }));

        let filtered = apply_selector(&som, "label:country");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e2");
    }

    #[test]
    fn test_selector_label_matches_camel_case_aria_label_attribute() {
        let mut som = make_test_som();
        som.regions[1].elements[0].attrs = Some(serde_json::json!({
            "ariaLabel": "Choose a country"
        }));

        let filtered = apply_selector(&som, "label:country");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e2");
    }

    #[test]
    fn test_selector_label_matches_camel_case_nested_aria_description() {
        let mut som = make_test_som();
        som.regions[1].elements[0].attrs = Some(serde_json::json!({
            "aria": { "ariaDescription": "Choose a country" }
        }));

        let filtered = apply_selector(&som, "label:country");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e2");
    }

    #[test]
    fn test_selector_placeholder_matches_placeholder_attribute() {
        let mut som = make_test_som();
        som.regions[1].elements[0].attrs = Some(serde_json::json!({
            "placeholder": "Choose a country"
        }));

        for selector in ["placeholder:country", "placeholder=country"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e2", "selector: {selector}");
        }
    }

    #[test]
    fn test_selector_label_matches_underscore_nested_aria_description() {
        let mut som = make_test_som();
        som.regions[1].elements[0].attrs = Some(serde_json::json!({
            "aria": { "aria_description": "Choose a country" }
        }));

        let filtered = apply_selector(&som, "label:country");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e2");
    }

    #[test]
    fn test_selector_interactive_elements() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "interactive");
        assert_eq!(filtered.regions.len(), 2);
        assert!(filtered
            .regions
            .iter()
            .flat_map(|region| region.elements.iter())
            .all(|element| element.role.is_interactive()));
    }

    #[test]
    fn test_selector_action() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "action:click");
        assert_eq!(filtered.regions.len(), 2);
        assert!(filtered
            .regions
            .iter()
            .flat_map(|region| region.elements.iter())
            .all(|element| element
                .actions
                .as_ref()
                .is_some_and(|actions| actions.contains(&"click".to_string()))));
    }

    #[test]
    fn test_selector_bare_action_alias() {
        let som = make_test_som();
        let filtered = apply_selector(&som, " click ");

        assert_eq!(filtered.regions.len(), 2);
        assert!(filtered
            .regions
            .iter()
            .flat_map(|region| &region.elements)
            .all(|element| element
                .actions
                .as_ref()
                .is_some_and(|actions| actions.iter().any(|action| action == "click"))));
    }

    #[test]
    fn test_selector_mcp_action_aliases() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-input".to_string(),
            role: ElementRole::TextInput,
            html_id: None,
            text: None,
            label: Some("Query".to_string()),
            actions: Some(vec!["type".to_string()]),
            attrs: None,
            children: None,
            shadow: None,
            hints: None,
        });
        som.regions[1].elements.push(Element {
            id: "e-select".to_string(),
            role: ElementRole::Select,
            html_id: None,
            text: None,
            label: Some("Sort".to_string()),
            actions: Some(vec!["select".to_string()]),
            attrs: None,
            children: None,
            shadow: None,
            hints: None,
        });

        assert_eq!(
            apply_selector(&som, "type_text").regions[0].elements[0].id,
            "e-input"
        );
        assert_eq!(
            apply_selector(&som, "action:type_text").regions[0].elements[0].id,
            "e-input"
        );
        assert_eq!(
            apply_selector(&som, "type-text").regions[0].elements[0].id,
            "e-input"
        );
        assert_eq!(
            apply_selector(&som, "action:type-text").regions[0].elements[0].id,
            "e-input"
        );
        assert_eq!(
            apply_selector(&som, "select_option").regions[0].elements[0].id,
            "e-select"
        );
        assert_eq!(
            apply_selector(&som, "action:select_option").regions[0].elements[0].id,
            "e-select"
        );
        assert_eq!(
            apply_selector(&som, "select-option").regions[0].elements[0].id,
            "e-select"
        );
        assert_eq!(
            apply_selector(&som, "action:select-option").regions[0].elements[0].id,
            "e-select"
        );
    }

    #[test]
    fn test_selector_clear_action_aliases() {
        let mut som = make_test_som();
        som.regions[1].elements[0].actions = Some(vec!["clear".to_string()]);

        for selector in [
            "clear_text",
            "clear-text",
            "clear text",
            "action:clear_text",
            "action:clear-text",
            "action : clear text",
            "action = clear_text",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e2", "selector: {selector}");
        }
    }

    #[test]
    fn test_selector_spaced_mcp_action_aliases() {
        let mut som = make_test_som();
        som.regions[1].elements[0].actions = Some(vec!["type".to_string()]);
        som.regions[1].elements.push(Element {
            id: "e-select".to_string(),
            role: ElementRole::Select,
            html_id: None,
            text: None,
            label: Some("Sort".to_string()),
            actions: Some(vec!["select".to_string()]),
            attrs: None,
            children: None,
            shadow: None,
            hints: None,
        });

        for selector in ["type text", "action:type text", "action : type text"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e2", "selector: {selector}");
        }
        for selector in ["select option", "action:select option", "action : select option"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-select", "selector: {selector}");
        }
    }

    #[test]
    fn test_selector_action_accepts_whitespace_around_separator() {
        let som = make_test_som();

        for selector in [
            "action : click",
            "action = click",
            "action click",
            " ACTION\tCLICK ",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 2, "selector: {selector}");
            assert!(filtered
                .regions
                .iter()
                .flat_map(|region| &region.elements)
                .all(|element| element
                    .actions
                    .as_ref()
                    .is_some_and(|actions| actions.iter().any(|action| action == "click"))));
        }
    }

    #[test]
    fn test_selector_action_whitespace_preserves_aliases() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-input".to_string(),
            role: ElementRole::TextInput,
            html_id: None,
            text: None,
            label: Some("Query".to_string()),
            actions: Some(vec!["type".to_string()]),
            attrs: None,
            children: None,
            shadow: None,
            hints: None,
        });

        for selector in ["action : type_text", "action = type-text"] {
            assert_eq!(
                apply_selector(&som, selector).regions[0].elements[0].id,
                "e-input",
                "selector: {selector}"
            );
        }
    }

    #[test]
    fn test_selector_submit_action_matches_compiled_submit_controls() {
        let mut som = make_test_som();
        som.regions[1].elements[1].attrs = Some(serde_json::json!({"button_type": "submit"}));

        let filtered = apply_selector(&som, "action:submit");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(
            filtered.regions[0].elements[0].html_id.as_deref(),
            Some("save")
        );
    }

    #[test]
    fn test_selector_reset_action_matches_compiled_reset_controls() {
        let mut som = make_test_som();
        som.regions[1].elements[1].attrs = Some(serde_json::json!({"button_type": "reset"}));

        let filtered = apply_selector(&som, "reset");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(
            filtered.regions[0].elements[0].attrs.as_ref().unwrap()["button_type"],
            "reset"
        );
    }

    #[test]
    fn test_selector_submit_action_keeps_native_submit_with_overridden_role() {
        let mut som = make_test_som();
        som.regions[1].elements[1].role = ElementRole::Link;
        som.regions[1].elements[1].attrs = Some(serde_json::json!({"button_type": "submit"}));

        let filtered = apply_selector(&som, "action:submit");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Link);
    }

    #[test]
    fn test_selector_region_id() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "#r2");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].id, "r2");
    }

    #[test]
    fn test_selector_som_element_id() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "#e3");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e3");
        assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Button);
    }

    #[test]
    fn test_selector_som_element_id_without_html_id() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "#e1");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e1");
        assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Link);
    }

    #[test]
    fn test_selector_explicit_id_forms_match_regions_and_elements() {
        let som = make_test_som();
        for selector in ["id=r2", "id:r2", "id='r2'"] {
            assert_eq!(apply_selector(&som, selector).regions[0].id, "r2");
        }
        for selector in ["id=e3", "id:e3", "id=\"e3\""] {
            assert_eq!(apply_selector(&som, selector).regions[0].elements[0].id, "e3");
        }
    }

    #[test]
    fn test_selector_id_preserves_case() {
        let mut som = make_test_som();
        som.regions[1].id = "CheckoutRegion".to_string();
        som.regions[1].elements[1].html_id = Some("SaveButton".to_string());

        for selector in [
            "#CheckoutRegion",
            "id=CheckoutRegion",
            "selector=#CheckoutRegion",
            "selector: '#CheckoutRegion'",
        ] {
            assert_eq!(apply_selector(&som, selector).regions[0].id, "CheckoutRegion");
        }
        for selector in [
            "#SaveButton",
            "id:SaveButton",
            "selector = id=SaveButton",
            "selector: 'id:SaveButton'",
        ] {
            assert_eq!(
                apply_selector(&som, selector).regions[0].elements[0].html_id.as_deref(),
                Some("SaveButton")
            );
        }
    }

    #[test]
    fn test_selector_nested_som_element_id() {
        let mut som = make_test_som();
        som.regions[0].elements[0].children = Some(vec![Element {
            id: "e-child".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: Some("Act".to_string()),
            label: None,
            actions: None,
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        }]);

        let filtered = apply_selector(&som, "#e-child");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        let children = filtered.regions[0].elements[0].children.as_ref().unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, "e-child");
    }

    #[test]
    fn test_selector_trims_whitespace() {
        let som = make_test_som();
        let filtered = apply_selector(&som, " main ");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].role, RegionRole::Main);
    }

    #[test]
    fn test_selector_accepts_named_selector_wrappers() {
        let som = make_test_som();
        for selector in [
            "selector=main",
            "selector: main",
            "selector main",
            "SELECTOR = 'main'",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].role, RegionRole::Main);
        }
    }

    #[test]
    fn test_selector_landmarks_returns_region_inventory_without_elements() {
        let som = make_test_som();
        for selector in [
            " LANDMARKS ",
            "landmark_region",
            "role=landmarks",
            "role:landmarks",
            "role=landmark_region",
            "role:landmark_region",
            "role=landmark-region",
            "role:landmark-regions",
            "role:landmark regions",
            "role:landmark_regions",
            "landmark-region",
            "role:landmark region",
            "landmark-inventory",
            "landmark inventories",
            "landmark_region_inventory",
            "landmark-region-inventories",
            "landmark region inventory",
            "landmark region inventories",
            "role:landmark inventory",
        ] {
            let filtered = apply_selector(&som, selector);

            assert_eq!(filtered.regions.len(), som.regions.len(), "selector: {selector}");
            assert!(
                filtered.regions.iter().all(|region| region.elements.is_empty()),
                "selector: {selector}"
            );
            assert_eq!(filtered.meta.element_count, 0, "selector: {selector}");
            assert_eq!(filtered.meta.interactive_count, 0, "selector: {selector}");
        }
        assert!(som.regions.iter().any(|region| !region.elements.is_empty()));
    }

    #[test]
    fn test_selector_query_control_aliases_match_text_inputs() {
        let mut som = make_test_som();
        som.regions[1].elements[0].role = ElementRole::TextInput;

        for selector in ["query_control", "query-control"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].role, ElementRole::TextInput);
        }
    }

    #[test]
    fn test_selector_heading_level_uses_compiled_level() {
        let mut som = make_test_som();
        som.regions[1].elements.extend([
            Element {
                id: "e-h1".to_string(),
                role: ElementRole::Heading,
                html_id: None,
                text: Some("Title".to_string()),
                label: None,
                actions: None,
                attrs: Some(serde_json::json!({"level": 1})),
                children: None,
                hints: None,
                shadow: None,
            },
            Element {
                id: "e-h2".to_string(),
                role: ElementRole::Heading,
                html_id: None,
                text: Some("Section".to_string()),
                label: None,
                actions: None,
                attrs: Some(serde_json::json!({"level": 2})),
                children: None,
                hints: None,
                shadow: None,
            },
            Element {
                id: "e-h-missing".to_string(),
                role: ElementRole::Heading,
                html_id: None,
                text: Some("No level".to_string()),
                label: None,
                actions: None,
                attrs: None,
                children: None,
                hints: None,
                shadow: None,
            },
            Element {
                id: "e-fake-level".to_string(),
                role: ElementRole::Paragraph,
                html_id: None,
                text: Some("Not a heading".to_string()),
                label: None,
                actions: None,
                attrs: Some(serde_json::json!({"level": 1})),
                children: None,
                hints: None,
                shadow: None,
            },
            Element {
                id: "e-aria-level".to_string(),
                role: ElementRole::Button,
                html_id: None,
                text: Some("Billing".to_string()),
                label: None,
                actions: Some(vec!["click".to_string()]),
                attrs: Some(serde_json::json!({"aria": {"level": "1"}})),
                children: None,
                hints: None,
                shadow: None,
            },
        ]);

        let h1 = apply_selector(&som, "h1");
        let h1_ids: Vec<_> = h1
            .regions
            .iter()
            .flat_map(|region| region.elements.iter())
            .map(|element| element.id.as_str())
            .collect();
        assert_eq!(h1.regions.len(), 1, "h1 must not return full SOM: {h1:?}");
        assert_eq!(
            h1_ids,
            vec!["e-h1"],
            "h1 must keep only compiled level 1: {h1:?}"
        );

        let h2 = apply_selector(&som, " H2 ");
        let h2_ids: Vec<_> = h2
            .regions
            .iter()
            .flat_map(|region| region.elements.iter())
            .map(|element| element.id.as_str())
            .collect();
        assert_eq!(
            h2_ids,
            vec!["e-h2"],
            "h2 must keep only compiled level 2: {h2:?}"
        );

        let headings = apply_selector(&som, "heading");
        let heading_ids: Vec<_> = headings
            .regions
            .iter()
            .flat_map(|region| region.elements.iter())
            .map(|element| element.id.as_str())
            .collect();
        assert_eq!(
            heading_ids,
            vec!["e-h1", "e-h2", "e-h-missing"],
            "heading must still match every heading role"
        );

        let unrecognized = apply_selector(&som, "h7");
        assert_eq!(
            unrecognized.regions.len(),
            som.regions.len(),
            "h7 must stay unrecognized and return full SOM"
        );
    }

    #[test]
    fn test_selector_article_alias_matches_content_region() {
        let mut som = make_test_som();
        som.regions[1].role = RegionRole::Content;
        for selector in [
            "article",
            "article_region",
            "article-region",
            "article region",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].role, RegionRole::Content);
        }
    }

    #[test]
    fn test_selector_compact_primary_content_aliases_match_main_region() {
        let som = make_test_som();

        for selector in ["primarycontent", "primarycontents"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].role, RegionRole::Main);
        }
    }

    #[test]
    fn test_selector_article_role_falls_back_to_section_element() {
        let mut som = make_test_som();
        som.regions[1].elements[0].role = ElementRole::Section;

        for selector in ["article", "role=article"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(
                filtered.regions[0].elements[0].role,
                ElementRole::Section,
                "selector: {selector}"
            );
        }
    }

    #[test]
    fn test_selector_region_role_falls_back_to_section_element() {
        let mut som = make_test_som();
        som.regions[1].elements[0].role = ElementRole::Section;

        for selector in [
            "content",
            "contents",
            "region",
            "regions",
            "content-region",
            "content regions",
            "role=content",
            "role=region",
            "role=content-region",
            "role:content region",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(
                filtered.regions[0].elements[0].role,
                ElementRole::Section,
                "selector: {selector}"
            );
        }
    }

    #[test]
    fn test_selector_grid_aliases_match_table_elements() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-grid".to_string(),
            role: ElementRole::Table,
            html_id: None,
            text: None,
            label: Some("Results".to_string()),
            actions: None,
            attrs: Some(serde_json::json!({"source_role": "grid"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [
            "grid",
            "grids",
            "data-grid",
            "data grids",
            "tree-grid",
            "tree grids",
            "treegrid",
            "treegrids",
            "table-grid",
            "table grids",
            "table-grids",
            "table_grid",
            "role=grid",
            "role: data-grid",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-grid");
        }
    }

    #[test]
    fn test_selector_specific_media_aliases_exclude_other_groups() {
        let mut som = make_test_som();
        som.regions[1].elements.extend([
            Element {
                id: "e-video".to_string(),
                role: ElementRole::Group,
                html_id: None,
                text: None,
                label: None,
                actions: None,
                attrs: Some(serde_json::json!({"source_role": "video"})),
                children: None,
                hints: None,
                shadow: None,
            },
            Element {
                id: "e-fieldset".to_string(),
                role: ElementRole::Group,
                html_id: None,
                text: None,
                label: None,
                actions: None,
                attrs: Some(serde_json::json!({"source_role": "fieldset"})),
                children: None,
                hints: None,
                shadow: None,
            },
        ]);

        for selector in ["video", "videos", "video-item", "role=video"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-video");
        }
    }

    #[test]
    fn test_selector_progress_controls_exclude_unrelated_groups() {
        let mut som = make_test_som();
        som.regions[1].elements.extend([
            Element {
                id: "e-progress".to_string(),
                role: ElementRole::Group,
                html_id: None,
                text: None,
                label: None,
                actions: None,
                attrs: Some(serde_json::json!({"source_role": "progressbar"})),
                children: None,
                hints: None,
                shadow: None,
            },
            Element {
                id: "e-meter".to_string(),
                role: ElementRole::Group,
                html_id: None,
                text: None,
                label: None,
                actions: None,
                attrs: Some(serde_json::json!({"source_role": "meter"})),
                children: None,
                hints: None,
                shadow: None,
            },
            Element {
                id: "e-group".to_string(),
                role: ElementRole::Group,
                html_id: None,
                text: None,
                label: None,
                actions: None,
                attrs: None,
                children: None,
                hints: None,
                shadow: None,
            },
        ]);

        for (selector, expected_id) in [
            ("progress", "e-progress"),
            ("progressbar", "e-progress"),
            ("progress_bar", "e-progress"),
            ("progress-bar", "e-progress"),
            ("progress bar", "e-progress"),
            ("meter", "e-meter"),
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, expected_id);
        }
    }

    #[test]
    fn test_selector_nested_html_id() {
        let mut som = make_test_som();
        som.regions[0].elements[0].children = Some(vec![Element {
            id: "e-child".to_string(),
            role: ElementRole::Button,
            html_id: Some("nested-action".to_string()),
            text: Some("Act".to_string()),
            label: None,
            actions: None,
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        }]);

        let filtered = apply_selector(&som, "#nested-action");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        let children = filtered.regions[0].elements[0].children.as_ref().unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].html_id.as_deref(), Some("nested-action"));
    }

    #[test]
    fn test_selector_matching_element_filters_unmatched_shadow_content() {
        let mut som = make_test_som();
        som.regions[1].elements[1].shadow = Some(ShadowRoot {
            mode: "open".to_string(),
            elements: vec![Element {
                id: "shadow-text".to_string(),
                role: ElementRole::Paragraph,
                html_id: None,
                text: Some("Do not leak".to_string()),
                label: None,
                actions: None,
                attrs: None,
                children: None,
                hints: None,
                shadow: None,
            }],
        });

        let filtered = apply_selector(&som, "button");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert!(filtered.regions[0].elements[0].shadow.is_none());
    }

    #[test]
    fn test_selector_element_id_filters_unmatched_shadow_content() {
        let mut som = make_test_som();
        som.regions[1].elements[1].shadow = Some(ShadowRoot {
            mode: "open".to_string(),
            elements: vec![Element {
                id: "shadow-text".to_string(),
                role: ElementRole::Paragraph,
                html_id: Some("other-id".to_string()),
                text: Some("Do not leak".to_string()),
                label: None,
                actions: None,
                attrs: None,
                children: None,
                hints: None,
                shadow: None,
            }],
        });

        let filtered = apply_selector(&som, "#e3");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert!(filtered.regions[0].elements[0].shadow.is_none());
    }

    #[test]
    fn test_selector_matches_css_attribute_tokens() {
        let mut som = make_test_som();
        som.regions[1].elements.extend([
            Element {
                id: "e-described-by-help-and-status".to_string(),
                role: ElementRole::Button,
                html_id: None,
                text: Some("Save".to_string()),
                label: None,
                actions: None,
                attrs: Some(serde_json::json!({"aria-describedby": "help status"})),
                children: None,
                hints: None,
                shadow: None,
            },
            Element {
                id: "e-described-by-helper".to_string(),
                role: ElementRole::Button,
                html_id: None,
                text: Some("Cancel".to_string()),
                label: None,
                actions: None,
                attrs: Some(serde_json::json!({"aria-describedby": "helper"})),
                children: None,
                hints: None,
                shadow: None,
            },
        ]);

        let filtered = apply_selector(&som, "[aria-describedby~='help']");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(
            filtered.regions[0].elements[0].id,
            "e-described-by-help-and-status"
        );

        let unquoted = apply_selector(&som, "[aria-describedby~=help]");
        assert_eq!(unquoted.regions.len(), 1);
        assert_eq!(unquoted.regions[0].elements.len(), 1);
        assert_eq!(
            unquoted.regions[0].elements[0].id,
            "e-described-by-help-and-status"
        );
    }

    #[test]
    fn test_selector_matches_css_attribute_tokens_with_operator_whitespace() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-described-by-help".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: Some("Save".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"aria-describedby": "help status"})),
            children: None,
            hints: None,
            shadow: None,
        });

        let filtered = apply_selector(&som, "[aria-describedby ~ = 'help']");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-described-by-help");
    }

    #[test]
    fn test_selector_matches_css_attribute_tokens_with_non_space_whitespace() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-described-by-status".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: Some("Save".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"aria-describedby": "help status"})),
            children: None,
            hints: None,
            shadow: None,
        });

        let filtered = apply_selector(&som, "[aria-describedby\n\t~\t=\t'status']");

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-described-by-status");
    }

    #[test]
    fn test_selector_matches_css_case_flag_without_space() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-case-flag".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: Some("Save".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"data-state": "Open"})),
            children: None,
            hints: None,
            shadow: None,
        });

        let filtered = apply_selector(&som, r#"[data-state="open"i]"#);

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-case-flag");
    }

    #[test]
    fn test_selector_matches_compact_aria_attribute_values() {
        let mut som = make_test_som();
        som.regions[1].elements.extend([
            Element {
                id: "e-current-page".to_string(),
                role: ElementRole::Link,
                html_id: None,
                text: Some("Home".to_string()),
                label: None,
                actions: None,
                attrs: Some(serde_json::json!({"aria-current": "page"})),
                children: None,
                hints: None,
                shadow: None,
            },
            Element {
                id: "e-current-step".to_string(),
                role: ElementRole::Link,
                html_id: None,
                text: Some("Checkout".to_string()),
                label: None,
                actions: None,
                attrs: Some(serde_json::json!({"aria-current": "step"})),
                children: None,
                hints: None,
                shadow: None,
            },
        ]);

        for selector in [
            "aria-current:page",
            "aria-current=page",
            "aria-current ~= page",
        ] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-current-page");
        }
    }

    #[test]
    fn test_selector_matches_forward_compatible_aria_attributes() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-braille-label".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: Some("Save".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"aria-braillelabel": "save action"})),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in ["[aria-braillelabel]", "[aria-braillelabel='save action']"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-braille-label");
        }
    }

    #[test]
    fn test_selector_ignores_operator_like_text_inside_quoted_value() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-data-label".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: Some("Save".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"data-label": "a^=b"})),
            children: None,
            hints: None,
            shadow: None,
        });

        let filtered = apply_selector(&som, r#"[data-label="a^=b"]"#);

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-data-label");
    }

    #[test]
    fn test_selector_matches_escaped_quotes_in_attribute_values() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-escaped-label".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: Some("Save".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({"data-label": "a\"=b"})),
            children: None,
            hints: None,
            shadow: None,
        });

        let filtered = apply_selector(&som, r#"[data-label=\"a\\\"=b\"]"#);

        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e-escaped-label");
    }

    #[test]
    fn test_compact_attribute_selectors_unescape_values() {
        let mut som = make_test_som();
        som.regions[1].elements.push(Element {
            id: "e-compact-escaped-value".to_string(),
            role: ElementRole::Button,
            html_id: None,
            text: Some("Save".to_string()),
            label: None,
            actions: None,
            attrs: Some(serde_json::json!({
                "data-label": "a=b",
                "aria-label": "a=b"
            })),
            children: None,
            hints: None,
            shadow: None,
        });

        for selector in [r#"data-label:a\=b"#, r#"aria-label:a\=b"#] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].id, "e-compact-escaped-value");
        }
    }

    #[test]
    fn test_selector_no_match_returns_full() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "dialog");
        assert_eq!(filtered.regions.len(), 2); // Full SOM returned
    }

    #[test]
    fn test_selector_unrecognised_returns_full() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "foobar");
        assert_eq!(filtered.regions.len(), 2);
    }

    #[test]
    fn test_selector_id_prefix_is_case_insensitive() {
        let som = make_test_som();

        for selector in ["ID=save", "Id:save"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].html_id.as_deref(), Some("save"));
        }
    }

    #[test]
    fn test_selector_id_prefix_accepts_whitespace_separator() {
        let som = make_test_som();

        for selector in ["id save", "ID 'save'"] {
            let filtered = apply_selector(&som, selector);
            assert_eq!(filtered.regions.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements.len(), 1, "selector: {selector}");
            assert_eq!(filtered.regions[0].elements[0].html_id.as_deref(), Some("save"));
        }
    }

    #[test]
    fn test_selector_id_prefix_rejects_escaped_closing_quote() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "id=\"save\\\"");

        assert_eq!(filtered.regions.len(), som.regions.len());
    }
}
