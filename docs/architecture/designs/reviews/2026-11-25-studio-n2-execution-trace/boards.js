const labels = {
  review: ["N2 局部交互评审", "N2 interaction review"],
  demo: ["演示场景", "Scenario"],
  language: ["语言", "Language"],
  size: ["基准字号", "Base text"],
  mixer: ["双进料 Mixer", "Two-feed Mixer"],
  large: ["大型序贯流程", "Large sequential flowsheet"],
  other: ["M3 串联回流", "M3 serial recycle"],
  process: ["计算过程", "Execution trace"],
  problems: ["当前问题", "Current problems"],
  resultTable: ["结果表", "Result table"],
  results: ["查看结果", "View results"],
  log: ["查看运行日志", "View run log"],
  logTitle: ["运行日志", "Run log"],
  logText: ["此入口打开现有工作区运行日志。", "This entry opens the existing workspace run log."],
  selectedRun: ["查看运行", "Review run"],
  currentResult: ["当前结果", "Current result"],
  noCurrent: ["暂无当前结果", "No current result"],
  revision: ["输入修订", "Input revision"],
  outcome: ["执行终态", "Execution outcome"],
  receipt: ["接收处置", "Acceptance"],
  succeeded: ["已完成", "Completed"],
  failed: ["失败", "Failed"],
  cancelled: ["已取消", "Cancelled"],
  accepted: ["已接收结果", "Result accepted"],
  diagnosticAccepted: ["已接收失败诊断", "Failure diagnostic accepted"],
  discarded: ["结果未接收", "Result not accepted"],
  steps: ["执行步骤", "Execution steps"],
  pending: ["未执行", "Not executed"],
  backToSteps: ["返回步骤", "Back to steps"],
  backToStep: ["返回原步骤", "Back to step"],
  backToTrace: ["返回计算过程", "Back to execution trace"],
  object: ["关联对象", "Target"],
  detail: ["技术详情", "Technical details"],
  first: ["第一股进料", "First feed"],
  second: ["第二股进料", "Second feed"],
  mix: ["混合器", "Mixer"],
  doneText: ["此步骤完成，输出已写入本次求解的工作状态。", "The step completed and its output entered this run's working state."],
  unknownText: ["该步骤位于失败点之后，求解未进入此步骤。", "The solver did not reach this step after the failure."],
  cancelledText: ["运行在到达此步骤前取消。", "The run was cancelled before this step began."],
  fault: ["第二股进料的组成缺少乙烷", "The second feed is missing ethane"],
  faultText: ["当前组成只提供了一种组分，二元烃 Lite 需要两种。混合器未执行。", "The composition supplies one component. Binary Hydrocarbon Lite requires two. The Mixer did not run."],
  locate: ["定位到流股组成", "Go to stream composition"],
  composition: ["流股组成", "Stream composition"],
  locationText: ["已定位到第二股进料的组成。甲烷已提供，乙烷缺失。", "The second feed composition is selected. Methane is present and ethane is missing."],
  locationHint: ["返回后仍选中原失败步骤。", "Returning restores the original failed step."],
  missingTarget: ["流股已移除，原诊断仍可查看。", "The stream was removed. The original diagnostic is still available."],
  noProblems: ["此演示场景无当前问题。", "There are no current problems in this scenario."],
  detailsMissing: ["记录已截断。步骤 513 至 4095 的详情未保留，实际失败步骤仍可查看。", "Trace truncated. Details for steps 513–4095 were omitted. The observed failed step is still available."],
  prefix: ["前 512 步的记录可用。此画板展示首行和失败步骤。", "The first 512 steps are available. This board shows the first row and the failed step."],
  missingSteps: ["步骤 513 至 4095 · 记录缺失", "Steps 513–4095 · Records missing"],
  sampleFailure: ["尾部设备执行失败", "A later unit failed"],
  faultOverflow: ["采集到步骤 4096 的失败与设备身份。较早缺段的步骤状态未知。", "The failure and unit identity at step 4096 were captured. Earlier omitted step states are unknown."],
  noRuns: ["本会话尚无运行", "No runs in this session"],
  noRunsText: ["显式运行后，可在此审阅稳态序贯执行步骤。", "Run the flowsheet to review the executed sequential steps here."],
  notCollected: ["此运行未采集计算过程", "No execution trace was collected for this run"],
  notCollectedText: ["M3 的 A、B 区证据仍沿用分区报告。", "M3 evidence for regions A and B remains in the partition report."],
  output: ["流股", "Stream"],
  flow: ["摩尔流量", "Molar flow"],
  outlet: ["Mixer 出口", "Mixer outlet"],
};

