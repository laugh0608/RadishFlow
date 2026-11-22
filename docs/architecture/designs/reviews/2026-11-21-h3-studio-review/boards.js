const params = new URLSearchParams(location.search);
const scenes = {
  opened: ["打开后待复算", "Opened · not yet rated", "benchmark", 0, false],
  pass: ["40 mol/s · 全部达标", "40 mol/s · all criteria met", "benchmark", 0, true],
  limit: ["50 mol/s · 环隙压降超限", "50 mol/s · annulus limit exceeded", "benchmark", 1, true],
  missing: ["缺项 · 有效长度", "Missing input · active length", "missing", 0, true],
  domain: ["越域 · 冷侧入口绝压", "Out of range · cold inlet pressure", "domain", 0, true],
};
const board = document.querySelector("#board");
const closeButton = document.querySelector("#close-review");
const sceneControl = document.querySelector("#scene");
const localeControl = document.querySelector("#language");
const sizeControl = document.querySelector("#size");
const announcer = document.querySelector("#announcer");

let locale = params.get("lang") === "en" ? "en" : "zh";
let scene;
let project;
let selected;
let results;
let closed = false;
let diagnosisField = null;

const t = (zh, en) => locale === "zh" ? zh : en;
const escape = value => String(value).replace(/[&<>"']/g, ch => ({
  "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
})[ch]);
const fmt = (value, digits = 2) =>
  Number(value).toLocaleString(t("zh-CN", "en-US"), { maximumFractionDigits: digits });
const unit = (value, suffix, digits) => value == null
  ? t("未填写", "Missing")
  : `${fmt(value, digits)} ${suffix}`;
const caption = item => `${item.record.object.tag} · ${t("水工况", "Water")}`;
const caseKey = item => `${item.record.object.task_id}/${item.record.object.case_id}`;
const flowLabel = item => `${t("冷侧", "Cold")} ${unit(item.record.case.cold.inlet.molar_flow_mol_s, "mol/s")}`;
const m = (value, suffix, digits) => escape(unit(value, suffix, digits));

function pill(label, tone, mark) {
  return `<span class="pill" data-tone="${tone}"><span aria-hidden="true" class="mark">${mark}</span>${label}</span>`;
}

function reviewState(result) {
  if (!result) return [t("待复算", "Not yet rated"), "neutral", "○"];
  if (result.status === "failed") return [t("复算失败", "Rating failed"), "bad", "×"];
  return [t("当前结果", "Current result"), "current", "●"];
}

function openScene(name, prime = true) {
  scene = Object.hasOwn(scenes, name) ? name : "opened";
  const [, , key, index, showResult] = scenes[scene];
  project = reviewEvidence[key];
  selected = index;
  results = new Map();
  if (prime && showResult) {
    const item = project.cases[selected];
    results.set(caseKey(item), item.result);
  }
  diagnosisField = null;
  closed = false;
  render();
}

