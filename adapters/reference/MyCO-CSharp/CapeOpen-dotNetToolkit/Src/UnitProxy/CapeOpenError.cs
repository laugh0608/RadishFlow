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

namespace Mose.CapeOpenToolkit.UnitProxy
{
    public class CapeOpenError
	{
		//data for ECapeUser
		private int mUserCode;
		private string mUserDescription;
		private string mUserInterfaceName;
		private string mUserInfo;
		private string mUserOperation;
		private string mUserScope;
		//data for ECapeRoot
		private string mParaRootName;
		//utility class of the CO error hierarchy
		private int mParaBadArgumentPos;

		private object mParaBadParameterObj;
		private string mParaBadParameterName;

		private string mParaBadInvOperation;

		private double mParaLowBoundary;
		private string mParaBoundaryType;
		private double mParaUpperBoundary;
		private double mParaBoundaryVal;
	
		public CapeOpenError()
		{
			mUserCode = 0;
			mUserDescription = "";
			mUserInterfaceName = "";
			mUserInfo = "";
			mUserOperation = "";
			mUserScope = "";
			mParaRootName = "";
			mParaBadArgumentPos = 0;
			mParaBadParameterObj = null;
			mParaBadParameterName = "";
			mParaBadInvOperation = "";
			mParaLowBoundary = -1E+35;
			mParaBoundaryType = "";
			mParaUpperBoundary = 1E+35;
			mParaBoundaryVal = 0;
		}																																													
		
		//methods for error info between ATError and other classes
		public double ParaBoundaryVal
		{
			get
			{
				return mParaBoundaryVal;
			}
			set
			{
				mParaBoundaryVal = value;
			}
		}
		
		public double UpperBoundary
		{
			get
			{
				return mParaUpperBoundary;
			}
			set
			{
				mParaUpperBoundary = value;
			}
		}

		public string BoundaryType
		{
			get
			{
				return mParaBoundaryType;
			}
			set
			{
				mParaBoundaryType = value;
			}
		}

		public double ParaLowBoundary
		{
			get
			{
				return mParaLowBoundary;
			}
			set
			{
				mParaLowBoundary = value;
			}
		}
		public string ParaBadInvOperation
		{
			get
			{
				return mParaBadInvOperation;
			}
			set
			{
				mParaBadInvOperation = value;
			}
		}

		public string ParaBadParameterName
		{
			get
			{
				return mParaBadParameterName;
			}
			set
			{
				mParaBadParameterName = value;
			}
		}

		public object ParaBadParameterObj
		{
			get
			{
				return mParaBadParameterObj;
			}
			set
			{
				mParaBadParameterObj = value;
			}
		}
		
		public int ParaBadArgumentPos
		{
			get
			{
				return mParaBadArgumentPos;
			}
			set
			{
				mParaBadArgumentPos = value;
			}
		}
	
		public string ParaRootName
		{
			get
			{
				return mParaRootName;
			}
			set
			{
				mParaRootName = value;
			}
		}
		
		public string UserScope
		{
			get
			{
				return mUserScope;
			}
			set
			{
				mUserScope = value;
			}
		}
		
		public string UserOperation
		{
			get
			{
				return mUserOperation;
			}
			set
			{
				mUserOperation = value;
			}
		}

		public string UserInfo
		{
			get
			{
				return mUserInfo;
			}
			set
			{
				mUserInfo = value;
			}
		}

		public string UserInterfaceName
		{
			get
			{
				return mUserInterfaceName;
			}
			set
			{
				mUserInterfaceName = value;
			}
		}

		public string UserDescription
		{
			get
			{
				return mUserDescription;
			}
			set
			{
				mUserDescription = value;
			}
		}

		public int UserCode
		{
			get
			{
				return mUserCode;
			}
			set
			{
				mUserCode = value;
			}
		}
	}
}
