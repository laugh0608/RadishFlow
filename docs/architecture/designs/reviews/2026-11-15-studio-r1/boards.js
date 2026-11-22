"use strict";

const params = new URLSearchParams(location.search);
const language = params.get("lang") === "en" ? "en" : "zh";
const t = (zh, en) => language === "zh" ? zh : en;
const studio = document.querySelector("#studio");
const dialogue = document.querySelector("#dialog");

const project = "Feed–Heater–Flash";
const projectName = t("试验工程", "Pilot plant");
const fullCase = t("超限代表工况", "Exceeding synthetic case");
const equipmentCases = {
  "matrix-v2-case1": { task: "V-103", name: fullCase, duration: 120 },
  "matrix-v2-case2": { task: "V-103", name: fullCase, duration: 120 },
  "adopted-v3-case": { task: "V-202", name: "C1 synthetic source", duration: 120 },
};
const views = {
  home: t("主页", "Home"),
  properties: t("物性", "Properties"),
  equipment: t("设备", "Equipment"),
  diagram: t("流程图", "Flowsheet"),
  run: t("流程运行", "Flowsheet run"),
  results: t("流程结果", "Flowsheet results"),
};

const scenarios = {
  run: t("流程运行失败", "Failed flowsheet run"),
  locate: t("从诊断定位到字段", "Locate a field from a diagnostic"),
  checked: t("重检通过，等待重跑", "Recheck passed, rerun pending"),
  "flow-result": t("重跑后审阅结果", "Review results after rerun"),
  unavailable: t("诊断目标失效", "Diagnostic target unavailable"),
  equipment: t("独立设备与同名工况", "Standalone equipment and same-name cases"),
  "rating-active": t("切换工况后的在途校核", "Active rating after switching cases"),
  source: t("查看工况来源", "View a case source"),
  "source-return": t("返回原工况和字段", "Return to the original case and field"),
  "rating-result": t("单工况校核结果", "Single-case rating result"),
  export: t("单工况导出", "Single-case export"),
  "export-return": t("取消导出后回到原动作", "Return to export after canceling"),
  reopened: t("重开后的设备结果", "Equipment results after reopening"),
};

let view = "run";
const flow = {
  object: "Heater",
  revision: 12,
  status: "failed",
  list: "execution",
  heaterDraft: "230",
  appliedPressure: 230,
  flashDraft: "-1",
  checked: false,
  unavailable: false,
};
const equipment = {
  task: "V-103",
  selected: { "V-103": "matrix-v2-case1", "V-202": "adopted-v3-case" },
  drafts: { "matrix-v2-case2": "120e" },
  results: {},
  stale: new Set(),
  pending: null,
  tab: "inputs",
};
let diagnosticReturn = null;
let sourceReturn = null;
let dialogReturn = null;
let feedback = "";

function button(label, action, { id, variant, disabled = false, pressed, current = false } = {}) {
  return `<button type="button" data-action="${action}"${id ? ` id="${id}"` : ""}
    ${variant ? ` class="${variant}"` : ""}${disabled ? " disabled" : ""}
    ${pressed === undefined ? "" : ` aria-pressed="${pressed}"`}
    ${current ? ' aria-current="page"' : ""}>${label}</button>`;
}

function tag(label, tone = "") {
  return `<span class="tag ${tone}">${label}</span>`;
}

function panel(title, body, className = "") {
  return `<section class="panel ${className}"><header class="panel-head"><h2>${title}</h2></header>
    <div class="panel-body">${body}</div></section>`;
}

function notice(content, tone = "", id = "") {
  return `<p ${id ? `id="${id}" tabindex="-1"` : ""} class="notice ${tone}">${content}</p>`;
}

function pairs(entries) {
  return `<dl class="pairs">${entries.map(([label, value]) =>
    `<div><dt>${label}</dt><dd>${value}</dd></div>`).join("")}</dl>`;
}

function table(headers, rows) {
  return `<table class="data"><thead><tr>${headers.map(header =>
    `<th scope="col">${header}</th>`).join("")}</tr></thead>
    <tbody>${rows.map(cells => `<tr>${cells.map((cell, index) => index === 0
      ? `<th scope="row">${cell}</th>`
      : `<td class="numeric">${cell}</td>`).join("")}</tr>`).join("")}</tbody></table>`;
}

function selectField(id, label, options, selection, className = "") {
  return `<div class="field ${className}"><label for="${id}">${label}</label><select id="${id}">
    ${options.map(([value, name]) =>
      `<option value="${value}"${value === selection ? " selected" : ""}>${name}</option>`).join("")}
    </select></div>`;
}

