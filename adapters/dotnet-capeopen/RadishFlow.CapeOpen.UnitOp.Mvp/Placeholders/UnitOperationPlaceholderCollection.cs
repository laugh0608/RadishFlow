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
    private readonly IReadOnlyDictionary<string, T> _itemsByName;
    private string _componentName;
    private string _componentDescription;

    public UnitOperationPlaceholderCollection(
        string componentName,
        string componentDescription,
        IEnumerable<T> items,
        Action<string, string, string?, object?>? ensureOwnerAccess = null)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(componentName);
        ArgumentNullException.ThrowIfNull(componentDescription);
        ArgumentNullException.ThrowIfNull(items);

        _componentName = componentName;
        _componentDescription = componentDescription;
        _ensureOwnerAccess = ensureOwnerAccess;
        _items = CreateFrozenItems(items);
        _itemsByName = CreateItemsByName(_items, componentName);
    }

    public string ComponentName
    {
        get
        {
            EnsureOwnerAccess(nameof(ComponentName));
            return _componentName;
        }
        set => _componentName = SetImmutableComponentName(_componentName, value, nameof(ComponentName));
    }

    public string ComponentDescription
    {
        get
        {
            EnsureOwnerAccess(nameof(ComponentDescription));
            return _componentDescription;
        }
        set => _componentDescription = SetImmutableComponentDescription(_componentDescription, value, nameof(ComponentDescription));
    }

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

    public bool ContainsName(string name)
    {
        EnsureOwnerAccess(nameof(ContainsName), name);
        if (string.IsNullOrWhiteSpace(name))
        {
            return false;
        }

        return _itemsByName.ContainsKey(name);
    }

    public bool TryGetByName(string name, out T? item)
    {
        EnsureOwnerAccess(nameof(TryGetByName), name);
        if (string.IsNullOrWhiteSpace(name))
        {
            item = null;
            return false;
        }

        return _itemsByName.TryGetValue(name, out item);
    }

    public T GetByName(string name)
    {
        EnsureOwnerAccess(nameof(GetByName), name);
        return FindByName(name, nameof(GetByName));
    }

    public T GetByOneBasedIndex(int oneBasedIndex)
    {
        EnsureOwnerAccess(nameof(GetByOneBasedIndex), oneBasedIndex);
        return ResolveByOneBasedIndex(oneBasedIndex, nameof(GetByOneBasedIndex));
    }

    object ICapeCollection.Item(object index)
    {
        EnsureOwnerAccess(ItemOperation, index);

        if (index is string name)
        {
            return FindByName(name, ItemOperation);
        }

        if (TryGetOneBasedIndex(index, out var oneBasedIndex))
        {
            return ResolveByOneBasedIndex(oneBasedIndex, ItemOperation);
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

    private T FindByName(string name, string operation)
    {
        if (string.IsNullOrWhiteSpace(name))
        {
            throw new CapeInvalidArgumentException(
                $"Collection `{ComponentName}` requires a non-empty component name.",
                CreateContext(operation, name));
        }

        if (_itemsByName.TryGetValue(name, out var item))
        {
            return item;
        }

        throw new CapeInvalidArgumentException(
            $"Collection `{ComponentName}` does not contain an item named `{name}`.",
            CreateContext(operation, name));
    }

    private T ResolveByOneBasedIndex(int oneBasedIndex, string operation)
    {
        if (oneBasedIndex >= 1 && oneBasedIndex <= _items.Count)
        {
            return _items[oneBasedIndex - 1];
        }

        throw new CapeInvalidArgumentException(
            $"Collection `{ComponentName}` index `{oneBasedIndex}` is out of range. Expected 1..{_items.Count}.",
            CreateContext(operation, oneBasedIndex));
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
            case nuint value when value <= int.MaxValue:
                oneBasedIndex = (int)value;
                return true;
            case nint value when value is >= int.MinValue and <= int.MaxValue:
                oneBasedIndex = (int)value;
                return true;
            case float value when IsWholeNumber(value) && value is >= int.MinValue and <= int.MaxValue:
                oneBasedIndex = (int)value;
                return true;
            case double value when IsWholeNumber(value) && value is >= int.MinValue and <= int.MaxValue:
                oneBasedIndex = (int)value;
                return true;
            case decimal value when decimal.Truncate(value) == value &&
                                    value is >= int.MinValue and <= int.MaxValue:
                oneBasedIndex = (int)value;
                return true;
            default:
                oneBasedIndex = default;
                return false;
        }
    }

    private static bool IsWholeNumber(float value)
    {
        return !float.IsNaN(value) && !float.IsInfinity(value) && float.Truncate(value) == value;
    }

    private static bool IsWholeNumber(double value)
    {
        return !double.IsNaN(value) && !double.IsInfinity(value) && double.Truncate(value) == value;
    }

    private static IReadOnlyList<T> CreateFrozenItems(IEnumerable<T> items)
    {
        var materialized = items.ToArray();
        if (materialized.Any(static item => item is null))
        {
            throw new ArgumentException("Placeholder collections cannot contain null items.", nameof(items));
        }

        return materialized;
    }

    private static IReadOnlyDictionary<string, T> CreateItemsByName(IReadOnlyList<T> items, string collectionName)
    {
        var itemsByName = new Dictionary<string, T>(StringComparer.OrdinalIgnoreCase);
        foreach (var item in items)
        {
            if (string.IsNullOrWhiteSpace(item.ComponentName))
            {
                throw new ArgumentException(
                    $"Collection `{collectionName}` cannot contain items with blank ComponentName.",
                    nameof(items));
            }

            if (!itemsByName.TryAdd(item.ComponentName, item))
            {
                throw new ArgumentException(
                    $"Collection `{collectionName}` cannot contain duplicate item name `{item.ComponentName}`.",
                    nameof(items));
            }
        }

        return itemsByName;
    }

    private string SetImmutableComponentName(string currentValue, string value, string operation)
    {
        EnsureOwnerAccess(operation, value);
        ArgumentException.ThrowIfNullOrWhiteSpace(value);

        if (string.Equals(currentValue, value, StringComparison.Ordinal))
        {
            return currentValue;
        }

        throw new CapeInvalidArgumentException(
            $"Collection `{currentValue}` does not allow ComponentName mutation in the MVP runtime.",
            CreateContext(operation, value));
    }

    private string SetImmutableComponentDescription(string currentValue, string value, string operation)
    {
        EnsureOwnerAccess(operation, value);
        ArgumentNullException.ThrowIfNull(value);

        if (string.Equals(currentValue, value, StringComparison.Ordinal))
        {
            return currentValue;
        }

        throw new CapeInvalidArgumentException(
            $"Collection `{ComponentName}` does not allow ComponentDescription mutation in the MVP runtime.",
            CreateContext(operation, value));
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
