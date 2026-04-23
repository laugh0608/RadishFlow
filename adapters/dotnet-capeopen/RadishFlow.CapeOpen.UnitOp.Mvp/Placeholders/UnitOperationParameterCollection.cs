using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

[ComVisible(true)]
[Guid(PlaceholderComClassIds.ParameterCollection)]
[ClassInterface(ClassInterfaceType.None)]
public sealed class UnitOperationParameterCollection
    : UnitOperationPlaceholderCollection<UnitOperationParameterPlaceholder>
{
    public UnitOperationParameterCollection(
        UnitOperationCollectionDefinition definition,
        IEnumerable<UnitOperationParameterPlaceholder> items,
        Action<string, string, string?, object?>? ensureOwnerAccess = null)
        : base(definition, items, ensureOwnerAccess)
    {
    }
}