const flowStatus = () => ({
  failed: tag(t("本次失败", "Run failed"), "bad"),
  stale: tag(t("结果过期", "Results stale"), "warn"),
  missing: tag(t("尚未运行", "No run yet"), "warn"),
  current: tag(t("结果当前", "Results current"), "good"),
})[flow.status];

function processDiagram() {
  return `<div class="diagram" aria-label="${t("流程对象", "Flowsheet objects")}">${["Feed", "Heater", "Flash"]
    .filter(object => !flow.unavailable || object !== "Heater")
    .map(object => button(object, `object-${object}`, {
      pressed: view === "diagram" && flow.object === object,
    })).join('<span class="arrow" aria-hidden="true">→</span>')}</div>`;
}

function header(title, context, actions = "") {
  return `<header class="page-header"><div class="meta">${context}</div>
    <div class="between"><h1 id="page-title" tabindex="-1">${title}</h1>
    <div class="row">${actions}</div></div></header>`;
}

function hasFlowDraft() {
  return flow.flashDraft !== null || flow.heaterDraft !== String(flow.appliedPressure);
}

function runPage() {
  const execution = flow.list === "execution";
  const historical = flow.status === "stale" || flow.status === "current";
  const listLabel = execution ? t("运行 #12 的诊断", "Run #12 diagnostics") : t("当前问题", "Current issues");
  const resolved = !execution && flow.checked;
  const changed = !execution && flow.revision === 13 && !flow.checked;
  const issue = flow.status === "missing" && execution
    ? notice(t("尚无本次会话的运行诊断。", "No run diagnostics in this session."))
    : resolved
    ? notice(t("原条目已解决。正式输入通过重检，重新运行后生成新结果。",
      "The original issue is resolved. Applied inputs pass recheck. Rerun to generate new results."), "good", "resolved")
    : changed
      ? notice(t("输入已更新。请重检当前问题。", "Inputs changed. Recheck current issues."), "warn", "check-needed")
      : `<article class="issue selected"><div class="between">${tag(execution ? "#12 · Heater" : "Heater", execution ? "bad" : "warn")}
        ${tag(execution && historical ? t("历史诊断", "Historical diagnostic") : t("阻塞", "Blocking"), "warn")}</div>
        <h3>${t("Heater 出口压力高于入口压力", "Heater outlet pressure exceeds inlet pressure")}</h3>
        <p class="meta">${t("目标字段 · 出口压力", "Target field · Outlet pressure")}</p>
        ${flow.unavailable || (execution && historical)
          ? notice(flow.unavailable ? t("Heater 已移除，原字段无法定位。", "Heater was removed. The original field is unavailable.")
            : t("执行时的输入已变更，请从当前问题定位。", "Run inputs changed. Locate from current issues."), "warn")
          : ""}
        <div>${button(t("定位字段", "Locate field"), "locate", {
          id: "locate", disabled: flow.unavailable || (execution && historical),
        })}</div></article>`;
  return header(views.run, `${t("当前流程", "Active flowsheet")} / ${project}`, flowStatus()
    + button(t("运行流程", "Run flowsheet"), "run-flow", {
      variant: "primary", id: "rerun", disabled: flow.unavailable || hasFlowDraft() || (flow.revision === 13 && !flow.checked),
    }))
    + `<main class="content"><div class="columns">
      <section class="panel"><header class="panel-head between">
        <h2 class="issue-list" id="issues" tabindex="-1">${listLabel}</h2>
        ${button(t("重检当前问题", "Recheck current issues"), "recheck", { id: "recheck" })}</header>
        <div class="tabs row">
          ${button(t("当前问题", "Current issues"), "list-current", { id: "current-issues", pressed: !execution })}
          ${button(t("运行诊断", "Run diagnostics"), "list-execution", { id: "execution-issues", pressed: execution })}
        </div>
        <div class="panel-body">${issue}${flow.flashDraft !== null ? `<div class="between notice warn">
          <span>${t("Flash 出口温度有未提交草稿。", "Flash outlet temperature has an unapplied draft.")}</span>
          ${button(t("查看草稿", "View draft"), "object-Flash", { id: "flash-draft" })}</div>` : ""}
        ${flow.heaterDraft !== String(flow.appliedPressure) ? `<div class="between notice warn">
          <span>${t("Heater 出口压力有未提交草稿。", "Heater outlet pressure has an unapplied draft.")}</span>
          ${button(t("查看草稿", "View draft"), "object-Heater")}</div>` : ""}</div>
      </section>
      ${panel(t("流程上下文", "Flowsheet context"), processDiagram() + pairs([
        [t("已应用计划", "Applied plan"), t("顺序稳态", "Sequential steady state")],
        [t("输入修订", "Input revision"), flow.revision],
        [t("最近运行", "Latest run"), flow.status === "missing" ? "—" : flow.status === "current" ? "#13" : "#12"],
      ]))}
    </div></main>`;
}

