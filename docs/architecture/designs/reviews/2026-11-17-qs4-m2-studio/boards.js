const labels = {
  zh: {
    workbench: "工作台", scenario: "评审状态", language: "语言", size: "字号",
    draft: "计划草稿", upgrade: "首次保存", saved: "工程已保存", current: "当前结果", failed: "运行失败",
    cancelling: "正在取消", missing: "重开后无结果", readonly: "只读工程",
    plan: "稳态运行计划", result: "运行结果", mixed: "混合分区 · 单撕裂回流",
    method: "块间连接", dag: "块间无环", recycle: "单撕裂回流", tear: "撕裂流股", locate: "定位 s-rba",
    source: "源端", sink: "汇端", initial: "回流初值", explicit: "显式组分流量", zero: "零组分流量",
    budget: "候选评价上限", scale: "尺度使用本次实际新鲜料总量，独立复核另计。",
    apply: "应用计划", discard: "放弃草稿", draftNotice: "先应用或放弃草稿，再运行。保存只写已应用的计划。",
    manual: "仅显式运行。Active 下也须点击 Run 或按 F5。", run: "运行 F5",
    confirmSave: "确认保存混合回流计划", saveNotice: "工程仍为 v4。本次保存增加混合回流能力。不支持该能力的旧版本会整份只读打开。",
    target: "保存目标", capability: "新增必需能力", cancel: "取消", save: "保存", saveAs: "另存为…", saveNow: "继续保存",
    reopen: "重开已保存工程", chooseCopy: "选择新目标后重新确认保存。",
    ready: "计划已应用", verification: "独立复核", iterations: "候选评价", absolute: "最大绝对残差",
    scaled: "最大缩放残差", balance: "分量守恒", passed: "通过", package: "本次物性包", detail: "最后一轮实际撕裂值",
    component: "组分", assumed: "汇端消费", returned: "源端返回", trace: "候选及复核轨迹", candidate: "第 15 次候选",
    failureNotice: "候选评价达到预算 1。展开诊断可查看最后一轮试值和闭合差。",
    diagnosis: "失败诊断 · s-rba", noResult: "当前会话尚无合格结果。点击 Run 或按 F5，重新计算已保存的计划。",
    cancelNotice: "取消已请求，等待计算收尾。此前接收的结果仍有效。",
    readonlyNotice: "此工程需要未知的计划或能力版本。文件以整份只读方式打开。",
    back: "返回工作区", locateMessage: "已定位 splitter-2.outlet_a → mixer-1.inlet_b"
  },
  en: {
    workbench: "Workbench", scenario: "Review state", language: "Language", size: "Text size",
    draft: "Plan draft", upgrade: "First save", saved: "Project saved", current: "Current result", failed: "Run failed",
    cancelling: "Cancelling", missing: "Reopened without results", readonly: "Read-only project",
    plan: "Steady-state run plan", result: "Run results", mixed: "Mixed partition · Single tear recycle",
    method: "Block connections", dag: "Acyclic blocks", recycle: "Single tear recycle", tear: "Tear stream", locate: "Locate s-rba",
    source: "Source", sink: "Sink", initial: "Recycle initialization", explicit: "Explicit component flows", zero: "Zero component flows",
    budget: "Candidate evaluation limit", scale: "Scaling uses the evaluated fresh feed total. Verification is counted separately.",
    apply: "Apply plan", discard: "Discard draft", draftNotice: "Apply or discard the draft before running. Save writes the applied plan.",
    manual: "Manual run only, including in Active mode. Select Run or press F5.", run: "Run F5",
    confirmSave: "Confirm mixed recycle save", saveNotice: "The project stays v4 and gains the mixed recycle capability. Older versions without it open the entire file read-only.",
    target: "Save destination", capability: "New required capability", cancel: "Cancel", save: "Save", saveAs: "Save As…", saveNow: "Continue and save",
    reopen: "Reopen saved project", chooseCopy: "Choose a new destination to review the save again.",
    ready: "Plan applied", verification: "Independent verification", iterations: "Candidate evaluations", absolute: "Max absolute residual",
    scaled: "Max scaled residual", balance: "Component balance", passed: "Passed", package: "Property package", detail: "Final evaluated tear boundary",
    component: "Component", assumed: "Sink consumption", returned: "Source return", trace: "Candidate and verification trace", candidate: "Candidate 15",
    failureNotice: "The run reached its budget of 1 candidate evaluation. Open the diagnostic to inspect the last trial and closure residual.",
    diagnosis: "Failure diagnostic · s-rba", noResult: "No qualified result in this session. Select Run or press F5 to solve the saved plan.",
    cancelNotice: "Cancellation requested. Waiting for the worker to finish. The previous accepted result is still available.",
    readonlyNotice: "This file requires an unsupported plan or capability version. The entire project opened read-only.",
    back: "Return to workspace", locateMessage: "Located splitter-2.outlet_a → mixer-1.inlet_b"
  }
};
const scenes = ["draft", "upgrade", "saved", "current", "failed", "cancelling", "missing", "readonly"];
const params = new URLSearchParams(location.search);
const language = document.querySelector("#language");
const size = document.querySelector("#size");
const scenario = document.querySelector("#scenario");
const board = document.querySelector("#board");
const saveDialog = document.querySelector("#save");
let scene = scenes.includes(params.get("scene")) ? params.get("scene") : "draft";
language.value = params.get("lang") === "en" ? "en" : "zh";
size.value = params.get("size") === "200" ? "200" : "100";
const t = key => labels[language.value][key];
const code = value => `<code>${value}</code>`;
const fact = (label, value) => `<div><dt>${t(label)}</dt><dd>${value}</dd></div>`;
const button = (action, label, primary = false) =>
  `<button type="button" data-action="${action}"${primary ? ' class="primary"' : ""}>${t(label)}</button>`;
