// Copyright (c) 2026 CCP Games

#include "include/BlueExposureMacros.h"

BLUEIMPORT void DummyFunction()
{
}

#if BLUE_WITH_PYTHON
BLUEIMPORT PyObject* PyThunkLeave(PyObject* ret)
{
	if( ret )
	{
		return ret;
	}

	if( !PyErr_Occurred() )
	{
		const char* lastLoggedError = CCP::GetLastErrorMessage();
		if( !lastLoggedError )
		{
			lastLoggedError = "Unknown error";
		}
		PyErr_SetString( PyExc_BlueError, lastLoggedError );
	}

	return nullptr;
}
#endif