function diagramPage() {
  const heater = flow.object === "Heater";
  const feed = flow.object === "Feed";
  const field = heater ? t("出口压力", "Outlet pressure") : t("出口温度", "Outlet temperature");
  const formalValue = heater ? `${flow.appliedPressure} kPa` : "300 K";
  const hasFlashDraft = !heater && flow.flashDraft !== null;
  const edit = feed ? pairs([[field, formalValue]]) : `<div class="field">
    <label for="flow-value">${field}</label>
    <div class="row"><input class="input numeric" id="flow-value" inputmode="decimal"
       aria-describedby="flow-value-status" ${heater ? "" : "readonly"} ${hasFlashDraft ? 'aria-invalid="true"' : ""}><span>${heater ? "kPa" : "K"}</span></div>
    <p class="meta" id="flow-value-status">${!hasFlashDraft
      ? `${t("已提交", "Applied")} · ${formalValue}`
      : `<span class="error">${t("草稿未提交。温度必须大于 0 K。", "Unapplied draft. Temperature must exceed 0 K.")}</span>`}</p>
    ${heater ? `<div>${button(t("应用压力", "Apply pressure"), "apply-pressure", {
      id: "apply-pressure", variant: "primary",
    })}</div>` : hasFlashDraft ? `<div>${button(t("丢弃草稿", "Discard draft"), "discard-flash", { id: "discard-flash" })}</div>` : ""}
    </div>`;
  return header(views.diagram, project, flowStatus())
    + `<main class="content"><div class="columns ${diagnosticReturn ? "focused-diagram" : ""}">
      ${panel(t("流程对象", "Flowsheet objects"), processDiagram(), "canvas-context")}
      <section class="panel">
        <header class="panel-head stack">
          ${diagnosticReturn ? `<div>${button(diagnosticReturn.list === "current"
            ? t("← 返回当前问题", "← Back to current issues")
            : t("← 返回运行 #12 的诊断", "← Back to run #12 diagnostics"), "return-issue", { id: "return-issue", variant: "link" })}</div>` : ""}
          <h2>${flow.object} / ${t("对象检查器", "Object inspector")}</h2>
        </header>
        <div class="panel-body">
          ${feedback ? notice(feedback, "warn", "feedback") : ""}
          ${edit}
          ${heater ? `<p class="meta">${t("入口压力 · 210 kPa", "Inlet pressure · 210 kPa")}</p>` : ""}
        </div>
      </section>
    </div></main>`;
}

function flowResultsPage() {
  const body = flow.status === "missing"
    ? notice(t("尚无本次会话的流程结果。", "No flowsheet results in this session."), "warn")
    : flow.status === "current"
    ? table([t("流股", "Stream"), t("温度 · K", "Temperature · K"), t("摩尔流量 · mol/s", "Molar flow · mol/s")],
      [[t("Heater 出口", "Heater outlet"), "360", "1.00"]])
    : notice(flow.status === "failed"
      ? t("本次运行失败。请查看运行 #12 的诊断。", "Run #12 failed. Review its diagnostics.")
      : t("输入已变更。请重新运行以刷新正式数值。", "Inputs changed. Rerun to refresh the results."), "warn");
  return header(views.results, `${t("当前流程", "Active flowsheet")} / ${project}`,
    flowStatus() + button(t("返回流程运行", "Back to flowsheet run"), "view-run"))
    + `<main class="content"><div class="columns">
      ${panel(t("流股结果", "Stream results"), body)}
      ${panel(t("运行与来源", "Run and provenance"), pairs([
        [t("运行", "Run"), flow.status === "missing" ? "—" : flow.status === "current" ? "#13" : "#12"],
        [t("执行时输入修订", "Run input revision"), flow.status === "missing" ? "—" : flow.status === "current" ? "13" : "12"],
        [t("执行物性", "Property package"), flow.status === "missing" ? "—" : t("演示烃类混合物", "Demonstration hydrocarbon mixture")],
      ]))}
    </div></main>`;
}

const caseKey = () => equipment.selected[equipment.task];
const caseLabel = key => `${equipmentCases[key].name} · ${key}`;
const equipmentTitle = () => `${equipment.task} / ${t("立式气液分离器校核", "Vertical gas–liquid separator rating")}`;
const caseContext = () => `${equipment.task} / ${caseLabel(caseKey())}`;

