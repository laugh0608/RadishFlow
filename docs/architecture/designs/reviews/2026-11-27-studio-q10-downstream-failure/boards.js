const get = id => document.getElementById(id);
const labels = {
  zh: {
    settings: "交互稿设置", title: "流程结果", scenario: "工况", language: "语言", size: "字号",
    cases: ["基准成功", "下游失败", "恢复未重跑", "恢复后重跑"],
    current: "Current · 10 股正式结果已发布", failed: "Failed · 本次无正式结果",
    stale: "Stale · 输入已改变，重新运行后查看数值。",
    cause: "依赖块 6 · heater-1 · 单元物料衡算超限",
    origin: (run, rev) => `运行 #${run} · 输入修订 ${rev} · radishflow.partitioned-sm/1`,
    downstream: "下游 heater-1 实际调用 1 · s-product-2",
    block: n => `依赖块 ${n}`, passed: "本块复核通过", diagnostic: "仅诊断",
    counts: n => `迭代求值 ${n} · 独立复核 1`,
    detail: "诊断依据", target: "单元 heater-1 · 关联流股 s-product-2、s-product"
  },
  en: {
    settings: "Review controls", title: "Flow results", scenario: "Case", language: "Language", size: "Text size",
    cases: ["Baseline success", "Downstream failure", "Restored before rerun", "Successful rerun"],
    current: "Current · 10 formal streams published", failed: "Failed · No formal result for this run",
    stale: "Stale · Inputs changed. Run again to view values.",
    cause: "Block 6 · heater-1 · Unit material balance exceeds tolerance",
    origin: (run, rev) => `Run #${run} · Input revision ${rev} · radishflow.partitioned-sm/1`,
    downstream: "Downstream heater-1 · 1 local call · s-product-2",
    block: n => `Block ${n}`, passed: "Block verification passed", diagnostic: "Diagnostic only",
    counts: n => `Iteration evaluations ${n} · Independent verification 1`,
    detail: "Diagnostic evidence", target: "Unit heater-1 · Related streams s-product-2, s-product"
  }
};

function render() {
  const scenario = get("scenario").value;
  const locale = get("language").value;
  const t = labels[locale];
  const failed = scenario === "failed";
  const stale = scenario === "restored";
  document.documentElement.lang = locale === "zh" ? "zh-CN" : "en";
  document.documentElement.style.fontSize = get("size").value + "px";
  document.querySelector("nav").setAttribute("aria-label", t.settings);
  document.querySelector("main").dataset.case = scenario;
  get("title").textContent = t.title;
  get("case-label").textContent = t.scenario;
  get("lang-label").textContent = t.language;
  get("size-label").textContent = t.size;
  Array.from(get("scenario").options).forEach((option, i) => { option.textContent = t.cases[i]; });
  get("status").textContent = stale ? t.stale : failed ? t.failed : t.current;
  get("cause").textContent = failed ? t.cause : "";
  get("cause").hidden = !failed;
  const run = { current: 1, failed: 2, rerun: 3 }[scenario];
  get("origin").textContent = stale ? "" : t.origin(run, run - 1);
  get("origin").hidden = stale;
  get("downstream").textContent = t.downstream;
  get("downstream").hidden = stale;
  const blocks = get("blocks");
  blocks.replaceChildren();
  blocks.hidden = stale;
  if (!stale) {
    for (const [index, count] of (failed ? [72, 27] : [48, 18]).entries()) {
      const item = document.createElement("li");
      const name = document.createElement("strong");
      const verdict = document.createElement("span");
      name.textContent = t.block(index + 4) + " · " + ["s-recycle", "s-recycle-2"][index];
      verdict.textContent = [t.passed, ...(failed ? [t.diagnostic] : []), t.counts(count)].join(" · ");
      item.append(name, verdict);
      blocks.append(item);
    }
  }
  get("details").hidden = !failed;
  if (!failed) get("details").open = false;
  get("detail-label").textContent = t.detail;
  get("detail-target").textContent = t.target;
}
for (const id of ["scenario", "language", "size"]) get(id).addEventListener("change", render);
render();
