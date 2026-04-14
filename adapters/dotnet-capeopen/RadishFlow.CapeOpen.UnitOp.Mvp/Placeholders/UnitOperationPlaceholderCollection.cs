using System.Collections;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

public sealed class UnitOperationPlaceholderCollection<T> : IReadOnlyList<T>
{
    private readonly IReadOnlyList<T> _items;

    public UnitOperationPlaceholderCollection(IEnumerable<T> items)
    {
        _items = items.ToArray();
    }

    public int Count => _items.Count;

    public T this[int index] => _items[index];

    public IEnumerator<T> GetEnumerator()
    {
        return _items.GetEnumerator();
    }

    IEnumerator IEnumerable.GetEnumerator()
    {
        return GetEnumerator();
    }
}