function equipmentStatus(key) {
  if (equipment.pending === key) return tag(t("校核中", "Rating running"), "info");
  if (equipment.stale.has(key)) return tag(t("结果过期", "Result stale"), "warn");
  if (equipment.results[key]) return tag(t("结果当前", "Result current"), "good");
  return tag(t("未计算", "No calculation"), "warn");
}

function equipmentAction(key) {
  return equipment.pending
    ? button(t("取消在途校核", "Cancel active rating"), "cancel-rating", { id: "cancel-rating" })
    : button(t("校核所选工况", "Rate selected case"), "rate", {
      id: "rate", variant: "primary", disabled: Object.hasOwn(equipment.drafts, key),
    });
}

function equipmentInputs(key) {
  const duration = equipmentCases[key].duration;
  const dirty = Object.hasOwn(equipment.drafts, key);
  const value = Number(equipment.drafts[key]);
  const invalid = dirty && !(Number.isFinite(value) && value > 0);
  return `<div class="field">
    <label for="duration">${t("所需工作时间", "Required working time")}</label>
    <div class="row">
      <input type="text" inputmode="decimal" id="duration" class="input numeric"
         aria-describedby="duration-status duration-error"${invalid ? ' aria-invalid="true"' : ""}><span>s</span>
      ${button(t("查看来源", "View source"), "source-input", { id: "source-input" })}
    </div>
    <p id="duration-status" class="meta">${t("正式采用值", "Applied value")} · ${duration} s</p>
    <p class="error" id="duration-error"${invalid ? "" : " hidden"}>${t("草稿未提交，请输入有效的工作时间。", "Unapplied draft. Enter a valid working time.")}</p>
      <div class="row" id="duration-actions"${dirty ? "" : " hidden"}>
        ${button(t("应用工作时间", "Apply working time"), "apply-duration", { id: "apply-duration", disabled: invalid })}
        ${button(t("丢弃草稿", "Discard draft"), "discard-duration", { id: "discard-duration" })}
      </div>
    </div>`;
}

function equipmentResult(key) {
  const result = equipment.results[key];
  const cited = key === "adopted-v3-case";
  if (equipment.stale.has(key)) {
    return notice(t("本工况的输入已变更。请重新校核以更新结果和计算表。",
      "The case inputs changed. Rerun this case to refresh its result and calculation sheet."), "warn")
      + pairs([[t("上次计算", "Previous calculation"), result]]);
  }
  if (!result) {
    return notice(t("当前工况尚无执行结果。请显式校核所选工况。",
      "This case has no execution result. Run the selected case to generate one."), "warn");
  }
  return pairs([
    [t("计算记录", "Calculation"), result],
    [t("工况", "Case"), caseLabel(key)],
    [t("方法版本", "Method version"), cited ? "v3" : "v2"],
    [t("校核范围", "Rating scope"), t("气相容量与工作液容积", "Gas capacity and working liquid volume")],
    ...(cited ? [] : [[t("工程结论", "Engineering checks"), tag(t("工作容积超限", "Working volume limit exceeded"), "warn")]]),
  ]) + (cited ? "" : table([t("工作液容积", "Working liquid volume"), t("容积 · m³", "Volume · m³")],
    [[t("所需", "Required"), "0.72"], [t("可用", "Available"), "0.60"]]))
    + `<div class="row">
    ${button(t("查看计算来源", "View calculation source"), "source-result", { id: "source-result" })}
    ${button(t("导出 CSV…", "Export CSV…"), "export-csv", { id: "export-csv", disabled: Object.hasOwn(equipment.drafts, key) })}
    ${button(t("导出 JSON…", "Export JSON…"), "export-json", { id: "export-json", disabled: Object.hasOwn(equipment.drafts, key) })}
    </div>`;
}

function equipmentPage() {
  const key = caseKey();
  const inputs = equipment.tab === "inputs";
  const pending = equipment.pending;
  return header(equipmentTitle(), caseContext(), equipmentStatus(key) + equipmentAction(key))
    + `<main class="content">
      ${pending ? notice(`${t("在途校核", "Active rating")} · ${equipmentCases[pending].task} / ${caseLabel(pending)}`, "info") : ""}
      ${feedback ? notice(feedback, "info", "feedback") : ""}
      <div class="columns">
        <section class="panel"><header class="panel-head"><h2>${t("校核工况", "Rating case")}</h2></header>
          <div class="tabs row">
            ${button(t("输入", "Inputs"), "equipment-inputs", { pressed: inputs, id: "input-tab" })}
            ${button(t("结果与依据", "Results and basis"), "equipment-results", { pressed: !inputs, id: "result-tab" })}
          </div>
          <div class="panel-body">${inputs ? equipmentInputs(key) : equipmentResult(key)}</div>
        </section>
        <div class="stack">
          ${panel(t("设备与工况", "Equipment and case"), selectField("task", t("设备任务", "Equipment task"),
            [["V-103", "V-103"], ["V-202", "V-202"]], equipment.task)
            + `<div class="case-options" role="group" aria-label="${t("工况", "Cases")}">${Object.keys(equipmentCases)
              .filter(id => equipmentCases[id].task === equipment.task)
              .map(id => button(caseLabel(id), `select-${id}`, { pressed: key === id, id: id })).join("")}</div>`,
            "tab-selection")}
          ${equipment.task === "V-202" && !inputs ? panel(t("固定采纳依据", "Frozen adoption evidence"), pairs([
            [t("来源任务", "Source task"), "V-201"],
            [t("来源工况", "Source case"), "C1 synthetic source"],
            [t("保存状态", "Persistence"), t("随工程保存", "Saved with project")],
          ])) : ""}
        </div>
      </div>
    </main>`;
}

