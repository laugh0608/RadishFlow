using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Common;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

[ComVisible(true)]
[Guid(UnitOperationComIdentity.SimulationContextPlaceholderClassId)]
[ClassInterface(ClassInterfaceType.None)]
[ComDefaultInterface(typeof(ICapeCOSEUtilities))]
public sealed class UnitOperationSimulationContextPlaceholder :
    ICapeIdentification,
    ICapeSimulationContext,
    ICapeDiagnostic,
    ICapeMaterialTemplateSystem,
    ICapeCOSEUtilities
{
    public string ComponentName { get; set; } = "RadishFlow simulation context placeholder";

    public string ComponentDescription { get; set; } =
        "Placeholder returned before a PME simulation context is available.";

    public object MaterialTemplates
    {
        get
        {
            UnitOperationComTrace.Write(nameof(ICapeMaterialTemplateSystem.MaterialTemplates), "get-enter");
            return Array.Empty<string>();
        }
    }

    public object? CreateMaterialTemplate(string materialTemplateName)
    {
        ArgumentNullException.ThrowIfNull(materialTemplateName);
        UnitOperationComTrace.Write(
            nameof(ICapeMaterialTemplateSystem.CreateMaterialTemplate),
            "enter",
            materialTemplateName);
        return null;
    }

    public object NamedValueList
    {
        get
        {
            UnitOperationComTrace.Write(nameof(ICapeCOSEUtilities.NamedValueList), "get-enter");
            return Array.Empty<string>();
        }
    }

    public object NamedValue(string value)
    {
        ArgumentNullException.ThrowIfNull(value);
        UnitOperationComTrace.Write(nameof(ICapeCOSEUtilities.NamedValue), "enter", value);
        return string.Empty;
    }

    public void PopUpMessage(string message)
    {
        UnitOperationComTrace.Write(nameof(ICapeDiagnostic.PopUpMessage), "enter", message);
    }

    public void LogMessage(string message)
    {
        UnitOperationComTrace.Write(nameof(ICapeDiagnostic.LogMessage), "enter", message);
    }
}