const panel = (title, body, controls = "") =>
  `<section class="panel"><header class="panel-head"><h2 tabindex="-1">${t(title)}</h2>${controls}</header><div class="panel-body">${body}</div></section>`;

function plan() {
  return panel("plan", `
    <p class="message">${t("draftNotice")}</p>
    <label class="setting">${t("method")}<select id="method"><option>${t("dag")}</option><option selected>${t("recycle")}</option></select></label>
    <div id="recycle-fields">
    <label class="setting">${t("tear")}<select><option>s-rba</option></select></label>
    <dl class="facts">${fact("source", code("splitter-2.outlet_a"))}${fact("sink", code("mixer-1.inlet_b"))}</dl>
    ${button("locate", "locate")}
    <p class="muted" id="location" role="status" hidden></p>
    <label class="setting">${t("initial")}<select id="initial"><option selected>${t("explicit")}</option><option>${t("zero")}</option></select></label>
    <div id="components" class="split">
      <label class="setting">Methane (mol/s)<input type="number" min="0" value="10"></label>
      <label class="setting">Ethane (mol/s)<input type="number" min="0" value="1"></label>
    </div>
    <label class="setting">${t("budget")}<input type="number" min="1" max="500" value="500"></label>
    <p class="muted">${t("scale")}</p>
    </div>
    <div class="controls">${button("review-save", "apply", true)}${button("missing", "discard")}</div>
  `);
}

function current() {
  return panel("result", `
    ${scene === "cancelling" ? `<p class="message">${t("cancelNotice")}</p>` : ""}
    <h3>${t("mixed")}</h3>
    <p><code>s-rba</code> · <code>splitter-2.outlet_a → mixer-1.inlet_b</code></p>
    <dl class="facts">
      ${fact("iterations", "15")}${fact("verification", t("passed"))}
      ${fact("absolute", "≈ 3.55 × 10⁻¹² mol/s")}${fact("scaled", "≈ 7.10 × 10⁻¹³")}
      ${fact("balance", t("passed"))}${fact("package", code("binary-hydrocarbon-lite-v1"))}
    </dl>
    ${button("locate", "locate")}
    <p class="muted" id="location" role="status" hidden></p>
    <details><summary>${t("detail")}</summary>
      <table class="tear-table"><thead><tr><th>${t("component")}</th><th>${t("assumed")} (mol/s)</th><th>${t("returned")} (mol/s)</th></tr></thead>
      <tbody>${[["Methane", "0.458823529"], ["Ethane", "0.776470588"]].map(([name, value]) =>
        `<tr><th scope="row">${name}</th><td class="value" data-label="${t("assumed")} (mol/s)">${value}</td>
         <td class="value" data-label="${t("returned")} (mol/s)">${value}</td></tr>`).join("")}</tbody></table>
    </details>
    <details><summary>${t("trace")}</summary>
      <p>${t("candidate")} → ${t("verification")}</p>
      <p>≤ 1 × 10⁻¹⁰ mol/s · ≤ 1 × 10⁻¹⁰</p>
    </details>
  `, scene === "current" ? button("missing", "reopen") : "");
}

