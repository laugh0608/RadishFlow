const evidence = window.h3Evidence;
const params = new URLSearchParams(location.search);
const scenarios = {
  normal: ["两档纯水工况", "Two saved water cases"],
  phase: ["冷侧相态拒绝", "Cold-side phase rejected"],
  source: ["冷侧物性资格拒绝", "Cold-side property source rejected"],
  flow: ["流程运行失败", "Flow run failed"],
};
const stages = {
  missing: ["待运行", "Ready to run"],
  flow: ["正在运行流程", "Running flowsheet"],
  rating: ["正在复核工况", "Reviewing case"],
  stopping: ["正在停止", "Stopping"],
  cancelled: ["已取消", "Cancelled"],
  stale: ["待重新复核", "Review again"],
  blocked: ["复核受阻", "Review blocked"],
  failed: ["流程运行失败", "Flow run failed"],
  current: ["当前结果", "Current result"],
};

const state = {
  lang: params.get("lang") === "en" ? "en" : "zh",
  size: params.get("size") === "20" ? "20" : "14",
  scenario: Object.hasOwn(scenarios, params.get("scenario")) ? params.get("scenario") : "normal",
  selected: 0,
  states: new Map(),
  job: null,
  targeted: null,
  evidenceOpen: false,
  open: true,
};
state.selected = state.scenario === "normal" ? 0 : 1;

const delay = params.has("slow") ? 6000 : 650;
const $ = (id) => document.getElementById(id);
const t = (zh, en) => state.lang === "en" ? en : zh;
const local = (words) => t(...words);
const label = (name) => local(stages[name]);
const number = (value) => new Intl.NumberFormat(state.lang === "en" ? "en-US" : "zh-CN", {
  minimumFractionDigits: 2, maximumFractionDigits: 2,
}).format(value);
const units = (value, unit) => `<span class="numeric">${number(value)} <small>${unit}</small></span>`;
const sideName = (side) => side === "hot" ? t("热侧", "Hot side") : t("冷侧", "Cold side");
const focus = (key) => document.querySelector(`[data-focus="${key}"]`)?.focus();
const stateOf = (index) => state.job?.index === index ? state.job.stage : state.states.get(index) ?? "missing";
const pill = (text, tone = "neutral") => `<span class="pill" data-tone="${tone}">${text}</span>`;
const announce = (text) => { $("announcer").textContent = text; };

function controls() {
  document.documentElement.lang = t("zh-CN", "en");
  document.documentElement.dataset.size = state.size;
  $("review-label").textContent = t("H3 流程关联局部画板 · 待评审", "H3 linked review · Design draft");
  $("scenario-label").textContent = t("样例", "Fixture");
  $("language-label").textContent = t("语言", "Language");
  $("size-label").textContent = t("基准字号", "Base font size");
  $("scenario").innerHTML = Object.entries(scenarios).map(([key, value]) => `<option value="${key}">${local(value)}</option>`).join("");
  $("scenario").value = state.scenario;
  $("language").value = state.lang;
  $("size").value = state.size;
}

function cases() {
  return `<nav class="panel case-menu" aria-label="${t("关联工况", "Linked cases")}">
    <h2>${t("关联工况", "Linked cases")}</h2>${evidence.cases.map((item, index) => {
    const phase = stateOf(index);
    return `<button class="case-button" data-action="case" data-index="${index}" data-focus="case-${index}"
      aria-pressed="${state.selected === index}"><span class="task-row"><strong>${item.tag}</strong>
      ${pill(label(phase), phase === "current" ? "current" : "neutral")}</span>
      <span class="small">${t("纯水工况", "Water case")} · ${item.id}</span></button>`;
  }).join("")}</nav>`;
}

