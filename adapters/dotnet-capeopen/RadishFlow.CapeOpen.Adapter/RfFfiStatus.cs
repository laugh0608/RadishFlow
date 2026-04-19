namespace RadishFlow.CapeOpen.Adapter;

public enum RfFfiStatus
{
    Ok = 0,
    NullPointer = 1,
    InvalidUtf8 = 2,
    InvalidEngineState = 3,
    Panic = 4,
    InvalidInput = 100,
    DuplicateId = 101,
    MissingEntity = 102,
    InvalidConnection = 103,
    Thermo = 104,
    Flash = 105,
    NotImplemented = 106,
}
