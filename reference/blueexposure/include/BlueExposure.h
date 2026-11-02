////////////////////////////////////////////////////////////////////////////////
//
// Creator:		Snorri Sturluson
// Created:		January 2013
// Copyright (c) 2026 CCP Games
//
// include this file to pull in the full public interface of BlueExposure. This is
// ideal for putting in your StdAfx.h file.

#pragma once
#ifndef BlueExposure_h
#define BlueExposure_h

#if !defined(BLUE_WITH_PYTHON) && !defined(BLUE_NO_EXPOSURE)
#define BLUE_WITH_PYTHON 1
#endif

#include "BlueTypes.h"
#include "BlueClass.h"
#include "BlueDict.h"
#include "BlueExposureMacros.h"
#include "BlueTypeTraits.h"
#include "BlueMemberFunctionTraits.h"
#include "BlueExtractArgument.h"
#include "BlueCallFunction.h"
#include "BlueListUtil.h"
#include "BlueMemberIterator.h"
#if BLUE_WITH_PYTHON
#include "BluePythonObject.h"
#include "BlueSmartPy.h"
#include "PythonKlass.h"
#include "IPythonMethods.h"
#include "IPythonNumeric.h"
#endif
#include "BlueStructureList.h"
#include "BlueRegistration.h"
#include "BlueSmartPtr.h"
#include "BlueUtil.h"
#include "BlueVectorTypes.h"
#include "BlueWeakRef.h"
#include "IInitialize.h"
#include "INotify.h"
#include "ICustomPersist.h"
#include "StringBeResult.h"
#include "BlueScriptCallback.h"
#include "BlueSharedString.h"
#include <CcpCore.h>

#endif // BlueExposure_h
