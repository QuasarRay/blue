// Copyright (c) 2026 CCP Games

#include "TestSuperclassAttributes.h"

BLUE_DEFINE( TestSuperclassAttributes );

const Be::ClassInfo* TestSuperclassAttributes::ExposeToBlue()
{
	EXPOSURE_BEGIN( TestSuperclassAttributes, "TestSuperclassAttributes is used to help test Blue exposure" )
		MAP_INTERFACE( TestSuperclassAttributes )
	EXPOSURE_CHAINTO(TestAttributes)
}
