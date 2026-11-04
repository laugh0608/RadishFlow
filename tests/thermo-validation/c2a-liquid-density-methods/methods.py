"""Offline liquid-volume candidates; explicit parameters and rejected domains only."""

from __future__ import annotations

import math


class Unavailable(ValueError):
    """A candidate cannot supply an accepted value at the requested input."""


def costald_volume(t: float, tc: float, vstar: float, omega: float) -> float:
    tr = t / tc
    if not 0.25 <= tr <= 0.95:
        raise Unavailable("reduced_temperature_outside_frozen_screening_domain")
    tau = 1 - tr
    reduced = 1 - 1.52816 * tau ** (1 / 3) + 1.43907 * tau ** (2 / 3) - 0.81446 * tau + 0.190454 * tau ** (4 / 3)
    delta = (-0.296123 + 0.386914 * tr - 0.0427258 * tr**2 - 0.0480645 * tr**3) / (tr - 1.00001)
    return vstar * reduced * (1 - omega * delta)


def rackett_volume(t: float, xs: list[float], components: list[dict], gas_constant: float) -> float:
    tc = sum(x * c["critical_temperature_k"] for x, c in zip(xs, components))
    tr = t / tc
    if not 0.25 <= tr <= 0.95:
        raise Unavailable("reduced_temperature_outside_frozen_screening_domain")
    zr = sum(x * c["rackett_z_ra"] for x, c in zip(xs, components))
    mw = sum(x * c["molar_mass_kg_mol"] for x, c in zip(xs, components))
    weighted = sum(x * c["critical_temperature_k"] / (c["critical_pressure_pa"] * c["molar_mass_kg_mol"])
                   for x, c in zip(xs, components))
    return gas_constant * mw * weighted * zr ** (1 + (1 - tr) ** (2 / 7))


def mixture_parameters(xs: list[float], components: list[dict], gas_constant: float) -> tuple[float, float, float, float]:
    volumes = [c["costald_characteristic_volume_m3_mol"] for c in components]
    vstar = (sum(x * v for x, v in zip(xs, volumes))
             + 3 * sum(x * v ** (1 / 3) for x, v in zip(xs, volumes))
             * sum(x * v ** (2 / 3) for x, v in zip(xs, volumes))) / 4
    tc = sum(x * math.sqrt(c["critical_temperature_k"] * v) for x, c, v in zip(xs, components, volumes)) ** 2 / vstar
    omega = sum(x * c["omega_srk"] for x, c in zip(xs, components))
    active = [c for x, c in zip(xs, components) if x > 0]
    pc = (active[0]["critical_pressure_pa"] if len(active) == 1
          else (0.291 - 0.080 * omega) * gas_constant * tc / vstar)
    return tc, vstar, omega, pc


def riedel_pressure(t: float, tc: float, pc: float, omega: float) -> float:
    tr = t / tc
    log_tr = math.log10(tr)
    alpha = 35 - 36 / tr - 96.736 * log_tr + tr**6
    return pc * 10 ** (5.8031817 * log_tr + 0.07608141 * alpha
                       + omega * 4.86601 * (log_tr + 0.03721754 * alpha))


def tait_factor(t: float, p: float, ps: float, tc: float, pc: float, omega: float) -> float:
    tau = 1 - t / tc
    if not 0 < tau < 1 or p < ps:
        raise Unavailable("pressure_below_correlation_psat_or_non_subcritical")
    e = math.exp(4.79594 + 0.250047 * omega + 1.14188 * omega**2)
    b = pc * (-1 - 9.070217 * tau ** (1 / 3) + 62.45326 * tau ** (2 / 3)
              - 135.1102 * tau + e * tau ** (4 / 3))
    if b + ps <= 0:
        raise Unavailable("tait_log_domain")
    factor = 1 - (0.0861488 + 0.0344483 * omega) * math.log1p((p - ps) / (b + ps))
    if not math.isfinite(factor) or factor <= 0:
        raise Unavailable("nonpositive_tait_volume")
    return factor


def evaluate(method: str, t: float, p: float | None, xs: list[float], parameters: dict, saturated: bool = False) -> dict:
    components = parameters["components"]
    if len(xs) != len(components) or any(not math.isfinite(x) or x < 0 for x in xs) or abs(sum(xs) - 1) > 1e-12:
        raise ValueError("Invalid mole fractions; no normalization fallback")
    if not math.isfinite(t) or (p is not None and not math.isfinite(p)):
        raise ValueError("Nonfinite state")
    if not 280 <= t <= 320 or (not saturated and (p is None or not 2e5 <= p <= 1e6)):
        raise Unavailable("outside_original_temperature_or_pressure_domain")
    r = parameters["gas_constant_j_mol_k"]
    tc, vstar, omega, pc = mixture_parameters(xs, components, r)
    if method == "COSTALD-Tait-1982":
        vs = costald_volume(t, tc, vstar, omega)
    elif method == "modified-Rackett-Tait-hybrid":
        vs = rackett_volume(t, xs, components, r)
    else:
        raise ValueError(f"Unknown method: {method}")
    ps = riedel_pressure(t, tc, pc, omega)
    factor = 1.0 if saturated else tait_factor(t, p, ps, tc, pc, omega)
    volume = vs * factor
    if not math.isfinite(volume) or volume <= 0:
        raise Unavailable("nonpositive_liquid_volume")
    mw = sum(x * c["molar_mass_kg_mol"] for x, c in zip(xs, components))
    return {"molar_volume_m3_mol": volume, "density_kg_m3": mw / volume,
            "saturated_volume_m3_mol": vs, "pressure_factor": factor,
            "correlation_psat_pa": ps, "saturation_observation": saturated}