const feedA = { index: 1, name: "first", unit: "feed-1", stream: "stream-feed-1-outlet" };
const feedB = { index: 2, name: "second", unit: "feed-2", stream: "stream-feed-2-outlet" };
const mixer = { index: 3, name: "mix", unit: "mixer-1", stream: "stream-mixer-1-outlet" };
const completedRun = {
  number: 18, revision: 16, outcome: "succeeded", receipt: "accepted",
  steps: [feedA, feedB, mixer].map(step => ({ ...step, status: "succeeded" })),
};
const failedRun = { number: 19, revision: 17, outcome: "failed", receipt: "diagnosticAccepted", steps: [
  { ...feedA, status: "succeeded" },
  { ...feedB, status: "failed", cause: "fault", diagnostic: "solver.step.execution" },
  { ...mixer, status: "pending" },
] };
const scenarios = {
  failed: { runs: [failedRun, completedRun], currentProblem: failedRun.steps[1] },
  historical: { runs: [failedRun], targetRemoved: true },
  completed: { runs: [completedRun], current: 18 },
  cancelled: {
    runs: [{ number: 20, revision: 16, outcome: "cancelled", receipt: "discarded", steps: [
      { ...feedA, status: "succeeded" }, { ...feedB, status: "succeeded" }, { ...mixer, status: "pending" },
    ] }, completedRun], current: 18,
  },
  truncated: {
    project: "large", runs: [{ number: 21, revision: 9, outcome: "failed", receipt: "diagnosticAccepted", truncated: true, steps: [
      { ...feedA, status: "succeeded" },
      { index: 4096, name: "sampleFailure", unit: "unit-4096", status: "failed", cause: "faultOverflow", diagnostic: "solver.step.execution" },
    ] }],
  },
  uncollected: { project: "other", runs: [{ number: 22, revision: 8, outcome: "succeeded", receipt: "accepted" }] },
  empty: { runs: [] },
};
const scenarioNames = {
  failed: ["第二步失败", "Second step fails"],
  historical: ["历史失败目标已移除", "History: target removed"],
  completed: ["收敛并接收", "Converged and accepted"],
  cancelled: ["取消时保留旧当前结果", "Cancelled · prior result"],
  truncated: ["记录截断", "Trace truncated"],
  uncollected: ["未采集过程", "Trace not collected"],
  empty: ["尚无运行", "No runs yet"],
};

const root = document.querySelector("#workbench");
const scenarioSelect = document.querySelector("#scenario");
const languageSelect = document.querySelector("#language");
const sizeSelect = document.querySelector("#size");
const params = new URLSearchParams(location.search);
let language = params.get("lang") === "en" ? "en" : "zh";
let scenario = Object.hasOwn(scenarios, params.get("case")) ? params.get("case") : "failed";
let runIndex = 0;
let stepIndex = 0;
let view = "trace";
let returnFocus = "#show-trace";
let narrowView = "list";
let located = false;

const selectText = pair => pair[language === "zh" ? 0 : 1];
const text = key => selectText(labels[key]);
const activeCase = () => scenarios[scenario];
const activeRun = () => activeCase().runs[runIndex];
const currentRun = () => activeRun()?.number === activeCase().current && activeRun() !== undefined;
const statusClass = { succeeded: "ok", failed: "fail", pending: "muted", cancelled: "warn" };

function chooseDefaultStep() {
  const failedStep = activeRun()?.steps?.findIndex(step => step.status === "failed");
  stepIndex = failedStep >= 0 ? failedStep : 0;
  located = false;
}

function updateReviewControls() {
  document.documentElement.lang = language === "zh" ? "zh-CN" : "en";
  document.title = `RadishFlow Studio · ${text("review")}`;
  document.querySelector(".review").ariaLabel = text("review");
  for (const [id, key] of [["review-title", "review"], ["case-label", "demo"], ["language-label", "language"], ["size-label", "size"]]) {
    document.getElementById(id).textContent = text(key);
  }
  scenarioSelect.replaceChildren(...Object.entries(scenarioNames).map(([id, names]) => new Option(selectText(names), id)));
  scenarioSelect.value = scenario;
  languageSelect.value = language;
  document.documentElement.style.fontSize = `${sizeSelect.value}px`;
}

