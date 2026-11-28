"""Replay the Q10 Mixer/Splitter trials with exact operands and binary64 outlets.

Run with python3 -B tests/solver-validation/q10_split_allocation.py.
Only the two component allocations change. Totals, tear inputs, checks, and
the Heater use the existing binary64 material replay.
"""

from dataclasses import dataclass
from enum import Enum
from fractions import Fraction

from q10_component_transfer import Rejected, Transfer, exact_product


class Allocation(Enum):
    INDEPENDENT = "independent"
    SMALL_FIRST = "small"
    LARGE_FIRST = "large"

    def outlets(self, amount, fraction_a):
        # Form both ratios as in the Rust Splitter, including 1 - alpha rounding.
        a = Fraction(fraction_a)
        b = Fraction(1.0 - fraction_a)
        if self is Allocation.INDEPENDENT:
            return float(amount * a), float(amount * b)
        if (a >= b) == (self is Allocation.LARGE_FIRST):
            first = float(amount * a)
            return first, float(amount - Fraction(first))
        first = float(amount * b)
        return float(amount - Fraction(first)), first


@dataclass(frozen=True)
class AllocationTrial(Transfer):
    rule: Allocation
    carry_mix_remainder: bool

    def split_after_mix(self, fresh, assumed, mixed, fraction):
        def split_component(component):
            if self.carry_mix_remainder:
                amount = Fraction(getattr(fresh, component)) + Fraction(getattr(assumed, component))
            else:
                amount = Fraction(getattr(mixed, component))
            return self.rule.outlets(amount, fraction)

        ethane_a, ethane_b = split_component("ethane")
        methane_a, methane_b = split_component("methane")
        return (
            self.stream(mixed.total * fraction, ethane_a, methane_a),
            self.stream(mixed.total * (1.0 - fraction), ethane_b, methane_b),
        )


# High-flow acceptances observed at the Rust CLI. Feed B = 3 passes in all six trials.
ACCEPTED = {
    (Allocation.INDEPENDENT, False): {537790, 537793, 537795, 537797},
    (Allocation.SMALL_FIRST, False): {537789, 537793, 537794},
    (Allocation.LARGE_FIRST, False): set(range(537789, 537798)) - {537792, 537794},
    (Allocation.INDEPENDENT, True): set(),
    (Allocation.SMALL_FIRST, True): set(),
    (Allocation.LARGE_FIRST, True): set(range(537789, 537798)) - {537794},
}


def verify_cases(trial, accepted):
    for feed in (3, *range(537789, 537798)):
        should_pass = feed == 3 or feed in accepted
        try:
            iterations, outlet, _ = trial.run(feed)
        except Rejected as error:
            assert not should_pass, (feed, error)
            stop = error.stage
            print(feed, stop)
        else:
            assert should_pass, feed
            assert all(
                abs(Fraction(value) - target) <= Fraction("1e-9")
                for value, target in zip(outlet[1:], exact_product(feed))
            )
            stop = None
            print(feed, f"accepted after {iterations}")
        if feed in (3, 537793):
            try:
                trial.run(feed, heater_loss=1e-4)
            except Rejected as error:
                assert error.stage == (stop or "heater"), (feed, error)
                print(feed, "with Heater loss:", error.stage)
            else:
                raise AssertionError("injected loss was accepted")


def verify_split_limits():
    amount = Fraction(1)
    for rule in Allocation:
        for split in (0.0, 0.5, 1.0):
            assert rule.outlets(amount, split) == (split, 1.0 - split)
    tiny = 2.0**-54
    assert Allocation.INDEPENDENT.outlets(amount, tiny)[0] == tiny
    assert Allocation.SMALL_FIRST.outlets(amount, tiny)[0] == tiny
    assert Allocation.LARGE_FIRST.outlets(amount, tiny)[0] == 0.0


if __name__ == "__main__":
    verify_split_limits()
    for (rule, carry), accepted in ACCEPTED.items():
        print(f"{rule.value}, carry Mixer remainder = {carry}")
        verify_cases(AllocationTrial(sum_total=False, rule=rule, carry_mix_remainder=carry), accepted)
