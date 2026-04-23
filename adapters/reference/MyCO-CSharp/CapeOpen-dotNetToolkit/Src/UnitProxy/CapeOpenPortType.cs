#region Copyright (c) 2005-2013, Mose, All rights reserved.
/*
// Copyright (c) 2005-2013, Mose Srl (http://www.mose.units.it/)
// Original capeopentoolkit.net Source Code: Copyright (c) 2005, Marco Carone, Marco Parenzan (e-mail: marco.carone@yahoo.it; marco.parenzan@libero.it)
// All rights reserved.
//  
// Redistribution and use in source and binary forms, with or without modification, are permitted 
// provided that the following conditions are met: 
//  
// (1) Redistributions of source code must retain the above copyright notice, this list of 
// conditions and the following disclaimer. 
// (2) Redistributions in binary form must reproduce the above copyright notice, this list of 
// conditions and the following disclaimer in the documentation and/or other materials 
// provided with the distribution. 
// (3) Neither the name of the Mose nor the names of its contributors may be used 
// to endorse or promote products derived from this software without specific prior 
// written permission.
//      
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS 
// OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY 
// AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR 
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL 
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, 
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER 
// IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT 
// OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
// -------------------------------------------------------------------------
//
// Original capeopentoolkit.net Source Code: Copyright (c) 2005, Marco Carone, Marco Parenzan (e-mail: marco.carone@yahoo.it; marco.parenzan@libero.it)
// 
// Mose is a registered trademark of Mose Srl.
// 
// For portions of this software, the some additional copyright notices may apply 
// which can either be found in the license.txt file included in the source distribution
// or following this notice. 
//
*/
#endregion

using System;
using CAPEOPEN093;
namespace Mose.CapeOpenToolkit.UnitProxy
{
    public enum CapeOpenPortType
    {
        Material = 1,
        Energy = 2,
        Information = 9998,
        Any = 9999
    }

    public class CapeOpenPortTypeHelper
    {
        public static CapeOpenPortType Map(CapePortType type)
        {
            switch (type)
            {
                case CapePortType.CAPE_MATERIAL:
                    return CapeOpenPortType.Material;
                case CapePortType.CAPE_ENERGY:
                    return CapeOpenPortType.Energy;
                case CapePortType.CAPE_INFORMATION:
                    return CapeOpenPortType.Information;
                case CapePortType.CAPE_ANY:
                    return CapeOpenPortType.Any;
                default:
                    throw new CapeOpenException();
            }
        }

        public static CapePortType Map(CapeOpenPortType type)
        {
            switch (type)
            {
                case CapeOpenPortType.Material:
                    return CapePortType.CAPE_MATERIAL;
                case CapeOpenPortType.Energy:
                    return CapePortType.CAPE_ENERGY;
                case CapeOpenPortType.Information:
                    return CapePortType.CAPE_INFORMATION;
                case CapeOpenPortType.Any:
                    return CapePortType.CAPE_ANY;
                default:
                    throw new CapeOpenException();
            }
        }
    }
}