function runBar(current) {
  const options = activeCase().runs.map((run, index) =>
    `<option value="${index}"${index === runIndex ? " selected" : ""}>#${run.number} · ${text(run.outcome)}</option>`
  ).join("");
  return `<div class="run-bar">
    <label>${text("selectedRun")} <select id="run-select">${options}</select></label>
    <span class="current-label">${activeCase().current ? `${text("currentResult")} #${activeCase().current}` : text("noCurrent")}</span>
    ${currentRun() ? `<button id="view-results" data-view="results">${text("results")}</button>` : ""}
    <button id="show-log" data-view="log">${text("log")}</button>
  </div>
  <dl class="run-facts">
    <div><dt>${text("revision")}</dt><dd>r${current.revision}</dd></div>
    <div><dt>${text("outcome")}</dt><dd class="status ${statusClass[current.outcome]}">${text(current.outcome)}</dd></div>
    <div><dt>${text("receipt")}</dt><dd>${text(current.receipt)}</dd></div>
  </dl>`;
}

function details(current, step) {
  const failures = step.status === "failed";
  const feedFailure = step.cause === "fault";
  let description = step.status === "succeeded" ? text("doneText") : text(current.outcome === "cancelled" ? "cancelledText" : "unknownText");
  if (failures) description = text(step.cause === "fault" ? "faultText" : "faultOverflow");
  if (located) return `<section class="detail" aria-labelledby="detail-title">
    <h2 id="detail-title" tabindex="-1">${text("composition")}</h2>
    <p><code>${step.stream}</code></p><p>${text("locationText")}</p><p class="muted">${text("locationHint")}</p>
    <button id="back-from-target" class="primary">${text("backToStep")}</button>
  </section>`;
  return `<section class="detail" aria-labelledby="detail-title">
    <button id="back-to-list" class="mobile-return">${text("backToSteps")}</button>
    <h2 id="detail-title" tabindex="-1">${text(step.name)} · ${text(step.status)}</h2>
    <div${failures ? ' class="error-detail"' : ""}>
      ${step.cause === "fault" ? `<h3>${text("fault")}</h3>` : ""}
      <p>${description}</p>
      <dl><dt>${text("object")}</dt><dd><code>${step.unit}</code>${step.stream ? ` <span aria-hidden="true">·</span> <code>${step.stream}</code>` : ""}</dd></dl>
    </div>
    ${feedFailure && activeCase().targetRemoved ? `<p class="muted">${text("missingTarget")}</p>` : ""}
    ${feedFailure && !activeCase().targetRemoved ? `<div class="actions"><button id="locate" class="primary">${text("locate")}</button></div>` : ""}
    ${step.diagnostic ? `<details><summary>${text("detail")}</summary><p><code>${step.diagnostic}</code></p></details>` : ""}
  </section>`;
}

function trace(current) {
  if (!current) return `<section class="empty"><h2>${text("noRuns")}</h2><p>${text("noRunsText")}</p></section>`;
  if (!current.steps) return `${runBar(current)}<section class="empty"><h2>${text("notCollected")}</h2><p>${text("notCollectedText")}</p></section>`;
  const rows = current.steps.map((step, index) => {
    const gap = current.truncated && index === 1 ? `<li class="gap">${text("missingSteps")}</li>` : "";
    return `${gap}<li><button type="button" data-step="${index}"${index === stepIndex ? ' aria-current="step"' : ""}>
      <span class="index">${step.index}</span>
      <span><span class="step-name">${text(step.name)}</span><code>${step.unit}</code></span>
      <span class="status ${statusClass[step.status]}">${text(step.status)}</span>
    </button></li>`;
  }).join("");
  return `${runBar(current)}
    ${current.truncated ? `<p class="notice">${text("detailsMissing")}</p>` : ""}
    <div class="split" data-view="${narrowView}">
      <section class="step-panel" aria-labelledby="steps-title">
        <h2 id="steps-title">${text("steps")}</h2>
        ${current.truncated ? `<p class="gap">${text("prefix")}</p>` : ""}
        <ol class="steps">${rows}</ol>
      </section>
      ${details(current, current.steps[stepIndex])}
    </div>`;
}

function problems() {
  const failed = activeCase().currentProblem;
  return `<section class="auxiliary">
    <h2 id="aux-title" tabindex="-1">${text("problems")}</h2>
    <p>${failed ? text("faultText") : text("noProblems")}</p>
    ${failed ? `<p><code>${failed.stream}</code></p>` : ""}
    <button id="return-to-trace">${text("backToTrace")}</button>
  </section>`;
}

function log() {
  return `<section class="auxiliary">
    <h2 id="aux-title" tabindex="-1">${text("logTitle")}</h2>
    <p>${text("logText")}</p>
    <button id="return-to-trace">${text("backToTrace")}</button>
  </section>`;
}

function results() {
  const number = activeCase().current;
  return `<section class="auxiliary">
    <h2 id="aux-title" tabindex="-1">${number ? `${text("currentResult")} #${number}` : text("noCurrent")}</h2>
    ${number ? `<table><thead><tr><th>${text("output")}</th><th>${text("flow")}</th></tr></thead><tbody>
      <tr><td>${text("first")}</td><td>2 mol/s</td></tr><tr><td>${text("second")}</td><td>3 mol/s</td></tr><tr><td>${text("outlet")}</td><td>5 mol/s</td></tr>
    </tbody></table>` : ""}
    <div class="actions"><button id="return-to-trace">${text("backToTrace")}</button></div>
  </section>`;
}

function render(focus) {
  const current = activeRun();
  const content = { trace: () => trace(current), problems, results, log };
  root.innerHTML = `<div class="app">
    <header class="app-header"><strong>RadishFlow Studio</strong><span>${text(activeCase().project ?? "mixer")}</span></header>
    <nav class="navigation" aria-label="${text("process")}">
      <button id="show-trace" data-view="trace" aria-pressed="${view === "trace"}">${text("process")}</button>
      <button id="show-problems" data-view="problems" aria-pressed="${view === "problems"}">${text("problems")}</button>
      <button id="show-results" data-view="results" aria-pressed="${view === "results"}">${text("resultTable")}</button>
    </nav>
    ${content[view]()}
  </div>`;
  if (focus) root.querySelector(focus)?.focus({ preventScroll: false });
}

function goBack() {
  if (view !== "trace") {
    view = "trace";
    render(returnFocus);
  } else if (located) {
    located = false;
    render("#locate");
  } else if (narrowView === "detail" && matchMedia("(max-width: 750px)").matches) {
    narrowView = "list";
    render(`[data-step="${stepIndex}"]`);
  }
}

root.addEventListener("click", event => {
  const button = event.target.closest("button");
  if (!button) return;
  if (button.dataset.view) {
    if (view === "trace") returnFocus = `#${button.id}`;
    view = button.dataset.view;
    render(view === "trace" ? "#show-trace" : "#aux-title");
  } else if (button.dataset.step !== undefined) {
    stepIndex = Number(button.dataset.step);
    narrowView = "detail";
    located = false;
    render(matchMedia("(max-width: 750px)").matches ? "#detail-title" : `[data-step="${stepIndex}"]`);
  } else if (button.id === "locate") {
    located = true;
    render("#detail-title");
  } else if (["back-from-target", "back-to-list", "return-to-trace"].includes(button.id)) {
    goBack();
  }
});

root.addEventListener("change", event => {
  if (event.target.id !== "run-select") return;
  runIndex = Number(event.target.value);
  narrowView = "list";
  chooseDefaultStep();
  render("#run-select");
});

root.addEventListener("keydown", event => {
  if (event.key === "Escape" && event.target.tagName !== "SELECT") goBack();
});

scenarioSelect.addEventListener("change", () => {
  scenario = scenarioSelect.value;
  runIndex = 0;
  view = "trace";
  narrowView = "list";
  chooseDefaultStep();
  render();
});
languageSelect.addEventListener("change", () => {
  language = languageSelect.value;
  updateReviewControls();
  render();
});
sizeSelect.addEventListener("change", () => { document.documentElement.style.fontSize = `${sizeSelect.value}px`; });
sizeSelect.value = params.get("size") === "20" ? "20" : "14";
narrowView = params.get("view") === "detail" ? "detail" : "list";
updateReviewControls();
chooseDefaultStep();
render();
