using RadishFlow.CapeOpen.Interop.Common;
using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

[ComVisible(true)]
[Guid(PlaceholderComClassIds.PortCollection)]
[ClassInterface(ClassInterfaceType.None)]
[ComDefaultInterface(typeof(ICapeCollection))]
public sealed class UnitOperationPortCollection
    : UnitOperationPlaceholderCollection<UnitOperationPortPlaceholder>
{
    public UnitOperationPortCollection(
        UnitOperationCollectionDefinition definition,
        IEnumerable<UnitOperationPortPlaceholder> items,
        Action<string, string, string?, object?>? ensureOwnerAccess = null)
        : base(definition, items, ensureOwnerAccess)
    {
    }
}
