"use strict";

const $ = (id) => document.getElementById(id);
const params = new URLSearchParams(location.search);
const scenarios = {
  entry: ["已保存的独立工况", "Saved independent case"],
  first: ["选择首次关联", "Choose initial association"],
  replace: ["更换冷侧流股", "Replace the cold-side stream"],
  preview: ["50 mol/s 候选预览", "Preview a 50 mol/s candidate"],
  confirm: ["首次覆盖的能力确认", "Confirm the first in-place capability addition"],
  conflict: ["另存目标已变化", "Save-as destination changed"],
  source: ["源工程已变化", "Source project changed"],
  saved: ["保存重开，待运行", "Reopened after save, waiting to run"],
  limit: ["50 mol/s 当前结果", "Current result at 50 mol/s"],
  unsupported: ["未知能力的原文只读", "Read-only review for an unknown capability"],
};
const streamSamples = {
  "hot-feed": { names: ["热水", "Hot water"], source: h3Evidence["40"].sources.hot },
  "cold-feed": { names: ["冷水 40", "Cold water 40"], source: h3Evidence["40"].sources.cold },
  "cold-feed-50": { names: ["冷水 50", "Cold water 50"], source: h3Evidence["50"].sources.cold },
};
const primaryFile = "H3 Association Trial.rfproj.json";
const copyFile = "H3 Association Trial - copy.rfproj.json";
const pair40 = { hot: "hot-feed", cold: "cold-feed" };
const pair50 = { hot: "hot-feed", cold: "cold-feed-50" };
const empty = { hot: "", cold: "" };
const same = (a, b) => a?.hot === b?.hot && a?.cold === b?.cold;
let language = params.get("lang") === "en" ? "en" : "zh";
let state;
let pending = null;

function t(zh, en) {
  return language === "zh" ? zh : en;
}

function localized(values) {
  return t(...values);
}

function option(value, text) {
  return new Option(text, value);
}

function node(tag, className, text) {
  const item = document.createElement(tag);
  if (className) item.className = className;
  if (text) item.textContent = text;
  return item;
}

function status(id, tone, text) {
  $(id).dataset.tone = tone;
  $(id).textContent = text;
}

function displayedNumber(value) {
  return new Intl.NumberFormat(t("zh-CN", "en"), { maximumFractionDigits: 2 }).format(value);
}

function streamLabel(id) {
  return id ? localized(streamSamples[id].names) : t("未选择", "Unselected");
}

function sideCard(side, streamId, showInlet) {
  const label = side === "hot" ? t("热侧", "Hot side") : t("冷侧", "Cold side");
  const sample = streamSamples[streamId];
  const cell = node("div", "pair-cell");
  cell.append(node("span", "small", label), node("strong", "", streamLabel(streamId)));
  if (sample) {
    cell.append(node("span", "small mono", sample.source.stream.stream_id));
    if (showInlet) {
      const inlet = sample.source.inlet;
      const observed = [
        displayedNumber(inlet.temperature_k) + " K",
        displayedNumber(inlet.pressure_pa) + " Pa",
        displayedNumber(inlet.molar_flow_mol_s) + " mol/s",
      ];
      cell.append(node("span", "numeric", observed.join(" · ")));
      const details = node("details");
      details.append(node("summary", "small", t("创建身份", "Creation identity")));
      details.append(node("p", "small mono", sample.source.stream.creation_id));
      cell.append(details);
    }
  }
  return cell;
}

function renderPair(id, pair, showInlet = false) {
  const box = $(id);
  if (pair) {
    box.replaceChildren(sideCard("hot", pair.hot, showInlet), sideCard("cold", pair.cold, showInlet));
  } else {
    box.replaceChildren(node("p", "small", t("使用独立工况的两侧入口", "Both inlets come from the independent case")));
  }
}