function references(item) {
  return `<section class="panel link-panel" aria-labelledby="references-title">
    <h2 id="references-title">${t("已保存流股关联", "Saved stream links")}</h2>
    <div class="link-cards">${["hot", "cold"].map((side) => {
      const stream = item.streams[side];
      return `<section id="reference-${side}" tabindex="-1" class="link-card ${state.targeted === side ? "is-targeted" : ""}">
        <h3>${sideName(side)}</h3><p class="mono">${stream.stream_id}</p>
        ${state.targeted === side ? `<p class="small">${state.scenario === "source" ? t("定物性与评审记录不匹配", "Properties differ from the reviewed source") : t("流股不满足纯水单液相资格", "Stream is outside the pure-water liquid qualification")}</p>` : ""}
      </section>`;
    }).join("")}</div></section>`;
}

function activity() {
  if (!state.job) return "";
  const stopping = state.job.stage === "stopping";
  return `<div class="activity" tabindex="-1" data-focus="activity"><div class="activity-label"><strong>${label(state.job.stage)}</strong>
    <span class="small">${evidence.cases[state.job.index].tag} · ${t("纯水工况", "Water case")}</span></div>
    <button class="button secondary" data-action="cancel" data-focus="cancel" ${stopping ? "disabled" : ""}>
      ${stopping ? t("已请求停止", "Stop requested") : t("取消复核", "Cancel review")}</button></div>`;
}

function diagnostic() {
  const sample = evidence.scenarios[state.scenario];
  const side = sample.streams[0]?.side;
  const words = {
    phase: t("冷侧流股未通过纯水单液相检查。", "The cold-side stream did not qualify as pure liquid water."),
    source: t("冷侧保存的液体定物性与评审记录不匹配。", "The saved cold-side properties differ from the reviewed source."),
    flow: t("流程求解失败。没有可用于本次复核的快照。", "The flow run failed before a snapshot became available for review."),
  };
  return `<div class="diagnostic"><h3>${words[state.scenario]}</h3><code>${sample.code.replaceAll(".", ".<wbr>")}</code>
    ${side ? `<button class="button inline" data-action="locate" data-focus="diagnostic" data-side="${side}">
      ${t("查看冷侧流股", "Locate cold-side stream")}</button>` : ""}
    ${state.targeted ? `<button class="button inline" data-action="return-diagnostic" data-focus="return-diagnostic">
      ${t("返回诊断", "Back to diagnostic")}</button>` : ""}</div>`;
}

function performance(item) {
  const data = item.result;
  const checks = [
    ["heat_duty", t("最低热负荷", "Minimum heat duty"), data.duty, item.limits.min_heat_duty_w, "W", "≥"],
    ["hot_pressure_drop", t("热侧压降上限", "Hot-side pressure drop limit"), data.hotDrop, item.limits.max_hot_pressure_drop_pa, "Pa", "≤"],
    ["cold_pressure_drop", t("冷侧压降上限", "Cold-side pressure drop limit"), data.coldDrop, item.limits.max_cold_pressure_drop_pa, "Pa", "≤"],
  ];
  return `<div class="review-result"><div class="status-line">
      ${pill(label("current"), "current")}
      ${data.checks.cold_pressure_drop === "failed" ? pill(t("1 项超限", "1 limit exceeded"), "caution") : pill(t("三项通过", "All three checks passed"), "good")}
    </div><dl class="indicators">
      <div class="metric"><dt>${t("热负荷", "Heat duty")}</dt><dd>${units(data.duty, "W")}</dd></div>
      <div class="metric"><dt>UA</dt><dd>${units(data.ua, "W/K")}</dd></div></dl>
    <div class="result-grid">
      <div class="data-box"><h3>${t("出口温度与压降", "Outlet temperatures and pressure drops")}</h3><table>
        <thead><tr><th>${t("侧别", "Side")}</th><th>${t("温度", "Temperature")}</th><th>${t("压降", "Pressure drop")}</th></tr></thead>
        <tbody><tr><th scope="row">${sideName("hot")}</th><td>${units(data.hotOutlet, "K")}</td><td>${units(data.hotDrop, "Pa")}</td></tr>
          <tr><th scope="row">${sideName("cold")}</th><td>${units(data.coldOutlet, "K")}</td><td>${units(data.coldDrop, "Pa")}</td></tr></tbody>
      </table></div>
      <div class="data-box"><h3>${t("校核准则", "Rating limits")}</h3>${checks.map(([key, name, actual, limit, unit, comparator]) =>
        `<div class="criterion"><span>${name}</span>${data.checks[key] === "failed" ? pill(t("超限", "Exceeded"), "caution") : pill(t("通过", "Passed"), "good")}
        <small>${number(actual)} ${unit} · ${comparator} ${number(limit)} ${unit}</small></div>`).join("")}</div>
    </div></div>`;
}

