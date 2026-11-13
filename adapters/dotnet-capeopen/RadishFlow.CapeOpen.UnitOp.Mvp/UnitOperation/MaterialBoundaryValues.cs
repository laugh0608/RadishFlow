using RadishFlow.CapeOpen.Interop.Errors;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

// Numerical checks at the material boundary, independent of COM argument shapes.
internal static class MaterialBoundaryValues
{
    public const double CompositionTolerance = 1e-10;
    public const double StateRelativeTolerance = 1e-8;

    public static CapeDataException Invalid(string stage, string detail) => new(
        $"{stage}: {detail}", Context(stage));

    public static CapeOpenExceptionContext Context(string stage, string? detail = null) => new(
        "ICapeUnit", "RadishFlow.CapeOpen.UnitOp.Mvp.MaterialBoundary", stage, MoreInfo: detail);

    public static CapeOpenException Propagate(Exception error, string stage, string description, bool equilibrium = false)
    {
        var original = error is CapeOpenException cape
            ? $"{cape.ErrorName}; {cape.Operation}; {cape.MoreInfo}"
            : error.GetType().Name;
        var context = Context(stage, $"{original}; HRESULT=0x{error.HResult:X8}; {error.Message}");
        description += $" [Original HRESULT=0x{error.HResult:X8}]";
        return error.HResult switch
        {
            CapeOpenErrorHResults.ECapeData => new CapeDataException(description, context),
            CapeOpenErrorHResults.ECapeSolvingError => new CapeSolvingException(description, context),
            CapeOpenErrorHResults.ECapeNoImpl => new CapeNoImplementationException(description, context),
            CapeOpenErrorHResults.ECapeInvalidArgument => new CapeInvalidArgumentException(description, context),
            CapeOpenErrorHResults.ECapeBadInvOrder => new CapeBadInvocationOrderException(description, context),
            CapeOpenErrorHResults.ECapeOutOfResources => new CapeOutOfResourcesException(description, context),
            CapeOpenErrorHResults.ECapeTimeOut => new CapeTimeOutException(description, context),
            CapeOpenErrorHResults.ECapeLicenceError => new CapeLicenceErrorException(description, context),
            CapeOpenErrorHResults.ECapeBadCOParameter => new CapeBadCoParameterException(description, context),
            CapeOpenErrorHResults.ECapeFailedInitialisation => new CapeFailedInitialisationException(description, context),
            _ when equilibrium => new CapeSolvingException(description, context),
            _ => new CapeUnknownException(description, context),
        };
    }

    public static double[] Numbers(object? value, string stage)
    {
        // Do not coerce null, strings or booleans into apparently valid material data.
        var values = value switch
        {
            double scalar => new[] { scalar },
            double[] array => array.ToArray(),
            Array array when array.Rank == 1 => array.Cast<object?>().Select(item => item is double number
                ? number : throw Invalid(stage, "Expected a numeric double array.")).ToArray(),
            _ => throw Invalid(stage, "Required numeric property is missing or has the wrong type."),
        };
        if (values.Length == 0 || values.Any(value => !double.IsFinite(value)))
            throw Invalid(stage, "Numeric property must be nonempty and finite.");
        return values;
    }

    public static string[] Strings(object? value, string stage)
    {
        if (value is not Array array || array.Rank != 1)
            throw Invalid(stage, "Required string array is missing.");
        var result = array.Cast<object?>().Select(item => item is string text && !string.IsNullOrWhiteSpace(text)
            ? text : throw Invalid(stage, "Identifiers must be nonempty strings.")).ToArray();
        if (result.Length == 0 || result.Distinct(StringComparer.Ordinal).Count() != result.Length)
            throw Invalid(stage, "Identifiers must be nonempty and unique.");
        return result;
    }

    public static double Scalar(object? value, string stage)
    {
        var values = Numbers(value, stage);
        if (values.Length != 1) throw Invalid(stage, "Expected exactly one value.");
        return values[0];
    }

    public static double[] Composition(IReadOnlyList<double> values, int count, string stage)
    {
        var sum = values.Sum();
        if (count == 0 || values.Count != count || values.Any(x => !double.IsFinite(x) || x < 0) ||
            !double.IsFinite(sum) || Math.Abs(sum - 1) > CompositionTolerance)
            throw Invalid(stage, $"Expected {count} nonnegative mole fractions summing to one (tolerance {CompositionTolerance:G}).");
        return values.Select(x => x / sum).ToArray();
    }

    public static void State(double temperature, double pressure, double flow, string stage)
    {
        if (!double.IsFinite(temperature) || temperature <= 0 || !double.IsFinite(pressure) || pressure <= 0 ||
            !double.IsFinite(flow) || flow < 0)
            throw Invalid(stage, "Temperature and pressure must be positive and finite; molar flow must be finite and nonnegative.");
    }

    public static void Equal(double actual, double expected, string stage)
    {
        if (!double.IsFinite(actual) || Math.Abs(actual - expected) > StateRelativeTolerance * Math.Max(Math.Abs(expected), 1e-12))
            throw Invalid(stage, $"Readback {actual:G17} differs from requested value {expected:G17}.");
    }
}

internal sealed record MaterialCompound(string Id, string? Name = null, string? CasNo = null);

internal static class MaterialCompoundMapping
{
    // Explicit aliases for the shipped examples. Formulae and punctuation folding are not identities.
    private static readonly IReadOnlyDictionary<string, string> KnownAliases = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase)
    {
        ["water"] = "water", ["7732-18-5"] = "water",
        ["ethanol"] = "ethanol", ["64-17-5"] = "ethanol",
        ["methane"] = "methane", ["74-82-8"] = "methane",
        ["ethane"] = "ethane", ["74-84-0"] = "ethane",
    };

    public static string Identity(MaterialCompound compound, string stage)
    {
        if (string.IsNullOrWhiteSpace(compound.Id)) throw MaterialBoundaryValues.Invalid(stage, "Empty compound ID.");
        var recognized = new[] { compound.Id, compound.Name, compound.CasNo }
            .Where(value => value is not null && KnownAliases.ContainsKey(value))
            .Select(value => KnownAliases[value!]).Distinct(StringComparer.Ordinal).ToArray();
        if (recognized.Length > 1) throw MaterialBoundaryValues.Invalid(stage, $"Conflicting identity metadata for `{compound.Id}`.");
        return recognized.Length == 1 ? recognized[0] : compound.Id;
    }

    public static int[] Match(IReadOnlyList<MaterialCompound> source, IReadOnlyList<MaterialCompound> target, string stage)
    {
        var from = source.Select(c => Identity(c, stage)).ToArray();
        var to = target.Select(c => Identity(c, stage)).ToArray();
        if (from.Length == 0 || from.Length != to.Length ||
            from.Distinct(StringComparer.Ordinal).Count() != from.Length ||
            to.Distinct(StringComparer.Ordinal).Count() != to.Length ||
            !from.Order(StringComparer.Ordinal).SequenceEqual(to.Order(StringComparer.Ordinal)))
            throw MaterialBoundaryValues.Invalid(stage,
                $"Compound sets must match one-to-one, including zero fractions. Source=[{string.Join(",", from)}]; target=[{string.Join(",", to)}].");
        return to.Select(id => Array.IndexOf(from, id)).ToArray();
    }
}
