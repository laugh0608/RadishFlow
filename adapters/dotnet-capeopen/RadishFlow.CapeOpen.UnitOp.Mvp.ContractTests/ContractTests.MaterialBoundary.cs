using System.Text.Json.Nodes;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Thermo;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal static partial class ContractTests
{
    public static void MaterialBoundary_RejectsUnsupportedConfiguration(ContractTestContext context)
    {
        var boundary = MaterialBoundaryConfiguration.Read(context.FlowsheetJsonText);
        ContractAssert.Equal("stream-feed", boundary.InputStreamId, "Unique inlet.");
        ContractAssert.Equal("stream-heated", boundary.OutputStreamId, "Unique outlet.");
        var root = JsonNode.Parse(context.FlowsheetJsonText)!;
        var flowsheet = root["document"]!["flowsheet"]!;
        var streams = flowsheet["streams"]!.AsObject();
        var units = flowsheet["units"]!.AsObject();
        var extraStream = streams["stream-feed"]!.DeepClone();
        extraStream["id"] = "extra-feed";
        streams["extra-feed"] = extraStream;
        var extraFeed = units["feed-1"]!.DeepClone();
        extraFeed["id"] = "extra-feed-unit";
        extraFeed["ports"]![0]!["stream_id"] = "extra-feed";
        units["extra-feed-unit"] = extraFeed;
        ContractAssert.Throws<CapeDataException>(() => MaterialBoundaryConfiguration.Read(root.ToJsonString()), "Multiple inlet/outlet rejected.");
        streams.Remove("extra-feed");
        units.Remove("extra-feed-unit");
        streams.Remove("stream-heated");
        ContractAssert.Throws<CapeDataException>(() => MaterialBoundaryConfiguration.Read(root.ToJsonString()), "Missing stream reference rejected.");
        root = JsonNode.Parse(context.FlowsheetJsonText)!;
        root["document"]!["flowsheet"]!["units"]!["heater-1"]!["ports"]!.AsArray().RemoveAt(1);
        root["document"]!["flowsheet"]!["streams"]!.AsObject().Remove("stream-heated");
        ContractAssert.Throws<CapeDataException>(() => MaterialBoundaryConfiguration.Read(root.ToJsonString()), "No outlet rejected.");
        var duplicate = context.FlowsheetJsonText.Replace("\"schemaVersion\": 1", "\"schemaVersion\": 1, \"schemaVersion\": 1");
        ContractAssert.Throws<CapeDataException>(() => MaterialBoundaryConfiguration.Read(duplicate), "Duplicate keys rejected.");
        var oldSample = Path.Combine(Path.GetDirectoryName(context.ManifestPath)!, "..", "..", "..", "flowsheets", "feed-heater-flash-binary-hydrocarbon.rfproj.json");
        var error = ContractAssert.Throws<CapeDataException>(() => MaterialBoundaryConfiguration.Read(File.ReadAllText(oldSample)), "Old two-product sample rejected.");
        ContractAssert.Contains(error.Description, "stream-vapor", "Actual outlet IDs retained.");
        ContractAssert.Throws<CapeDataException>(() => boundary.ValidatePackage("[{\"packageId\":\"p\",\"componentIds\":[\"methane\"]}]", "p"), "Package component loss rejected.");
        context.ConfigureMinimumValidInputs();
        context.FeedPort.Disconnect();
        context.FeedPort.Connect(new IdentityOnlyMaterial());
        var message = string.Empty;
        ContractAssert.False(context.UnitOperation.Validate(ref message), "Identity-only objects cannot validate as material.");
        context.FeedPort.Disconnect();
        context.FeedPort.Connect(ContractThermoMaterial.CreateFeed("Extra zero component",
            ["methane", "ethane", "water"], ["methane", "ethane", "water"], 300, 100000, 1, [0.5, 0.5, 0]));
        ContractAssert.False(context.UnitOperation.Validate(ref message), "Even zero-concentration extra compounds are rejected.");
        AssertCalculationFailure(context);
    }

    public static void MaterialBoundary_MapsCompleteCompoundSets(ContractTestContext context)
    {
        var boundary = MaterialBoundaryConfiguration.Read(context.FlowsheetJsonText);
        var feed = ContractThermoMaterial.CreateFeed("Reordered", ["74-84-0", "74-82-8"], ["ethane", "methane"], 315, 135000, 7, [0.8, 0.2]);
        var state = CapeOpenFeedMaterialReader.Read(CapeOpenMaterialAccess.Select(feed, "Feed", false), boundary);
        for (var i = 0; i < boundary.Compounds.Length; i++)
            ContractAssert.Close(boundary.Compounds[i].Id == "methane" ? 1.4 : 5.6, state.Fractions[i] * state.FlowMolS, 1e-12, "Component molar flow survives reordering.");
        var json = FlowsheetBoundaryFeedMaterialOverlay.Apply(context.FlowsheetJsonText, boundary, state);
        ContractAssert.True(json != context.FlowsheetJsonText, "Calculation uses a copy.");
        ContractAssert.SequenceEqual(new double[] { 0.8, 0.2 }, feed.GetStoredOverallVector("fraction"), "Reading/overlay does not mutate host composition.");
        foreach (var compounds in new MaterialCompound[][]
        {
            [new("methane")], [new("methane"), new("ethane"), new("water")],
            [new("methane"), new("74-82-8")], [new("methane", "ethane"), new("ethane")],
            [new("CH4"), new("ethane")], [new("meth-ane"), new("ethane")],
        })
            ContractAssert.Throws<CapeDataException>(() => MaterialCompoundMapping.Match(compounds, boundary.Compounds, "test"), "Missing, extra, ambiguous and guessed identities rejected.");
        foreach (var fractions in new double[][] { [0.2], [0.2, 0.7], [-0.1, 1.1], [double.NaN, 0.5], [0, 0] })
            ContractAssert.Throws<CapeDataException>(() => MaterialBoundaryValues.Composition(fractions, 2, "test"), "Invalid composition rejected.");
        var corrected = MaterialBoundaryValues.Composition([0.2, 0.8 + 1e-12], 2, "test");
        ContractAssert.Close(1, corrected.Sum(), 1e-15, "Only tiny complete-composition drift normalized.");
        ContractAssert.Throws<CapeDataException>(() => MaterialBoundaryValues.State(double.PositiveInfinity, 1, 1, "test"), "Nonfinite state rejected.");
        ContractAssert.Throws<CapeDataException>(() => MaterialBoundaryValues.State(300, 0, 1, "test"), "Zero pressure rejected.");
        ContractAssert.Throws<CapeDataException>(() => MaterialBoundaryValues.State(300, 1, -1, "test"), "Negative flow rejected.");
        ContractAssert.Throws<CapeDataException>(() => MaterialBoundaryValues.Numbers(null, "test"), "Missing property rejected.");
        ContractAssert.Throws<CapeDataException>(() => MaterialBoundaryValues.Numbers(new object[] { true }, "test"), "No boolean coercion.");
    }

    public static void MaterialBoundary_FailureAndRecovery(ContractTestContext context)
    {
        context.ConfigureMinimumValidInputs();
        var feed = (ContractThermoMaterial)context.FeedPort.connectedObject!;
        var product = (ContractThermoMaterial)context.ProductPort.connectedObject!;
        // Validate inspects configuration and compound identity, not uncomputed feed state.
        feed.FailAt = "GetOverallTPFraction";
        var message = string.Empty;
        ContractAssert.True(context.UnitOperation.Validate(ref message), "Unreadable feed state does not invalidate configuration.");
        foreach (var stage in new[] { "GetOverallTPFraction", "GetOverallProp.totalFlow" })
        {
            feed.FailAt = stage;
            AssertCalculationFailure(context);
            ContractAssert.Equal(0, product.Calls.Count(call => call == "ClearAllProps"), "Unreadable feed never publishes saved inputs.");
        }
        feed.FailAt = null;
        context.UnitOperation.Calculate();
        var initialResult = context.UnitOperation.LastCalculationResult!;
        ContractAssert.Equal("Thermo1.1", initialResult.DeliveredProductThermoVersion, "Protocol source recorded.");
        ContractAssert.Equal("Liquid", initialResult.DeliveredProduct!.Phases.Single().Label, "Delivered phase is host response.");
        ContractAssert.True(initialResult.Streams.Single(s => s.Id == "stream-heated").Phases.All(p => p.Label != "Liquid"), "Native evidence remains separate.");
        ContractAssert.True(product.Calls.IndexOf("ClearAllProps") < product.Calls.IndexOf("CalcEquilibrium"), "Clear precedes equilibrium.");
        ContractAssert.True(product.Calls.IndexOf("CalcEquilibrium") < product.Calls.IndexOf("GetPresentPhases"), "Readback follows equilibrium.");
        foreach (var stage in new[] { "ClearAllProps", "SetPresentPhases", "CheckEquilibriumSpec", "SetOverallProp.fraction", "SetOverallProp.totalFlow", "SetOverallProp.temperature", "SetOverallProp.pressure", "CalcEquilibrium", "GetOverallTPFraction", "GetPresentPhases" })
        {
            product.FailAt = stage;
            var failure = AssertCalculationFailure(context);
            ContractAssert.Contains(failure.Description, "may be partially updated", "Partial publication explicitly reported.");
            ContractAssert.Contains(failure.MoreInfo ?? "", "80004005", "Original HRESULT retained.");
            if (stage == "CalcEquilibrium") ContractAssert.True(failure is CapeSolvingException, "Flash failure is a solving error.");
            product.FailAt = null;
            context.UnitOperation.Calculate();
            ContractAssert.NotNull(context.UnitOperation.LastCalculationResult, "Recovery requires a fresh successful calculation.");
        }
        product.InvalidPhases = true;
        AssertCalculationFailure(context);
        product.InvalidPhases = false;
        product.ReadbackFlowFactor = 2;
        AssertCalculationFailure(context);
        product.ReadbackFlowFactor = 1;
        product.PhaseComposition = [0.5, 0.5];
        AssertCalculationFailure(context);
        product.PhaseComposition = null;
        product.PhaseFraction = 0.9;
        AssertCalculationFailure(context);
        product.PhaseFraction = 1;
        product.SupportsEquilibrium = false;
        AssertCalculationFailure(context);
        product.SupportsEquilibrium = true;
        product.FailAt = "SetOverallProp.fraction";
        product.FailureHResult = CapeOpenErrorHResults.ECapeData;
        ContractAssert.True(AssertCalculationFailure(context) is CapeDataException, "Standard HRESULT category survives COM boundary.");
        product.FailAt = null;
        product.FailureHResult = unchecked((int)0x80004005);
        // The material fixture rejects any attempt to set CAPE_ATEQUILIBRIUM from the PMC.
        feed.SetOverallProp("fraction", "Mole", new double[] { 0.2, 0.8 });
        feed.SetOverallProp("totalFlow", "Mole", new double[] { 9 });
        context.UnitOperation.Calculate();
        var result = context.UnitOperation.LastCalculationResult!;
        ContractAssert.Close(1.8, result.DeliveredProduct!.TotalMolarFlowMolS * result.DeliveredProduct.OverallMoleFractions["methane"], 1e-12, "Recalculation consumes changed feed.");
        var boundary = MaterialBoundaryConfiguration.Read(context.FlowsheetJsonText);
        var zeroResult = result with { Streams = result.Streams.Select(s => s.Id == boundary.OutputStreamId ? s with { TotalMolarFlowMolS = 0 } : s).ToArray() };
        feed.SetOverallProp("totalFlow", "Mole", new double[] { 0 });
        context.UnitOperation.Calculate();
        ContractAssert.Close(0, context.UnitOperation.LastCalculationResult!.DeliveredProduct!.TotalMolarFlowMolS, 0,
            "The native Feed/Heater path also preserves zero flow.");
        var count = product.Calls.Count(call => call == "CalcEquilibrium");
        var zero = CapeOpenMaterialObjectPublisher.Publish(CapeOpenMaterialAccess.Select(product, "Product", true), boundary, zeroResult);
        ContractAssert.Close(0, zero.TotalMolarFlowMolS, 0, "Zero flow stays zero.");
        ContractAssert.Equal(count + 1, product.Calls.Count(call => call == "CalcEquilibrium"), "Zero flow still equilibrates.");
        ContractAssert.Close(1, zero.OverallMoleFractions.Values.Sum(), 1e-15, "Zero flow retains composition.");
        product.Calls.Clear();
        ContractAssert.Throws<CapeDataException>(() => CapeOpenMaterialObjectPublisher.Publish(CapeOpenMaterialAccess.Select(product, "Product", true), boundary, result with { Streams = [] }), "Missing output refused before mutation.");
        ContractAssert.Throws<CapeDataException>(() => CapeOpenMaterialObjectPublisher.Publish(CapeOpenMaterialAccess.Select(product, "Product", true), boundary, result with { Status = "failed" }), "Nonconverged result refused before mutation.");
        ContractAssert.False(product.Calls.Contains("ClearAllProps"), "Candidate failure cannot modify Product.");
        context.ProductPort.Disconnect();
        ContractAssert.Null(context.UnitOperation.LastCalculationResult, "Disconnect invalidates results.");
        context.ProductPort.Connect(product);
        context.UnitOperation.Calculate();
        context.UnitOperation.Terminate();
        ContractAssert.Null(context.UnitOperation.LastCalculationResult, "Terminate clears results.");
    }

    public static void MaterialBoundary_SelectsProtocolBeforeCalls(ContractTestContext context)
    {
        ContractAssert.Equal(new Guid("678C0A9E-7D66-11D2-A67D-00105A42887F"), typeof(ICapeThermoPhases).GUID,
            "Phase catalog uses the standard IID.");
        ContractAssert.SequenceEqual(new[] { "GetNumPhases", "GetPhaseInfo", "GetPhaseList" },
            typeof(ICapeThermoPhases).GetMethods().Select(method => method.Name), "Phase catalog keeps standard vtable order.");
        context.ConfigureMinimumValidInputs();
        context.FeedPort.Disconnect();
        context.ProductPort.Disconnect();
        var feed = new LegacyContractMaterial();
        var product = new LegacyContractMaterial();
        context.FeedPort.Connect(feed);
        context.ProductPort.Connect(product);
        context.UnitOperation.Calculate();
        ContractAssert.Equal("Thermo1.0", context.UnitOperation.LastCalculationResult!.DeliveredProductThermoVersion, "Legacy-only path works.");
        product.State.FailAt = "CalcEquilibrium";
        ContractAssert.True(AssertCalculationFailure(context) is CapeSolvingException, "Legacy flash failure propagates.");
        product.State.FailAt = null;
        feed.State.FailAt = "GetOverallTPFraction";
        AssertCalculationFailure(context);
        feed.State.FailAt = null;
        context.UnitOperation.Calculate();
        context.FeedPort.Disconnect();
        var dual = new DualContractMaterial { FailAt = "GetOverallTPFraction" };
        context.FeedPort.Connect(dual);
        AssertCalculationFailure(context);
        ContractAssert.Equal(0, dual.LegacyCalls, "No retry through 1.0 after 1.1 read failure.");
        dual.FailAt = null;
        context.ProductPort.Disconnect();
        var dualProduct = new DualContractMaterial { FailAt = "CalcEquilibrium" };
        context.ProductPort.Connect(dualProduct);
        AssertCalculationFailure(context);
        ContractAssert.Equal(0, dualProduct.LegacyCalls, "No retry through 1.0 after 1.1 equilibrium failure.");
    }

    private static CapeOpenException AssertCalculationFailure(ContractTestContext context)
    {
        var error = ContractAssert.Throws<CapeOpenException>(() => context.UnitOperation.Calculate(), "Calculation must fail.");
        ContractAssert.Null(context.UnitOperation.LastCalculationResult, "Failure clears current result.");
        ContractAssert.Equal(UnitOperationCalculationReportState.Failure, context.UnitOperation.GetCalculationReportState(), "Failure never reports success.");
        ContractAssert.False(context.ReadSession().Summary.HasCurrentResults, "Failure cannot expose a current session result.");
        return error;
    }
}

