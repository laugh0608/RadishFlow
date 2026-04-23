#pragma once

wstring HResError(HRESULT hr); //get an error description from a HRESULT code
wstring CO_Error(IDispatch *capeObject,HRESULT hr); //get an error description from a HRESULT code thrown by a CAPE-OPEN object