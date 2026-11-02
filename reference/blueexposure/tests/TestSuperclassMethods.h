// Copyright (c) 2026 CCP Games

#pragma once
#ifndef TestSuperclassMethods_h
#define TestSuperclassMethods_h

#include "TestMethods.h"

BLUE_CLASS( TestSuperclassMethods ) : public TestMethods
{
	public:
		EXPOSE_TO_BLUE();
};

TYPEDEF_BLUECLASS( TestSuperclassMethods );

#endif // TestSuperclassMethods_h
