// Copyright (c) 2026 CCP Games

#pragma once
#ifndef TestSuperclassAttributes_h
#define TestSuperclassAttributes_h

#include "TestAttributes.h"

BLUE_CLASS( TestSuperclassAttributes ) : public TestAttributes
{
	public:
		EXPOSE_TO_BLUE();
};

TYPEDEF_BLUECLASS( TestSuperclassAttributes );

#endif // TestSuperclassAttributes_h