internal sealed class IdentityOnlyMaterial : RadishFlow.CapeOpen.Interop.Common.ICapeIdentification
{
    public string ComponentName { get; set; } = "Identity only";
    public string ComponentDescription { get; set; } = "No material interface";
}

internal sealed class LegacyContractMaterial : RadishFlow.CapeOpen.Interop.Common.ICapeIdentification, ICapeThermoMaterialObject
{
    public ContractThermoMaterial State { get; } = new("Legacy material");
    public string ComponentName { get; set; } = "Legacy material";
    public string ComponentDescription { get; set; } = "Thermo 1.0 controlled contract material";
    public object ComponentIds => new[] { "methane", "ethane" };
    public object PhaseIds => new[] { "Overall", "Liquid" };
    public object? GetUniversalConstant(object? props) => throw new NotSupportedException();
    public object? GetComponentConstant(object? props, object? compIds) => throw new NotSupportedException();
    public void CalcProp(object? props, object? phases, string? calcType) => throw new NotSupportedException();
    public object? GetProp(string property, string? phase, object? compIds, string? calcType, string? basis)
    {
        object? result = null;
        if (phase != "Overall")
        {
            if (!State.Equilibrated) throw new InvalidOperationException("Equilibrium not completed.");
            State.GetSinglePhaseProp(property, phase!, basis, ref result);
        }
        else if (property is "temperature" or "pressure")
        {
            double t = 0, p = 0;
            State.GetOverallTPFraction(ref t, ref p, ref result);
            result = new[] { property == "temperature" ? t : p };
        }
        else State.GetOverallProp(property, basis, ref result);
        return result;
    }
    public void SetProp(string property, string? phase, object? compIds, string? calcType, string? basis, object? values) => State.SetOverallProp(property, basis, values);
    public void CalcEquilibrium(string flashType, object? props) => State.CalcEquilibrium(null, null, "unspecified");
    public void SetIndependentVar(object? indVars, object? values) => throw new NotSupportedException();
}

internal sealed class DualContractMaterial : ContractThermoMaterial, ICapeThermoMaterialObject
{
    public DualContractMaterial() : base("Dual material") { }
    public int LegacyCalls { get; private set; }
    private object LegacyCall() { LegacyCalls++; throw new InvalidOperationException("Legacy path must not be called."); }
    public object ComponentIds => LegacyCall();
    public object PhaseIds => LegacyCall();
    public object? GetUniversalConstant(object? props) => LegacyCall();
    public object? GetComponentConstant(object? props, object? compIds) => LegacyCall();
    public void CalcProp(object? props, object? phases, string? calcType) => LegacyCall();
    public object? GetProp(string property, string? phase, object? compIds, string? calcType, string? basis) => LegacyCall();
    public void SetProp(string property, string? phase, object? compIds, string? calcType, string? basis, object? values) => LegacyCall();
    public void CalcEquilibrium(string flashType, object? props) => LegacyCall();
    public void SetIndependentVar(object? indVars, object? values) => LegacyCall();
}
