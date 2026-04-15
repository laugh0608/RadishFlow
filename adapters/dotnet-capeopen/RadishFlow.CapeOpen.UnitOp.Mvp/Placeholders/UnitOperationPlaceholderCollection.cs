using System.Collections;
using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

public sealed class UnitOperationPlaceholderCollection<T> : ICapeIdentification, ICapeCollection, IReadOnlyList<T>
    where T : class, ICapeIdentification
{
    private const string InterfaceName = nameof(ICapeCollection);
    private const string ItemOperation = "Item";
    private readonly Action<string, string, string?, object?>? _ensureOwnerAccess;
    private readonly IReadOnlyList<T> _items;

    public UnitOperationPlaceholderCollection(
        string componentName,
        string componentDescription,
        IEnumerable<T> items,
        Action<string, string, string?, object?>? ensureOwnerAccess = null)
    {
        ComponentName = componentName;
        ComponentDescription = componentDescription;
        _ensureOwnerAccess = ensureOwnerAccess;
        _items = items.ToArray();
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }

    public int Count
    {
        get
        {
            EnsureOwnerAccess("Count");
            return _items.Count;
        }
    }

    public T this[int index]
    {
        get
        {
            EnsureOwnerAccess(ItemOperation, index + 1);
            return _items[index];
        }
    }

    object ICapeCollection.Item(object index)
    {
        EnsureOwnerAccess(ItemOperation, index);

        if (index is string name)
        {
            return FindByName(name);
        }

        if (TryGetOneBasedIndex(index, out var oneBasedIndex))
        {
            return GetByOneBasedIndex(oneBasedIndex);
        }

        throw new CapeInvalidArgumentException(
            $"Collection `{ComponentName}` only accepts a 1-based integer index or component name.",
            CreateContext(ItemOperation, index));
    }

    int ICapeCollection.Count()
    {
        EnsureOwnerAccess("Count");
        return _items.Count;
    }

    public IEnumerator<T> GetEnumerator()
    {
        EnsureOwnerAccess("GetEnumerator");
        return _items.GetEnumerator();
    }

    IEnumerator IEnumerable.GetEnumerator()
    {
        return GetEnumerator();
    }

    private T FindByName(string name)
    {
        if (string.IsNullOrWhiteSpace(name))
        {
            throw new CapeInvalidArgumentException(
                $"Collection `{ComponentName}` requires a non-empty component name.",
                CreateContext(ItemOperation, name));
        }

        var item = _items.FirstOrDefault(candidate =>
            string.Equals(candidate.ComponentName, name, StringComparison.OrdinalIgnoreCase));
        if (item is not null)
        {
            return item;
        }

        throw new CapeInvalidArgumentException(
            $"Collection `{ComponentName}` does not contain an item named `{name}`.",
            CreateContext(ItemOperation, name));
    }

    private T GetByOneBasedIndex(int oneBasedIndex)
    {
        if (oneBasedIndex >= 1 && oneBasedIndex <= _items.Count)
        {
            return _items[oneBasedIndex - 1];
        }

        throw new CapeInvalidArgumentException(
            $"Collection `{ComponentName}` index `{oneBasedIndex}` is out of range. Expected 1..{_items.Count}.",
            CreateContext(ItemOperation, oneBasedIndex));
    }

    private static bool TryGetOneBasedIndex(object index, out int oneBasedIndex)
    {
        switch (index)
        {
            case sbyte value:
                oneBasedIndex = value;
                return true;
            case byte value:
                oneBasedIndex = value;
                return true;
            case short value:
                oneBasedIndex = value;
                return true;
            case ushort value:
                oneBasedIndex = value;
                return true;
            case int value:
                oneBasedIndex = value;
                return true;
            case uint value when value <= int.MaxValue:
                oneBasedIndex = (int)value;
                return true;
            case long value when value is >= int.MinValue and <= int.MaxValue:
                oneBasedIndex = (int)value;
                return true;
            default:
                oneBasedIndex = default;
                return false;
        }
    }

    private CapeOpenExceptionContext CreateContext(string operation, object? parameter)
    {
        return new CapeOpenExceptionContext(
            InterfaceName: InterfaceName,
            Scope: "RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders",
            Operation: operation,
            ParameterName: "index",
            Parameter: parameter);
    }

    private void EnsureOwnerAccess(string operation, object? parameter = null)
    {
        _ensureOwnerAccess?.Invoke(InterfaceName, operation, "index", parameter);
    }
}