function fillOptions() {
  const scene = $("scene").value;
  $("scene").replaceChildren(...Object.entries(scenarios).map(([value, names]) => option(value, localized(names))));
  $("scene").value = scene;
  for (const side of ["hot", "cold"]) {
    const select = $(side);
    select.replaceChildren(option("", t("选择流股", "Select a stream")));
    for (const id of Object.keys(streamSamples)) {
      select.add(option(id, streamLabel(id) + " · " + id));
    }
  }
}

function isComplete() {
  return state.draft.hot && state.draft.cold && state.draft.hot !== state.draft.cold;
}

function isChanged() {
  return !same(state.draft, state.saved || empty);
}

function updateNotice() {
  const notice = $("notice");
  let tone = "caution";
  let text = "";
  if (state.conflict === "target") {
    text = t("另存目标在确认期间发生变化。草稿已保留，请重新选择保存目标。", "The destination changed during confirmation. Your draft is intact. Choose a save destination again.");
  } else if (state.conflict === "source") {
    text = t("源工程在确认期间发生变化。草稿已保留，请先重新载入并核对流股。", "The source project changed during confirmation. Your draft is intact. Reload the source and check its streams.");
  } else if (state.openedAfterSave) {
    tone = "good";
    text = t("已打开保存后的工程。请显式运行以获取当前结果。", "The saved project is open. Run explicitly to get a current result.");
  } else if (state.unsupported) {
    text = t("工程包含未知能力。原文件以只读方式打开。", "This project has an unknown capability and is open for read-only inspection.");
  }
  notice.hidden = !text;
  status("notice", tone, text);
}

function renderReview() {
  renderPair("review-pair", state.saved);
  $("edit").hidden = state.unsupported;
  $("edit").textContent = state.saved ? t("编辑流股关联", "Edit stream association") : t("建立流股关联", "Set up stream association");
  $("rate").hidden = !state.saved || state.unsupported;
  const box = $("result-details");
  if (state.unsupported) {
    status("review-status", "neutral", t("原文只读", "Read-only inspection"));
    box.replaceChildren(node("p", "small", t("此工程需要支持其全部能力的 Studio 版本才能运行。", "Running this project requires a Studio version that supports all of its capabilities.")));
    return;
  }
  if (state.rating === null) {
    status("review-status", "neutral", t("待运行", "Waiting to run"));
    box.replaceChildren(node("p", "small", t("运行后显示热负荷、压降和限值。", "Run to see heat duty, pressure drops and limits.")));
    return;
  }
  const r = h3Evidence[state.rating];
  status("review-status", "current", t("当前结果", "Current result"));
  const table = node("dl", "details");
  addRow(table, t("热负荷", "Heat duty"), displayedNumber(r.heatDuty) + " W");
  addRow(table, "UA", displayedNumber(r.ua) + " W/K");
  addRow(table, t("热侧压降", "Hot-side pressure drop"), displayedNumber(r.hotDrop) + " Pa");
  addRow(table, t("冷侧压降", "Cold-side pressure drop"), displayedNumber(r.coldDrop) + " Pa");
  addRow(table, t("冷侧压降限值", "Cold-side pressure-drop limit"), "25,000 Pa");
  const failed = r.coldLimit === "failed";
  const check = node("p", "pill", failed ? t("冷侧压降超限", "Cold-side pressure-drop limit exceeded") : t("三项限值通过", "All three limits passed"));
  check.dataset.tone = failed ? "caution" : "good";
  box.replaceChildren(check, table);
}