function sourcePage() {
  const key = sourceReturn.key;
  const result = sourceReturn.tab === "results";
  const title = result ? t("计算来源", "Calculation provenance") : t("所需工作时间 · 来源", "Required working time · Source");
  return header(title, caseContext(),
    button(result ? t("← 返回结果与依据", "← Back to results and basis") : t("← 返回原工况字段", "← Back to the case field"),
      "return-source", { id: "return-source", variant: "link" }))
    + `<main class="content"><div class="columns">
      ${panel(result ? t("执行时记录", "Captured execution record") : t("工程基础与采用值", "Engineering basis and applied value"),
        pairs(result ? [
          [t("计算记录", "Calculation"), equipment.results[key]],
          [t("工况", "Case"), caseLabel(key)],
          [t("方法版本", "Method version"), key === "adopted-v3-case" ? "v3" : "v2"],
        ] : [
          ...(key === "adopted-v3-case" ? [] : [[t("共享基础条目", "Shared basis entry"), "basis-time"]]),
          [t("来源记录", "Source record"), key === "adopted-v3-case" ? "adoption-source-2" : "source-test"],
          [t("工况", "Case"), caseLabel(key)],
          [t("采用值", "Applied value"), `${equipmentCases[key].duration} s`],
        ]))}
      ${panel(t("返回位置", "Return location"), pairs([
        [t("设备", "Equipment"), equipment.task],
        [t("页签", "Tab"), result ? t("结果与依据", "Results and basis") : t("输入", "Inputs")],
        [t("操作", "Action"), result ? t("查看计算来源", "View calculation source") : t("所需工作时间 / 查看来源", "Required working time / View source")],
      ]))}
    </div></main>`;
}

function homePage() {
  return header(views.home, t("RadishFlow 项目", "RadishFlow projects"))
    + `<main class="content"><div class="columns">
      ${panel(t("流程示例", "Flowsheet example"), `<p>${project}</p>
        ${button(t("打开流程图", "Open flowsheet"), "view-diagram", { variant: "primary" })}`)}
      ${panel(t("独立设备", "Standalone equipment"), `<p>V-103 / V-202</p>
        ${button(t("打开独立设备", "Open standalone equipment"), "view-equipment", { variant: "primary" })}`)}
    </div></main>`;
}

function propertiesPage() {
  return header(views.properties, project) + `<main class="content">
    ${panel(t("当前方法", "Active method"), pairs([
      [t("物性包", "Package"), t("演示烃类混合物", "Demonstration hydrocarbon mixture")],
    ]))}</main>`;
}

const pages = {
  home: homePage,
  properties: propertiesPage,
  run: runPage,
  diagram: diagramPage,
  results: flowResultsPage,
  equipment: equipmentPage,
  source: sourcePage,
};

function focus(id, scroll = null) {
  const element = document.getElementById(id);
  element?.focus({ preventScroll: true, focusVisible: true });
  if (scroll !== null) window.scrollTo(0, scroll);
  element?.scrollIntoView({ block: "nearest", inline: "nearest" });
}

function render(restoreFocus = null, scroll = null) {
  const inEquipment = view === "equipment" || view === "source";
  studio.innerHTML = `<header class="masthead">
    <div class="titlebar between"><strong>RadishFlow Studio</strong><span class="meta">${projectName}</span></div>
    <nav class="nav row" aria-label="${t("工作区", "Workspaces")}">${Object.entries(views).map(([key, label]) =>
      button(label, `view-${key}`, { id: `nav-${key}`, variant: "nav-item", current: key === (inEquipment ? "equipment" : view) })
    ).join("")}</nav>
    </header>${pages[view]()}`;

  const flowValue = document.querySelector("#flow-value");
  if (flowValue) flowValue.value = flow.object === "Heater" ? flow.heaterDraft : (flow.flashDraft ?? "300");
  const durationInput = document.querySelector("#duration");
  if (durationInput) durationInput.value = equipment.drafts[caseKey()] ?? equipmentCases[caseKey()].duration;
  if (restoreFocus) focus(restoreFocus, scroll);
}