function render() {
  document.documentElement.lang = t("zh-CN", "en");
  localeControl.value = locale;
  document.querySelector("#review-label").textContent = t("H3 局部画板 · 待确认", "H3 board · for review");
  document.querySelector("#scene-label").textContent = t("场景", "Scene");
  document.querySelector("#language-label").textContent = t("语言", "Language");
  document.querySelector("#size-label").textContent = t("基准字号", "Type size");
  sceneControl.innerHTML = Object.entries(scenes).map(([key, labels]) =>
    `<option value="${key}" ${key === scene ? "selected" : ""}>${t(...labels.slice(0, 2))}</option>`).join("");
  closeButton.hidden = closed;
  closeButton.textContent = t("返回工作区", "Back to workspace");
  if (closed) {
    board.innerHTML = `<section class="panel after" aria-labelledby="closed-heading">
      <h1 id="closed-heading" tabindex="-1">${t("已返回工作区", "Returned to workspace")}</h1>
      <button type="button" class="button primary" id="reopen">${t("重新打开示例", "Reopen example")}</button>
    </section>`;
    board.querySelector("#reopen").addEventListener("click", () => {
      openScene(scene, false);
      board.querySelector("#review-heading").focus();
    });
    return;
  }

  const item = project.cases[selected];
  board.innerHTML = `
    <section class="page-head" aria-labelledby="review-heading">
      <div class="head-line">
        <span class="pill readonly">${t("工程只读", "Read-only project")}</span>
        <div class="file-meta"><span>${escape(project.filename)}</span><span class="dot">v${project.file.project_schema_version}</span></div>
      </div>
      <h1 id="review-heading" tabindex="-1">${t("同心套管逆流校核", "Concentric-tube counterflow rating")}</h1>
      <p class="subheading">${t("基于已保存输入复算；结果仅在本次复核中保留。", "Rate the saved inputs. Results are kept for this review only.")}</p>
    </section>
    <div class="workspace-grid ${project.cases.length > 1 ? "" : "single-case"}">
      <nav class="case-menu panel" aria-labelledby="case-menu-title">
        <h2 id="case-menu-title">${t("工况", "Cases")}</h2>
        ${caseButtons()}
      </nav>
      <div class="stack">
        <section class="case-head panel" aria-labelledby="case-title">
          <div class="heading-text">
            <h2 id="case-title">${escape(caption(item))}</h2>
            <div class="small">${escape(flowLabel(item))} · ${t("定物性液相", "Constant-property liquids")}</div>
          </div>
          <div class="actions"><button type="button" id="rate" class="button primary">${t("复算当前工况", "Rate selected case")}</button></div>
        </section>
        <div id="outcome-slot"></div>
        ${savedInputs(item)}
        ${trace(item)}
      </div>
    </div>`;

  board.querySelectorAll(".case-button").forEach(button => {
    button.addEventListener("click", () => selectCase(Number(button.dataset.case)));
  });
  board.querySelector("#rate").addEventListener("click", rateSelected);
  renderOutcome();
}

function caseButtons() {
  return project.cases.map((item, index) => `
    <button class="case-button" type="button" data-case="${index}" aria-pressed="${index === selected}">
      <strong>${escape(caption(item))}</strong>
      <span class="small numeric">${escape(flowLabel(item))}</span>
      ${pill(...reviewState(results.get(caseKey(item))))}
    </button>`).join("");
}

function selectCase(index) {
  selected = index;
  diagnosisField = null;
  render();
  board.querySelector(`.case-button[data-case="${selected}"]`).focus();
}

function rateSelected() {
  const item = project.cases[selected];
  results.set(caseKey(item), item.result);
  if (diagnosisField) {
    board.querySelector(`#${diagnosisField}`).removeAttribute("aria-invalid");
    board.querySelector("#return-diagnostic").hidden = true;
  }
  diagnosisField = null;
  renderOutcome();
  const state = pill(...reviewState(item.result));
  const menu = board.querySelector(`.case-button[data-case="${selected}"] .pill`);
  if (menu) menu.outerHTML = state;
  announcer.textContent = `${caption(item)} · ${reviewState(item.result)[0]}`;
}

function renderOutcome() {
  const item = project.cases[selected];
  const result = results.get(caseKey(item));
  const slot = board.querySelector("#outcome-slot");
  let body;
  if (!result) {
    body = `<p class="message">${t("选择「复算当前工况」以查看热工结果和压降准则。", "Rate the selected case to review thermal results and pressure-drop criteria.")}</p>`;
  } else if (result.status === "failed") {
    body = diagnostic(item);
  } else {
    body = current(result, item.record.case.limits);
  }
  const exceeded = result?.status === "current"
    ? Object.values(result.checks).filter(status => status === "failed").length
    : 0;
  slot.innerHTML = `<section class="outcome panel ${result ? "" : "empty"}" aria-labelledby="result-label">
    <div class="status-line">
      <h3 id="result-label">${t("校核结果", "Rating results")}</h3>
      <div class="status-badges">${pill(...reviewState(result))}
        ${exceeded ? pill(t(`${exceeded} 项超限`, `${exceeded} ${exceeded === 1 ? "limit" : "limits"} exceeded`), "caution", "!") : ""}
      </div>
    </div>
    ${body}
  </section>`;
  slot.querySelector("#locate-diagnostic")?.addEventListener("click", () => focusDiagnosticField(item));
  const calculation = board.querySelector("#calculation-evidence");
  calculation.hidden = result?.status !== "current";
  calculation.innerHTML = result?.status === "current" ? calculationEvidence(result) : "";
}