function render({ openSave = false } = {}) {
  document.documentElement.lang = language.value === "zh" ? "zh-CN" : "en";
  document.documentElement.dataset.size = size.value;
  for (const element of document.querySelectorAll("[data-text]")) {
    element.textContent = t(element.dataset.text);
  }
  scenario.replaceChildren(...scenes.map(value => new Option(t(value), value)));
  scenario.value = scene;
  const state = document.querySelector("#state");
  state.dataset.state = scene;
  state.textContent = t(scene);
  if (saveDialog.open && scene !== "upgrade") saveDialog.close("change");
  switch (scene) {
    case "draft":
      board.innerHTML = plan();
      break;
    case "upgrade":
      board.innerHTML = panel("plan",
        `<p>${t("ready")} · ${t("mixed")}</p>
        ${button("confirm", "save", true)} <p id="save-feedback" role="status" hidden></p>`);
      if (openSave && !saveDialog.open) showSave();
      break;
    case "saved":
      board.innerHTML = panel("plan", `<p>${t("ready")} · ${t("mixed")}</p><p class="muted">${t("manual")}</p>
        <div class="controls">${button("current", "run", true)}${button("missing", "reopen")}</div>`);
      break;
    case "current":
    case "cancelling":
      board.innerHTML = current();
      break;
    case "failed":
      board.innerHTML = panel("result", `<p class="message error">${t("failureNotice")}</p>
        <p>${code("solver.mixed.iteration_limit")}</p>${button("locate", "locate")}
        <p class="muted" id="location" role="status" hidden></p>
        <details><summary>${t("diagnosis")}</summary><p>Methane 0.39 mol/s · Ethane 0.66 mol/s</p></details>`);
      break;
    case "missing":
      board.innerHTML = panel("result", `<p>${t("noResult")}</p><p class="muted">${t("manual")}</p>`,
        button("current", "run", true));
      break;
    case "readonly":
      board.innerHTML = panel("plan", `<p class="message">${t("readonlyNotice")}</p>
        <p>${code("radishflow.solver.fixed-tp-mixed-recycle / 2")}</p>${button("missing", "back")}`);
      break;
  }
}

function switchScene(next, options) {
  scene = next;
  render(options);
  if (options?.focusBoard) {
    (board.querySelector(".primary") ?? board.querySelector("h2"))?.focus();
  }
}

function showSave() {
  saveDialog.returnValue = "";
  saveDialog.showModal();
}

scenario.addEventListener("change", () => switchScene(scenario.value, { openSave: true }));
language.addEventListener("change", () => render());
size.addEventListener("change", () => render());
board.addEventListener("change", event => {
  if (event.target.id === "method") {
    document.querySelector("#recycle-fields").hidden = event.target.selectedIndex === 0;
  }
  if (event.target.id === "initial") {
    document.querySelector("#components").hidden = event.target.selectedIndex === 1;
  }
});
board.addEventListener("click", event => {
  const action = event.target.closest("[data-action]")?.dataset.action;
  if (action === "review-save") {
    switchScene("upgrade", { focusBoard: true });
  }
  if (action === "confirm") showSave();
  if (action === "missing" || action === "current") switchScene(action, { focusBoard: true });
  if (action === "locate") {
    const location = document.querySelector("#location");
    location.hidden = false;
    location.textContent = t("locateMessage");
  }
});
saveDialog.addEventListener("close", () => {
  if (saveDialog.returnValue === "confirm") switchScene("saved", { focusBoard: true });
  if (saveDialog.returnValue === "save-as") {
    const feedback = document.querySelector("#save-feedback");
    feedback.hidden = false;
    feedback.textContent = t("chooseCopy");
  }
});
document.addEventListener("keydown", event => {
  if (event.key === "F5" && ["missing", "saved"].includes(scene)) {
    event.preventDefault();
    switchScene("current", { focusBoard: true });
  }
});
render({ openSave: true });
