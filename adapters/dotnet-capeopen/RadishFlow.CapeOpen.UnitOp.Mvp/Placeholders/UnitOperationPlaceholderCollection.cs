using System.Collections;
using System.Globalization;
using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

// Generic collection helpers are kept internal to the CLR side and should not be
// surfaced directly as COM runtime types.
[ComVisible(false)]
public class UnitOperationPlaceholderCollection<T> : ICapeIdentification, ICapeCollection, IReadOnlyList<T>
    where T : class, ICapeIdentification
{
    private const string InterfaceName = nameof(ICapeCollection);
    private const string ItemOperation = "Item";
    private readonly Action<string, string, string?, object?>? _ensureOwnerAccess;
    private readonly UnitOperationCollectionDefinition _definition;
    private readonly IReadOnlyList<T> _items;
    private readonly IReadOnlyDictionary<string, T> _itemsByName;

    public UnitOperationPlaceholderCollection(
        UnitOperationCollectionDefinition definition,
        IEnumerable<T> items,
        Action<string, string, string?, object?>? ensureOwnerAccess = null)
    {
        ArgumentNullException.ThrowIfNull(definition);
        ArgumentNullException.ThrowIfNull(items);

        _definition = definition;
        _ensureOwnerAccess = ensureOwnerAccess;
        _items = CreateFrozenItems(items);
        _itemsByName = CreateItemsByName(_items, definition.Name);
    }

    public string ComponentName
    {
        get
        {
            EnsureOwnerAccess(nameof(ComponentName));
            return _definition.Name;
        }
        set => SetImmutableComponentName(value, nameof(ComponentName));
    }

    public string ComponentDescription
    {
        get
        {
            EnsureOwnerAccess(nameof(ComponentDescription));
            return _definition.Description;
        }
        set => SetImmutableComponentDescription(value, nameof(ComponentDescription));
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
        UnitOperationComTrace.Write($"{_definition.Name}.{nameof(ICapeCollection)}.{ItemOperation}", "enter", index?.ToString());
        try
        {
            EnsureOwnerAccess(ItemOperation, index);

            object item;
            if (index is string name)
            {
                item = FindByName(name, ItemOperation);
            }
            else if (index is null)
            {
                throw new CapeInvalidArgumentException(
                    $"Collection `{ComponentName}` requires a non-null selector.",
                    CreateContext(ItemOperation, index));
            }
            else if (TryGetOneBasedIndex(index, out var oneBasedIndex))
            {
                item = ResolveByOneBasedIndex(oneBasedIndex, ItemOperation);
            }
            else
            {
                throw new CapeInvalidArgumentException(
                    $"Collection `{ComponentName}` only accepts a 1-based integer index or component name.",
                    CreateContext(ItemOperation, index));
            }

            UnitOperationComTrace.Write(
                $"{_definition.Name}.{nameof(ICapeCollection)}.{ItemOperation}",
                "exit",
                ((ICapeIdentification)item).ComponentName);
            return item;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception($"{_definition.Name}.{nameof(ICapeCollection)}.{ItemOperation}", error);
            throw;
        }
    }

    int ICapeCollection.Count()
    {
        UnitOperationComTrace.Write($"{_definition.Name}.{nameof(ICapeCollection)}.{nameof(ICapeCollection.Count)}", "enter");
        try
        {
            EnsureOwnerAccess("Count");
            var count = _items.Count;
            UnitOperationComTrace.Write(
                $"{_definition.Name}.{nameof(ICapeCollection)}.{nameof(ICapeCollection.Count)}",
                "exit",
                count.ToString(CultureInfo.InvariantCulture));
            return count;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception($"{_definition.Name}.{nameof(ICapeCollection)}.{nameof(ICapeCollection.Count)}", error);
            throw;
        }
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

    private void SetImmutableComponentName(string value, string operation)
    {
        EnsureOwnerAccess(operation, value);
        ArgumentException.ThrowIfNullOrWhiteSpace(value);

        if (string.Equals(_definition.Name, value, StringComparison.Ordinal))
        {
            return;
        }

        throw new CapeInvalidArgumentException(
            $"Collection `{_definition.Name}` does not allow ComponentName mutation in the MVP runtime.",
            CreateContext(operation, value));
    }

    private void SetImmutableComponentDescription(string value, string operation)
    {
        EnsureOwnerAccess(operation, value);
        ArgumentNullException.ThrowIfNull(value);

        if (string.Equals(_definition.Description, value, StringComparison.Ordinal))
        {
            return;
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