function openView(next, destination = "page-title", scroll = 0) {
  view = next;
  feedback = "";
  render(destination, scroll);
}

function showDialog(kind) {
  const leave = kind === "reopen";
  const key = caseKey();
  const result = equipment.results[key];
  dialogReturn = { key, result, kind, trigger: leave ? "simulate-reopen" : `export-${kind}` };
  const title = leave ? t("重新打开工程？", "Reopen the project?") : `${t("保存计算表", "Save calculation sheet")} · ${kind.toUpperCase()}`;
  const body = leave
    ? notice(t("重新打开将丢弃本次会话的草稿和执行结果。已保存的正式输入与固定采纳依据仍可审阅。",
      "Reopening discards session drafts and execution results. Saved inputs and frozen adoption evidence remain available."), "warn")
    : `<p class="meta">${t("原生保存面板示意", "Native save dialog preview")}</p>` + pairs([
      [t("设备与工况", "Equipment and case"), `${equipment.task} / ${caseLabel(key)}`],
      [t("发起时计算", "Captured calculation"), result],
      [t("输出单位", "Export units"), "SI"],
    ]);
  dialogue.setAttribute("aria-labelledby", "dialog-title");
  dialogue.innerHTML = `<div class="panel-head"><h2 id="dialog-title">${title}</h2></div>
    <div class="panel-body">${body}<div class="row dialog-actions">
      ${button(leave ? t("继续编辑", "Keep editing") : t("取消", "Cancel"), "close-dialog", { id: "close-dialog" })}
      ${button(leave ? t("重新打开", "Reopen") : t("保存", "Save"), "confirm-dialog", { id: "confirm-dialog", variant: "primary" })}
    </div></div>`;
  dialogue.showModal();
  focus("close-dialog");
}

function closeDialog(confirmed) {
  const capture = dialogReturn;
  if (!capture) return;
  dialogReturn = null;
  dialogue.close();
  if (!confirmed) {
    if (capture.kind === "reopen") {
      focus(capture.trigger);
    } else {
      feedback = t("已取消导出。", "Export canceled.");
      render(capture.trigger);
    }
    return;
  }
  if (capture.kind === "reopen") {
    equipment.task = capture.key === "adopted-v3-case" ? "V-202" : "V-103";
    equipment.tab = "results";
    equipment.results = {};
    equipment.stale.clear();
    equipment.drafts = {};
    equipment.pending = null;
    Object.values(equipmentCases).forEach(record => { record.duration = 120; });
    ratingGeneration++;
    diagnosticReturn = null;
    sourceReturn = null;
    flow.heaterDraft = String(flow.appliedPressure);
    flow.flashDraft = null;
    flow.status = "missing";
    flow.checked = false;
    flow.list = "execution";
    openView("equipment");
    return;
  }
  feedback = equipment.results[capture.key] === capture.result && !equipment.stale.has(capture.key)
    && !Object.hasOwn(equipment.drafts, capture.key)
    ? t("导出示例完成。", "Export preview complete.")
    : t("原计算已变更。请返回工况并重新发起导出。", "The captured result changed. Start a new export from the original case.");
  render(capture.trigger);
}

let ratingGeneration = 0;
let nextCalculation = 25;

function updateDurationDraft(value) {
  const key = caseKey();
  const dirty = value !== String(equipmentCases[key].duration);
  if (dirty) equipment.drafts[key] = value;
  else delete equipment.drafts[key];
  const invalid = dirty && !(Number.isFinite(Number(value)) && Number(value) > 0);
  const input = document.getElementById("duration");
  input.setAttribute("aria-invalid", String(invalid));
  document.getElementById("duration-error").hidden = !invalid;
  document.getElementById("duration-actions").hidden = !dirty;
  document.getElementById("apply-duration").disabled = invalid;
  const rate = document.getElementById("rate");
  if (rate) rate.disabled = dirty;
}

