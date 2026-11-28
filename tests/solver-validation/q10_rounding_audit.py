"""Attribute Q10 binary64 material differences and compare exact finite iterations.

Run from the repository root with python3 -B tests/solver-validation/q10_rounding_audit.py.
All differences are signed (computed minus exact, or outlet minus inlet), in mol/s.
"""

from fractions import Fraction
from itertools import groupby
from typing import NamedTuple

from q10_component_transfer import ABSOLUTE, SCALED, TOTAL, Rejected, Transfer, exact_product


class Terms(NamedTuple):
    mixer: Fraction
    recycle_branch: Fraction
    product_branch: Fraction
    tear_in_minus_out: Fraction


def rounding_terms(sweep, component):
    """Lift each actual operand to its exact binary value before doing arithmetic."""
    fresh, assumed, mixed, returned, product = (
        Fraction(getattr(stream, component))
        for stream in (sweep.fresh, sweep.assumed, sweep.mixed, sweep.returned, sweep.product)
    )
    alpha = Fraction(sweep.fraction)
    beta = Fraction(1.0 - sweep.fraction)
    assert alpha + beta == 1  # Both split settings used by Q10.
    ideal_mix = fresh + assumed
    ideal_return = mixed * alpha
    ideal_product = mixed * beta
    rounded = tuple(map(float, (ideal_mix, ideal_return, ideal_product)))
    assert (mixed, returned, product) == rounded
    terms = Terms(
        mixed - ideal_mix, returned - ideal_return, product - ideal_product, assumed - returned
    )
    assert sum(terms) == product - fresh
    return terms


def report_binary(feed_b, expected_stage, expected_counts):
    sweeps = []
    try:
        Transfer(sum_total=False).run(feed_b, on_sweep=sweeps.append)
    except Rejected as error:
        assert error.stage == expected_stage
        rejection = error.balance
    else:
        raise AssertionError(f"expected rejection at {expected_stage}")

    counts = []
    ethane_change = Fraction(0)
    for region, records in groupby(sweeps, key=lambda sweep: sweep.region):
        runs = list(records)
        assert [s.evaluation for s in runs] == [*range(1, len(runs)), "verification"]
        counts.append(len(runs) - 1)
        for component in ("ethane", "methane"):
            first_rounding = {}
            for sweep in runs:
                terms = rounding_terms(sweep, component)
                for name, value in zip(Terms._fields[:3], terms[:3]):
                    if value and name not in first_rounding:
                        first_rounding[name] = (sweep.evaluation, float(value))
            # The last sweep is the independent verification that supplied the outlet.
            assert terms.tear_in_minus_out == 0
            change = sum(terms)
            print(f"  region {region} {component}, first rounding (iteration, error): {first_rounding}")
            print(f"    verification {dict(zip(Terms._fields, map(float, terms)))}, change {float(change)}")
            if component == "ethane":
                ethane_change += change
    assert tuple(counts) == expected_counts
    assert -ethane_change == rejection.absolute > ABSOLUTE
    assert rejection.scaled <= SCALED and rejection.total <= TOTAL
    print(f"  {counts} candidate iterations; rejected at {expected_stage}, {rejection}")


def exact_finite_iterations(feed_b):
    """Zero initial tear, decimal feeds and exact split ratios, with the same stop rule."""
    absolute, scaled, total = (Fraction(str(x)) for x in (ABSOLUTE, SCALED, TOTAL))
    feeds = exact_product(feed_b)
    fresh = feeds
    counts = []
    for region, alpha in enumerate((Fraction(3, 5), Fraction(1, 4)), 1):
        scale = sum(fresh)
        # For each component f, R_k - R_(k-1) = f * alpha**k.
        for k in range(1, 501):
            residual = max(fresh) * alpha**k
            if residual <= absolute and residual / scale <= scaled:
                break
        else:
            raise AssertionError("exact recycle iteration budget exhausted")
        counts.append(k)
        # One extra sweep supplies the verified product P_(k+1) = f * (1 - alpha**(k+1)).
        deficit = tuple(f * alpha**(k + 1) for f in fresh)
        assert max(deficit) <= absolute and max(deficit) / scale <= scaled
        assert sum(deficit) <= total
        fresh = tuple(f - d for f, d in zip(fresh, deficit))
        print(f"  exact region {region}: {k} candidates, ethane change {float(-deficit[0]):.17g}")
    differences = tuple(product - inlet for product, inlet in zip(fresh, feeds))
    maximum = max(map(abs, differences))
    assert counts == [71, 26]
    assert maximum <= absolute and maximum / sum(feeds) <= scaled and abs(sum(differences)) <= total
    print(f"  exact overall changes (ethane, methane): {tuple(map(float, differences))}")


if __name__ == "__main__":
    for feed, stage, counts in ((537792, "overall", (72, 27)), (537794, "region 1 balance", (71,))):
        print(f"Feed B {feed} mol/s")
        report_binary(feed, stage, counts)
        exact_finite_iterations(feed)