function current(result, limits) {
  const pressure = t("绝压", "Abs. pressure");
  const drop = t("沿程压降", "Pressure drop");
  const hot = result.hot;
  const cold = result.cold;
  return `<dl class="indicators">
    ${metric(t("热负荷 Q", "Heat duty Q"), result.heat_duty_w, "W")}
    ${metric(t("总传热能力 UA", "Conductance UA"), result.ua_w_per_k, "W/K")}
  </dl>
  <div class="result-grid">
    <section class="data-box" aria-labelledby="outlets-title">
      <h3 id="outlets-title">${t("两侧出口", "Outlet conditions")}</h3>
      <table class="numeric" aria-labelledby="outlets-title">
        <thead><tr><th scope="col">${t("指标", "Measure")}</th><th scope="col">${t("内管 · 热侧", "Hot tube")}</th><th scope="col">${t("环隙 · 冷侧", "Cold annulus")}</th></tr></thead>
        <tbody>
          ${tableRow(t("温度", "Temperature"), hot.outlet.temperature_k, cold.outlet.temperature_k, "K")}
          ${tableRow(pressure, hot.outlet.pressure_pa, cold.outlet.pressure_pa, "Pa")}
          ${tableRow(drop, hot.passage.pressure_drop_pa, cold.passage.pressure_drop_pa, "Pa")}
        </tbody>
      </table>
    </section>
    <section class="data-box" aria-labelledby="criteria-title">
      <h3 id="criteria-title">${t("校核准则", "Acceptance criteria")}</h3>
      ${criterion(t("最低热负荷", "Min. heat duty"), limits.min_heat_duty_w, "W", "≥", result.checks.heat_duty)}
      ${criterion(t("内管压降上限", "Max. tube pressure drop"), limits.max_hot_pressure_drop_pa, "Pa", "≤", result.checks.hot_pressure_drop)}
      ${criterion(t("环隙压降上限", "Max. annulus pressure drop"), limits.max_cold_pressure_drop_pa, "Pa", "≤", result.checks.cold_pressure_drop)}
    </section>
  </div>`;
}

function metric(label, value, suffix) {
  return `<div class="metric"><dt>${label}</dt><dd class="numeric">${fmt(value)} <small>${suffix}</small></dd></div>`;
}
function tableRow(label, hot, cold, suffix) {
  const value = number => `<span class="table-number">${fmt(number)}</span> <span class="table-unit">${suffix}</span>`;
  return `<tr><th scope="row">${label}</th><td>${value(hot)}</td><td>${value(cold)}</td></tr>`;
}
function criterion(label, value, suffix, op, status) {
  const badges = {
    passed: [t("通过", "Pass"), "good", "✓"],
    failed: [t("超限", "Limit exceeded"), "caution", "!"],
    not_specified: [t("未指定", "Not specified"), "neutral", "○"],
  };
  return `<div class="criterion"><span>${label}</span>${pill(...badges[status])}${value == null ? "" : `<small class="numeric">${op} ${m(value, suffix)}</small>`}</div>`;
}

function diagnostic(item) {
  const { diagnostic: finding } = item.result;
  const missing = finding.code.endsWith(".missing_input");
  const domain = item.record.case.cold.properties.liquid_domain;
  const pressure = item.record.case.cold.inlet.pressure_pa;
  return `<section class="diagnostic" aria-labelledby="diagnostic-heading">
    <h3 id="diagnostic-heading">${missing ? t("缺少有效换热长度", "Active length is missing") : t("冷侧入口超出单液相适用域", "Cold inlet is outside the liquid domain")}</h3>
    <p>${missing ? t("保存工况未填写有效换热长度（L）。",
                        "Active heat-transfer length (L) is blank in the saved case.")
      : t(`保存的冷侧入口绝压为 ${unit(pressure, "Pa")}；声明范围为 ${fmt(domain.min_pressure_pa)}–${unit(domain.max_pressure_pa, "Pa")}。`,
          `The saved cold inlet is at ${unit(pressure, "Pa")} absolute. The declared range is ${fmt(domain.min_pressure_pa)}–${unit(domain.max_pressure_pa, "Pa")}.`)}</p>
    <button type="button" id="locate-diagnostic" class="button inline">${t("查看输入字段", "Show input field")}</button>
    <code>${finding.code.split(".").map(escape).join(".<wbr>")}</code>
  </section>`;
}