function rateCase() {
  if (equipment.pending || Object.hasOwn(equipment.drafts, caseKey())) return;
  const target = caseKey();
  equipment.pending = target;
  const submittedDuration = equipmentCases[target].duration;
  feedback = "";
  const generation = ++ratingGeneration;
  render("cancel-rating");
  window.setTimeout(() => {
    if (ratingGeneration !== generation) return;
    equipment.pending = null;
    if (equipmentCases[target].duration === submittedDuration) {
      equipment.stale.delete(target);
      equipment.results[target] = `CALC-${nextCalculation++}`;
    }
    if (view !== "equipment") return;
    const active = document.activeElement;
    const selection = active.id === "duration" ? [active.selectionStart, active.selectionEnd] : null;
    const nextFocus = active.id === "cancel-rating"
      ? (Object.hasOwn(equipment.drafts, caseKey()) ? "result-tab" : "rate") : active.id;
    render(nextFocus);
    if (selection) document.querySelector("#duration")?.setSelectionRange(...selection);
  }, 4500);
}

function runFlowsheet() {
  if (hasFlowDraft() || flow.unavailable) return;
  if (flow.appliedPressure <= 210 && flow.checked) {
    flow.status = "current";
    openView("results");
  } else {
    flow.status = "failed";
    flow.list = "execution";
    openView("run", "locate");
  }
}

const actions = {
  locate() {
    if (flow.unavailable) return;
    diagnosticReturn = { list: flow.list, scroll: window.scrollY };
    flow.object = "Heater";
    openView("diagram", "flow-value");
  },
  "return-issue"() {
    const origin = diagnosticReturn;
    if (!origin) return;
    diagnosticReturn = null;
    flow.list = origin.list;
    const liveIssue = !flow.unavailable && (flow.revision === 12 || (flow.list === "current" && !flow.checked));
    openView("run", liveIssue && flow.revision === 12 ? "locate" : "issues", origin.scroll);
  },
  "apply-pressure"() {
    const pressure = Number(flow.heaterDraft);
    if (!(Number.isFinite(pressure) && pressure > 0 && pressure <= 210)) {
      feedback = t("请输入大于 0 且不高于入口压力的值。", "Enter a positive pressure at or below the inlet pressure.");
      render("flow-value");
      return;
    }
    flow.appliedPressure = pressure;
    flow.heaterDraft = String(pressure);
    flow.status = "stale";
    flow.revision = 13;
    flow.checked = false;
    feedback = t("压力已应用。返回诊断后重检当前问题。", "Pressure applied. Return to the diagnostics and recheck current issues.");
    render(diagnosticReturn ? "return-issue" : "apply-pressure");
  },
  "discard-flash"() { flow.flashDraft = null; render("flow-value"); },
  recheck() {
    flow.checked = flow.revision === 13;
    flow.list = "current";
    render(flow.checked ? "resolved" : "issues");
  },
  "run-flow": runFlowsheet,
  "list-current"() { flow.list = "current"; render("current-issues"); },
  "list-execution"() { flow.list = "execution"; render("execution-issues"); },
  "equipment-inputs"() { equipment.tab = "inputs"; openView("equipment", "input-tab", window.scrollY); },
  "equipment-results"() { equipment.tab = "results"; openView("equipment", "result-tab", window.scrollY); },
  rate: rateCase,
  "cancel-rating"() {
    const target = equipment.pending;
    if (!target) return;
    ratingGeneration++;
    equipment.pending = null;
    feedback = `${t("已取消", "Canceled")} · ${equipmentCases[target].task} / ${caseLabel(target)}`;
    render(equipment.drafts[caseKey()] ? "feedback" : "rate");
  },
  "apply-duration"() {
    const key = caseKey();
    const value = Number(equipment.drafts[key]);
    if (!(Number.isFinite(value) && value > 0)) return;
    equipmentCases[key].duration = value;
    if (equipment.results[key]) equipment.stale.add(key);
    delete equipment.drafts[key];
    render("duration");
  },
  "discard-duration"() { delete equipment.drafts[caseKey()]; render("duration"); },
  "source-input"() { openSource("source-input"); },
  "source-result"() { openSource("source-result"); },
  "return-source"() {
    const origin = sourceReturn;
    if (!origin) return;
    equipment.task = equipmentCases[origin.key].task;
    equipment.selected[equipment.task] = origin.key;
    equipment.tab = origin.tab;
    sourceReturn = null;
    openView("equipment", origin.trigger, origin.scroll);
  },
  "export-csv"() { showDialog("csv"); },
  "export-json"() { showDialog("json"); },
  "close-dialog"() { closeDialog(false); },
  "confirm-dialog"() { closeDialog(true); },
};

function openSource(trigger) {
  sourceReturn = { key: caseKey(), tab: equipment.tab, trigger, scroll: window.scrollY };
  openView("source", "return-source");
}

function changeCase(key) {
  equipment.selected[equipment.task] = key;
  feedback = "";
  render(key, window.scrollY);
}