function sources(item, phase) {
  const data = item.result;
  return `<details class="panel detail" id="source-details" ${state.evidenceOpen ? "open" : ""}>
    <summary data-focus="sources">${t("物性依据与流股身份", "Property evidence and stream identity")}</summary>
    <div class="detail-body trace">
      <p>${t("相态依据", "Phase basis")} · NIST / Gubkov et al. · 293.15–303.15 K · 50,000–120,000 Pa</p>
      <p>${t("定物性来源", "Fixed properties")} · Pátek et al., Table 8 · nist-water-table-8 · r3</p>
      <p>${t("关联身份", "Saved identities")} · <code>${item.streams.hot.creation_id}</code> / <code>${item.streams.cold.creation_id}</code></p>
      ${phase === "current" ? `<h3>${t("本次快照入口", "Inputs used from this snapshot")}</h3>
        <div class="effective-inlets">${[["hot", data.hotInlet], ["cold", data.coldInlet]].map(([side, inlet]) =>
          `<p><strong>${sideName(side)}</strong> · ${number(inlet.temperature_k)} K · ${number(inlet.pressure_pa)} Pa · ${number(inlet.molar_flow_mol_s)} mol/s</p>`).join("")}</div>
        <p>${t("来源快照", "Source snapshot")} · <code>${data.flow.snapshot_id}</code> · ${data.flow.package.id} · v${data.flow.package.version}</p>
        <p class="identity mono">${data.flow.package.payload_fingerprint}</p>` : ""}
    </div></details>`;
}

function outcome(item, phase) {
  if (phase === "current") return performance(item);
  if (phase === "blocked" || phase === "failed") return `${pill(label(phase), "bad")}${diagnostic()}`;
  const hints = {
    missing: t("运行流程后，使用同一次快照复核当前工况。", "Run the flowsheet and use its new snapshot to review this case."),
    flow: t("正在计算两条关联流股。", "Calculating the two linked streams."),
    rating: t("正在校验快照与流股资格并执行换热器校核。", "Checking snapshot eligibility and rating the exchanger."),
    stopping: t("正在等待后台任务停止。", "Waiting for the background task to stop."),
    cancelled: t("如需结果，请显式再次运行复核。", "Run this review again to obtain a result."),
    stale: t("流程已重新运行，需重新复核此工况。", "The flowsheet was rerun. Review this case again."),
  };
  return `<div class="stack">${pill(label(phase))}<p class="message">${hints[phase]}</p></div>`;
}

