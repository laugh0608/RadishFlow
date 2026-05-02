using System.Runtime.InteropServices;

namespace RadishFlow.CapeOpen.Interop.Ole;

[StructLayout(LayoutKind.Sequential)]
public struct OleSize
{
    public int Width;
    public int Height;

    public OleSize(int width, int height)
    {
        Width = width;
        Height = height;
    }
}