function renderEditor() {
  renderPair("saved-pair", state.saved);
  $("hot").value = state.draft.hot;
  $("cold").value = state.draft.cold;
  $("hint").textContent = state.draft.hot && state.draft.hot === state.draft.cold
    ? t("冷热侧需要选择不同的流股。", "Choose different streams for the hot and cold sides.")
    : t("选择同一工程内的两条流股。", "Choose two streams in this project.");
  const validPreview = state.preview && state.conflict !== "source";
  if (validPreview) {
    renderPair("preview-inlets", state.draft, true);
  }
  $("preview-inlets").hidden = !validPreview;
  status("preview-status", validPreview ? "good" : "neutral",
    validPreview ? t("入口资格通过", "Inlet qualification passed") : t("待预览", "Preview needed"));
  const qualification = h3Evidence["40"].qualification;
  $("preview-evidence").textContent = validPreview
    ? t("已接收稳态快照，使用 ", "Accepted steady-state snapshot using ") + qualification.id + " v" + qualification.revision
    : t("显式运行流程后核对两侧温度、压力和摩尔流量。", "Run the flowsheet to check inlet temperature, pressure and molar flow.");
  $("preview").disabled = !isComplete() || state.conflict === "source";
  $("reload-source").hidden = state.conflict !== "source";
  $("preview").classList.toggle("primary", !validPreview);
  $("save").disabled = !validPreview || !isChanged();
  $("save-as").disabled = !validPreview || !isChanged();
  $("reset").disabled = !isChanged();
}

function render() {
  document.documentElement.lang = t("zh-CN", "en");
  for (const item of document.querySelectorAll("[data-zh]")) {
    item.textContent = item.dataset[language];
  }
  const editing = state.screen === "edit";
  $("project-name").textContent = state.file;
  $("view-title").textContent = editing
    ? t("编辑 H3 流程关联", "Edit H3 flow association")
    : t("H3 换热器复核", "H3 exchanger review");
  $("review-view").hidden = editing;
  $("edit-view").hidden = !editing;
  updateNotice();
  if (editing) renderEditor();
  else renderReview();
}

function setup(scene) {
  const first = scene === "entry" || scene === "first" || scene === "confirm";
  const sent = scene === "saved" || scene === "limit";
  const editing = !["entry", "saved", "limit", "unsupported"].includes(scene);
  state = {
    screen: editing ? "edit" : "review",
    file: primaryFile,
    saved: first || scene === "unsupported" ? null : sent ? { ...pair50 } : { ...pair40 },
    draft: first ? { ...empty } : { ...pair40 },
    preview: false,
    conflict: null,
    openedAfterSave: scene === "saved",
    unsupported: scene === "unsupported",
    rating: scene === "limit" ? "50" : null,
  };
  if (["preview", "conflict", "source"].includes(scene)) {
    state.draft = { ...pair50 };
    state.preview = scene !== "source";
  }
  if (scene === "conflict" || scene === "source") state.conflict = scene === "conflict" ? "target" : "source";
  if (scene === "confirm") {
    state.draft = { ...pair40 };
    state.preview = true;
  }
  render();
  if (scene === "confirm") openConfirm("save", "save");
}

function enterEditor() {
  state.screen = "edit";
  state.draft = { ...(state.saved || empty) };
  state.preview = false;
  state.openedAfterSave = false;
  render();
  $("hot").focus();
}

function updateSelection() {
  state.draft = { hot: $("hot").value, cold: $("cold").value };
  state.preview = false;
  render();
}

function backToCase() {
  state.screen = "review";
  state.conflict = null;
  state.preview = false;
  render();
  $("edit").focus();
}

function addRow(list, field, value) {
  const row = node("div");
  row.append(node("dt", "small", field), node("dd", "", value));
  list.append(row);
}