function render(prefer) {
  const previous = prefer ?? document.activeElement?.dataset.focus;
  $("close-review").hidden = !state.open;
  $("close-review").textContent = t("返回工作区", "Back to workspace");
  if (!state.open) {
    $("board").innerHTML = `<section class="after panel"><h1>${t("原工作区", "Original workspace")}</h1>
      <button class="button primary" data-action="reopen" data-focus="reopen">${t("重新打开复核工程", "Reopen review project")}</button></section>`;
  } else {
    const item = evidence.cases[state.selected];
    const phase = stateOf(state.selected);
    $("board").innerHTML = `<div class="page-head"><span class="pill readonly">${t("工程只读", "Read-only project")}</span>
      <h1>${t("同心套管流程关联复核", "Linked concentric-tube review")}</h1>
      <p class="head-line">h3-linked-review-pair.rfproj.json · v4</p></div>
      <div class="workspace-grid">${cases()}<div class="stack">
      <section class="panel case-head"><div class="heading-text"><h2>${item.tag} · ${t("纯水工况", "Water case")}</h2></div>
        <div class="actions"><button class="button primary" data-action="run" data-focus="run" ${state.job ? "disabled" : ""}>
        ${t("运行并复核当前工况", "Run and review this case")}</button></div></section>
      ${references(item)}${activity()}<section class="panel outcome" aria-label="${t("复核结果", "Review result")}">
      ${outcome(item, phase)}</section>${sources(item, phase)}</div></div>`;
    $("source-details").addEventListener("toggle", (event) => {
      if (event.currentTarget.isConnected) state.evidenceOpen = event.currentTarget.open;
    });
  }
  if (previous) focus(previous);
}

function reset(open = true) {
  state.open = open;
  state.job = null;
  state.states.clear();
  state.targeted = null;
  state.evidenceOpen = false;
  state.selected = state.scenario === "normal" ? 0 : 1;
}

function finish(job, phase) {
  if (state.job !== job || !state.open) return;
  const returnToAction = ["cancel", "activity"].includes(document.activeElement?.dataset.focus);
  state.job = null;
  state.states.set(job.index, phase);
  announce(`${evidence.cases[job.index].tag} · ${label(phase)}`);
  render(returnToAction ? "run" : undefined);
}

function start() {
  if (state.job) return;
  const index = state.selected;
  state.targeted = null;
  if (index === 1 && state.scenario === "source") {
    state.states.set(index, "blocked");
    announce(label("blocked"));
    render("diagnostic");
    return;
  }
  for (const [caseIndex, phase] of state.states) if (phase === "current") state.states.set(caseIndex, "stale");
  const job = {index, stage: "flow"};
  state.job = job;
  announce(label("flow"));
  render("cancel");
  setTimeout(() => {
    if (state.job !== job || job.stage !== "flow") return;
    if (state.scenario === "flow") { finish(job, "failed"); return; }
    job.stage = "rating";
    announce(label("rating"));
    render();
    setTimeout(() => {
      if (state.job === job && job.stage === "rating") finish(job, index === 1 && state.scenario === "phase" ? "blocked" : "current");
    }, delay / 2);
  }, delay);
}

function stop() {
  const job = state.job;
  if (!job || job.stage === "stopping") return;
  job.stage = "stopping";
  announce(label("stopping"));
  render("activity");
  setTimeout(() => finish(job, "cancelled"), 850);
}

function returnToDiagnostic() {
  state.targeted = null;
  render("diagnostic");
}

function close() {
  reset(false);
  render("reopen");
}

$("board").addEventListener("click", (event) => {
  const button = event.target.closest("button[data-action]");
  if (!button) return;
  switch (button.dataset.action) {
    case "case": state.selected = Number(button.dataset.index); state.targeted = null; render(`case-${state.selected}`); break;
    case "run": start(); break;
    case "cancel": stop(); break;
    case "locate": state.targeted = button.dataset.side; render(); $(`reference-${state.targeted}`).focus(); break;
    case "return-diagnostic": returnToDiagnostic(); break;
    case "reopen": reset(); render(`case-${state.selected}`); break;
  }
});
$("close-review").addEventListener("click", close);
$("scenario").addEventListener("change", (event) => { state.scenario = event.target.value; reset(); render(); });
$("language").addEventListener("change", (event) => { state.lang = event.target.value; controls(); render(); });
$("size").addEventListener("change", (event) => { state.size = event.target.value; controls(); render(); });
document.addEventListener("keydown", (event) => {
  if (event.key !== "Escape" || !state.open || event.target.closest(".review-controls")) return;
  event.preventDefault();
  if (state.targeted) returnToDiagnostic(); else close();
});
controls();
render();
