/*Copyright 2016 Caster

* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
* 
*     http://www.apache.org/licenses/LICENSE-2.0
* 
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License.*/

using System;
#pragma warning disable 1591

namespace CasterCore
{
    [Serializable]
    public enum ECapeErrorHResult
    {
        ECapeUnknownHR = -2147220223,
        ECapeDataHR = -2147220222,
        ECapeLicenceErrorHR = -2147220221,
        ECapeBadCOParameterHR = -2147220220,
        ECapeBadArgumentHR = -2147220219,
        ECapeInvalidArgumentHR = -2147220218,
        ECapeOutOfBoundsHR = -2147220217,
        ECapeImplementationHR = -2147220216,
        ECapeNoImplHR = -2147220215,
        ECapeLimitedImplHR = -2147220214,
        ECapeComputationHR = -2147220213,
        ECapeOutOfResourcesHR = -2147220212,
        ECapeNoMemoryHR = -2147220211,
        ECapeTimeOutHR = -2147220210,
        ECapeFailedInitialisationHR = -2147220209,
        ECapeSolvingErrorHR = -2147220208,
        ECapeBadInvOrderHR = -2147220207,
        ECapeInvalidOperationHR = -2147220206,
        ECapePersistenceHR = -2147220205,
        ECapeIllegalAccessHR = -2147220204,
        ECapePersistenceNotFoundHR = -2147220203,
        ECapePersistenceSystemErrorHR = -2147220202,
        ECapePersistenceOverflowHR = -2147220201,
        ECapeOutsideSolverScopeHR = -2147220200,
        ECapeHessianInfoNotAvailableHR = -2147220199,
        ECapeThrmPropertyNotAvailableHR = -2147220192,
    }
}