function openConfirm(kind, focusId) {
  pending = { kind, focusId };
  const detail = $("confirm-detail");
  detail.replaceChildren();
  if (kind === "discard") {
    $("confirm-title").textContent = t("舍弃关联草稿？", "Discard the association draft?");
    $("confirm-note").textContent = t("返回后仍使用已保存的入口来源。", "The case will keep its saved inlet sources.");
    $("confirm-cancel").textContent = t("继续编辑", "Keep editing");
    $("confirm-apply").textContent = t("舍弃草稿", "Discard draft");
  } else {
    const copy = kind === "save-as";
    const firstInPlace = !state.saved && !copy;
    $("confirm-title").textContent = firstInPlace
      ? t("确认首次保存关联", "Confirm the first association save")
      : t("确认保存关联", "Confirm association save");
    $("confirm-note").textContent = firstInPlace
      ? t("此文件将声明 H3 流程关联能力。旧读者将整份文件作为只读打开。", "This file will declare H3 flow association. Older readers will open the whole project as read-only.")
      : t("保存后打开工程的新会话，从待运行开始。", "After saving, the project opens in a new session waiting to run.");
    addRow(detail, t("保存目标", "Save destination"), copy ? copyFile : state.file);
    addRow(detail, t("热侧", "Hot side"), streamLabel(state.draft.hot));
    addRow(detail, t("冷侧", "Cold side"), streamLabel(state.draft.cold));
    $("confirm-cancel").textContent = t("取消", "Cancel");
    $("confirm-apply").textContent = t("确认保存", "Confirm save");
  }
  $("confirm").showModal();
  $("confirm-cancel").focus();
}

function cancelDialog() {
  const focusId = pending?.focusId;
  $("confirm").close();
  pending = null;
  if (focusId) $(focusId).focus();
}

function finishDialog() {
  const kind = pending.kind;
  $("confirm").close();
  pending = null;
  if (kind === "discard") {
    backToCase();
  } else {
    state.saved = { ...state.draft };
    if (kind === "save-as") state.file = copyFile;
    state.screen = "review";
    state.conflict = null;
    state.preview = false;
    state.rating = null;
    state.openedAfterSave = true;
    render();
    $("rate").focus();
  }
}

$("language").value = language;
$("size").value = params.get("size") === "20" ? "20" : "14";
document.documentElement.dataset.size = $("size").value;
$("review-controls").hidden = params.get("archive") === "1";
fillOptions();
$("scene").value = Object.hasOwn(scenarios, params.get("scene")) ? params.get("scene") : "entry";
setup($("scene").value);

$("scene").addEventListener("change", (e) => setup(e.target.value));
$("language").addEventListener("change", (e) => {
  language = e.target.value;
  fillOptions();
  render();
});
$("size").addEventListener("change", (e) => { document.documentElement.dataset.size = e.target.value; });
$("edit").addEventListener("click", enterEditor);
$("hot").addEventListener("change", updateSelection);
$("cold").addEventListener("change", updateSelection);
$("preview").addEventListener("click", () => {
  state.preview = true;
  render();
  const target = $("save").disabled ? $("preview") : $("save");
  target.focus();
});
$("reload-source").addEventListener("click", () => {
  state.conflict = null;
  state.preview = false;
  render();
  $("preview").focus();
});
$("reset").addEventListener("click", () => {
  state.draft = { ...(state.saved || empty) };
  state.preview = false;
  render();
  $("hot").focus();
});
$("leave").addEventListener("click", () => {
  if (isChanged()) openConfirm("discard", "leave");
  else backToCase();
});
$("save").addEventListener("click", () => openConfirm("save", "save"));
$("save-as").addEventListener("click", () => openConfirm("save-as", "save-as"));
$("confirm-cancel").addEventListener("click", cancelDialog);
$("confirm-apply").addEventListener("click", finishDialog);
$("confirm").addEventListener("cancel", (e) => {
  e.preventDefault();
  cancelDialog();
});
document.addEventListener("keydown", (e) => {
  if (state.screen !== "edit" || e.defaultPrevented) return;
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "s") {
    e.preventDefault();
    if (!$("confirm").open) $(e.shiftKey ? "save-as" : "save").click();
  } else if (!$("confirm").open && e.key === "Escape" && !(e.target instanceof HTMLSelectElement)) {
    e.preventDefault();
    $("leave").click();
  }
});
$("rate").addEventListener("click", () => {
  const entry = state.saved;
  if (same(entry, pair40) || same(entry, pair50)) {
    state.rating = same(entry, pair40) ? "40" : "50";
    state.openedAfterSave = false;
    render();
  }
});
