"""Replay Q10 material arithmetic in binary64 without loading the product solver.

Component order is ethane, methane. The two feeds, splits and zero tear guesses
match fixed-tp-q7-serial-recycle.rfproj.json. Only material flows are evaluated.
"""

from dataclasses import dataclass
from fractions import Fraction
from typing import NamedTuple


ABSOLUTE = 1e-10  # mol/s per component
SCALED = 1e-10
TOTAL = 2 * ABSOLUTE


class Stream(NamedTuple):
    total: float
    ethane: float
    methane: float


class Balance(NamedTuple):
    """Absolute component residual (mol/s), scaled residual, total residual (mol/s)."""

    absolute: float
    scaled: float
    total: float


class Rejected(Exception):
    def __init__(self, stage, balance):
        super().__init__(stage)
        self.stage = stage
        self.balance = balance


def check(stage, inputs, outputs, scale):
    # Match the frozen checks: accumulate each side, then subtract.
    differences = [
        abs(sum(s[i] for s in outputs) - sum(s[i] for s in inputs))
        for i in range(3)
    ]
    absolute = max(differences[1:])
    balance = Balance(absolute, absolute / scale, differences[0])
    if balance.absolute > ABSOLUTE or balance.scaled > SCALED or balance.total > TOTAL:
        raise Rejected(stage, balance)
    return balance


@dataclass(frozen=True)
class Transfer:
    sum_total: bool

    def stream(self, total, ethane, methane):
        return Stream(ethane + methane if self.sum_total else total, ethane, methane)

    def mix(self, a, b):
        return self.stream(a.total + b.total, a.ethane + b.ethane, a.methane + b.methane)

    def split(self, inlet, fraction):
        return self.stream(
            inlet.total * fraction,
            inlet.ethane * fraction,
            inlet.methane * fraction,
        )

    def region(self, index, fresh, fraction):
        recycle = Stream(0.0, 0.0, 0.0)

        def sweep(tear, evaluation):
            # The tear specifies component rates; its inlet total is their sum.
            assumed = Stream(tear.ethane + tear.methane, tear.ethane, tear.methane)
            mixed = self.mix(fresh, assumed)
            check(f"region {index} mixer {evaluation}", [fresh, assumed], [mixed], fresh.total)
            returned = self.split(mixed, fraction)
            product = self.split(mixed, 1.0 - fraction)
            check(f"region {index} splitter {evaluation}", [mixed], [returned, product], fresh.total)
            residual = max(
                abs(returned.ethane - assumed.ethane),
                abs(returned.methane - assumed.methane),
            )
            return returned, product, residual

        for iteration in range(1, 501):
            recycle, _, residual = sweep(recycle, iteration)
            if residual <= ABSOLUTE and residual / fresh.total <= SCALED:
                _, product, residual = sweep(recycle, "verification")
                if residual > ABSOLUTE or residual / fresh.total > SCALED:
                    raise ValueError("independent verification failed")
                balance = check(f"region {index} balance", [fresh], [product], fresh.total)
                return product, iteration, balance
        raise ValueError("iteration budget exhausted")

    def run(self, feed_b, heater_loss=0.0):
        a = Stream(2.0, 2.0 * 0.2, 2.0 * 0.8)
        b = Stream(feed_b, feed_b * 0.8, feed_b * 0.2)
        scale = a.total + b.total
        fresh = self.mix(a, b)
        check("fresh mixer", [a, b], [fresh], scale)
        bleed, first, first_balance = self.region(1, fresh, 0.6)
        inlet, second, second_balance = self.region(2, bleed, 0.25)
        outlet = self.stream(inlet.total - heater_loss, inlet.ethane - heater_loss, inlet.methane)
        heater = check("heater", [inlet], [outlet], scale)
        overall = check("overall", [a, b], [outlet], scale)
        return (first, second), outlet, (first_balance, second_balance, heater, overall)


def exact_product(feed_b):
    # Decimal feed specifications, independently of recycle iteration and splits.
    flow = Fraction(str(feed_b))
    return (2 + 4 * flow) / 5, (8 + flow) / 5


def main():
    for sum_total in (True, False):
        method = "sum component rates" if sum_total else "carry unit total"
        cases = [(value, 0.0) for value in (3, *range(537789, 537798))]
        # Observed with the frozen Rust trial. These expectations concern this replay.
        accepted_feeds = {3} if sum_total else {3, 537790, 537793, 537795, 537797}
        if not sum_total:
            cases.extend([(3, 1e-4), (537793, 1e-4)])
        for feed_b, loss in cases:
            label = f"{method}, feed B {feed_b}, Heater loss {loss:g} mol/s"
            try:
                iterations, product, balances = Transfer(sum_total).run(feed_b, loss)
            except Rejected as error:
                if (not loss and feed_b in accepted_feeds) or (loss and error.stage != "heater"):
                    raise AssertionError(f"{label}: unexpected rejection at {error}") from error
                print(f"{label}: rejected at {error}, {error.balance}")
                continue
            if loss or feed_b not in accepted_feeds:
                raise AssertionError(f"{label}: unexpected acceptance")
            differences = [
                abs(Fraction(n) - expected)
                for n, expected in zip((product.ethane, product.methane), exact_product(feed_b))
            ]
            if max(differences) > Fraction("1e-9"):
                raise AssertionError("accepted product exceeds the analytical comparison tolerance")
            print(f"{label}: accepted after {iterations}, product {product}, balances {balances}")


if __name__ == "__main__":
    main()