function focusDiagnosticField(item) {
  const missing = item.result.diagnostic.code.endsWith(".missing_input");
  diagnosisField = missing ? "active-length" : "cold-pressure";
  board.querySelector("#inputs").open = true;
  board.querySelector("#return-diagnostic").hidden = false;
  const field = board.querySelector(`#${diagnosisField}`);
  field.setAttribute("aria-invalid", "true");
  field.scrollIntoView({ block: "center", behavior: "instant" });
  field.focus({ preventScroll: true });
}

function returnToDiagnostic() {
  if (!diagnosisField) return false;
  board.querySelector(`#${diagnosisField}`)?.removeAttribute("aria-invalid");
  diagnosisField = null;
  board.querySelector("#return-diagnostic").hidden = true;
  const action = board.querySelector("#locate-diagnostic");
  if (action) {
    action.scrollIntoView({ block: "center", behavior: "instant" });
    action.focus({ preventScroll: true });
  }
  return true;
}

function detailRow(label, value, id) {
  const target = id ? ` id="${id}" tabindex="-1"` : "";
  return `<div class="detail-row"${target}><dt>${label}</dt><dd class="numeric">${value}</dd></div>`;
}

function savedInputs(item) {
  const { geometry, case: data } = item.record;
  const groups = [
    [t("给定几何", "Given geometry"), [
      [t("有效换热长度 L", "Active length L"), m(geometry.active_length_m, "m", 5), "active-length"],
      [t("内管内径", "Tube inner diameter"), m(geometry.inner_tube_inner_diameter_m, "m", 5)],
      [t("内管外径", "Tube outer diameter"), m(geometry.inner_tube_outer_diameter_m, "m", 5)],
      [t("外管内径", "Outer pipe inner diameter"), m(geometry.outer_tube_inner_diameter_m, "m", 5)],
      [t("管壁导热系数", "Wall conductivity"), m(geometry.wall_thermal_conductivity_w_per_m_k, "W/(m·K)")],
    ]],
    ...[["hot", t("热侧入口 · 内管", "Hot inlet · tube")], ["cold", t("冷侧入口 · 环隙", "Cold inlet · annulus")]]
      .map(([side, label]) => [label, [
        [t("温度", "Temperature"), m(data[side].inlet.temperature_k, "K")],
        [t("绝对压力", "Absolute pressure"), m(data[side].inlet.pressure_pa, "Pa"), side === "cold" ? "cold-pressure" : null],
        [t("摩尔流量", "Molar flow"), m(data[side].inlet.molar_flow_mol_s, "mol/s")],
      ]]),
    [t("限值", "Limits"), [
      [t("最低热负荷", "Min. heat duty"), m(data.limits.min_heat_duty_w, "W")],
      [t("内管压降上限", "Max. tube pressure drop"), m(data.limits.max_hot_pressure_drop_pa, "Pa")],
      [t("环隙压降上限", "Max. annulus pressure drop"), m(data.limits.max_cold_pressure_drop_pa, "Pa")],
    ]],
    ...[["hot", t("热侧定物性和温压域", "Hot properties and domain")], ["cold", t("冷侧定物性和温压域", "Cold properties and domain")]]
      .map(([side, label]) => {
        const props = data[side].properties;
        const domain = props.liquid_domain;
        return [label, [
          [t("摩尔质量", "Molar mass"), m(props.molar_mass_kg_per_mol, "kg/mol", 8)],
          [t("定压摩尔热容", "Molar heat capacity"), m(props.molar_heat_capacity_j_per_mol_k, "J/(mol·K)", 6)],
          [t("密度", "Density"), m(props.density_kg_per_m3, "kg/m³", 6)],
          [t("动力黏度", "Dynamic viscosity"), m(props.dynamic_viscosity_pa_s, "Pa·s", 12)],
          [t("导热系数", "Thermal conductivity"), m(props.thermal_conductivity_w_per_m_k, "W/(m·K)", 9)],
          [t("允许温度", "Temperature range"), `${fmt(domain.min_temperature_k)}–${m(domain.max_temperature_k, "K")}`],
          [t("允许绝压", "Absolute pressure range"), `${fmt(domain.min_pressure_pa)}–${m(domain.max_pressure_pa, "Pa")}`],
        ]];
      }),
  ];

  return `<details class="panel detail" id="inputs">
    <summary>${t("已保存输入", "Saved inputs")}</summary>
    <div class="detail-body">
      <button type="button" id="return-diagnostic" class="button inline" hidden>${t("返回诊断", "Back to diagnostic")}</button>
      <div class="detail-grid">${groups.map(([label, fields]) => `
        <div><h3>${label}</h3><dl>${fields.map(row => detailRow(...row)).join("")}</dl></div>`).join("")}</div>
    </div>
  </details>`;
}