function handleClick(event) {
  const control = event.target.closest("button[data-action]");
  if (!control || control.disabled) return;
  const { action } = control.dataset;
  if (Object.hasOwn(actions, action)) {
    actions[action]();
  } else if (action.startsWith("view-")) {
    openView(action.slice(5));
  } else if (action.startsWith("object-")) {
    flow.object = action.slice(7);
    openView("diagram", flow.object === "Feed" ? "page-title" : "flow-value");
  } else if (action.startsWith("select-")) {
    changeCase(action.slice(7));
  }
}

studio.addEventListener("click", handleClick);
dialogue.addEventListener("click", handleClick);
dialogue.addEventListener("cancel", event => { event.preventDefault(); closeDialog(false); });
studio.addEventListener("change", event => {
  if (event.target.id !== "task") return;
  equipment.task = event.target.value;
  feedback = "";
  render("task", window.scrollY);
});
studio.addEventListener("input", event => {
  if (event.target.id === "duration") updateDurationDraft(event.target.value);
  if (event.target.id !== "flow-value") return;
  if (flow.object === "Heater") flow.heaterDraft = event.target.value;
});

const scenario = Object.hasOwn(scenarios, params.get("board")) ? params.get("board") : "run";
const fontSize = params.get("size") === "20" ? "20" : "14";
document.documentElement.lang = t("zh-CN", "en");
document.documentElement.dataset.size = fontSize;

document.querySelector("#review").hidden = params.get("capture") === "1";
document.querySelector("#review-description").textContent = t("交互画板 · 已认可 · 使用预设计算与导出示例",
  "Interactive boards · Approved · Simulated run and export");
document.querySelector("#scenario-label").textContent = t("画板", "Board");
document.querySelector("#language-label").textContent = t("语言", "Language");
document.querySelector("#font-size-label").textContent = t("字号", "Font size");
const scenarioSelect = document.querySelector("#scenario");
for (const [key, name] of Object.entries(scenarios)) scenarioSelect.add(new Option(name, key));
scenarioSelect.value = scenario;
const languageSelect = document.querySelector("#language");
languageSelect.value = language;
const sizeSelect = document.querySelector("#font-size");
sizeSelect.value = fontSize;

function updateReviewSetting(key, value) {
  params.set(key, value);
  location.search = params.toString();
}

scenarioSelect.addEventListener("change", event => updateReviewSetting("board", event.target.value));
languageSelect.addEventListener("change", event => updateReviewSetting("lang", event.target.value));
sizeSelect.addEventListener("change", event => updateReviewSetting("size", event.target.value));
const reopenControl = document.querySelector("#simulate-reopen");
reopenControl.textContent = t("演示设备重开确认", "Preview equipment reopen");
reopenControl.addEventListener("click", () => showDialog("reopen"));

let initialFocus = null;

switch (scenario) {
  case "locate":
    view = "diagram";
    diagnosticReturn = { list: "execution", scroll: 0 };
    initialFocus = "flow-value";
    break;
  case "checked":
    flow.revision = 13;
    flow.appliedPressure = 180;
    flow.heaterDraft = "180";
    flow.status = "stale";
    flow.checked = true;
    flow.flashDraft = null;
    flow.list = "current";
    initialFocus = "resolved";
    break;
  case "flow-result":
    view = "results";
    flow.revision = 13;
    flow.appliedPressure = 180;
    flow.heaterDraft = "180";
    flow.checked = true;
    flow.status = "current";
    flow.flashDraft = null;
    break;
  case "unavailable":
    flow.unavailable = true;
    flow.object = "Feed";
    break;
  case "equipment":
    view = "equipment";
    break;
  case "rating-active":
    view = "equipment";
    equipment.pending = "matrix-v2-case1";
    equipment.selected["V-103"] = "matrix-v2-case2";
    break;
  case "source":
  case "source-return":
    view = scenario === "source" ? "source" : "equipment";
    equipment.selected["V-103"] = "matrix-v2-case2";
    if (scenario === "source") {
      sourceReturn = { key: caseKey(), tab: "inputs", trigger: "source-input", scroll: 0 };
      initialFocus = "return-source";
    } else {
      initialFocus = "source-input";
    }
    break;
  case "rating-result":
  case "export":
  case "export-return":
  case "reopened":
    view = "equipment";
    equipment.task = scenario === "rating-result" ? "V-103" : "V-202";
    equipment.tab = "results";
    if (scenario !== "reopened") equipment.results[caseKey()] = equipment.task === "V-202" ? "CALC-24" : "CALC-17";
    if (scenario === "reopened") equipment.drafts = {};
    if (scenario === "export-return") {
      feedback = t("已取消导出。", "Export canceled.");
      initialFocus = "export-csv";
    }
    break;
}

render(initialFocus, 0);
if (scenario === "export") showDialog("csv");