function trace(item) {
  const { object, sources } = item.record;
  const rev = object.revisions;
  const method = object.method;
  return `<details class="panel detail" id="trace">
    <summary>${t("来源与计算依据", "Sources and calculation details")}</summary>
    <div class="detail-body trace">
      <dl>
        ${detailRow(t("方法", "Method"), `<code>${escape(method.id)} / ${method.version}</code>`)}
        ${detailRow(t("工程 SHA-256", "Project SHA-256"), `<code>${escape(project.file.file_sha256)}</code>`)}
        ${detailRow(t("任务 / 工况", "Task / case"), `<code>${escape(caseKey(item))}</code>`)}
        ${detailRow(t("任务 / 几何 / 工况修订", "Task / geometry / case revisions"), `${rev.task} / ${rev.geometry} / ${rev.case}`)}
      </dl>
      <h3>${t("已引用的来源", "Referenced sources")}</h3>
      <ul class="sources">${sources.map(source => `<li>${escape(source.data.title)} · ${escape(source.data.locator)}</li>`).join("")}</ul>
      <section id="calculation-evidence" hidden></section>
    </div>
  </details>`;
}

function calculationEvidence(result) {
  return `<h3>${t("热工与水力依据", "Thermal and hydraulic details")}</h3><dl>
    ${detailRow(t("LMTD", "LMTD"), m(result.lmtd_k, "K", 4))}
    ${detailRow(t("内 / 外换热面积", "Inner / outer transfer area"), `${m(result.inner_heat_transfer_area_m2, "m²", 5)} / ${m(result.outer_heat_transfer_area_m2, "m²", 5)}`)}
    ${detailRow(t("内管 / 环隙 Re", "Tube / annulus Re"), `${fmt(result.hot.passage.reynolds_number)} / ${fmt(result.cold.passage.reynolds_number)}`)}
    ${detailRow(t("内管 / 环隙 Nu", "Tube / annulus Nu"), `${fmt(result.hot.passage.nusselt_number)} / ${fmt(result.cold.passage.nusselt_number)}`)}
    ${detailRow(t("整体显热残差", "Overall sensible-heat residual"), `${result.overall_sensible_energy_residual_w.toExponential(2)} W`)}
    ${detailRow(t("Q − UA × LMTD 残差", "Q − UA × LMTD residual"), `${result.lmtd_heat_duty_residual_w.toExponential(2)} W`)}
  </dl>`;
}

function closeReview() {
  closed = true;
  results.clear();
  diagnosisField = null;
  render();
  board.querySelector("#closed-heading").focus();
}

board.addEventListener("click", event => {
  if (event.target.id === "return-diagnostic") returnToDiagnostic();
});
closeButton.addEventListener("click", closeReview);
document.addEventListener("keydown", event => {
  if (event.key === "Escape" && !closed && !event.defaultPrevented && event.target.tagName !== "SELECT") {
    if (!returnToDiagnostic()) closeReview();
    event.preventDefault();
  }
});
sceneControl.addEventListener("change", event => openScene(event.target.value));
localeControl.addEventListener("change", event => { locale = event.target.value; render(); });
sizeControl.addEventListener("change", event => { document.documentElement.dataset.size = event.target.value; });
sizeControl.value = params.get("size") === "20" || (!params.has("size") && window.innerWidth <= 490) ? "20" : "14";
document.documentElement.dataset.size = sizeControl.value;
document.querySelector("#review-controls").hidden = params.get("preview") === "1";
openScene(params.get("scene"));
