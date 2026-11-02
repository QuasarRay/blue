////////////////////////////////////////////////////////////////////////////////
//
// Creator:		Snorri Sturluson
// Created:		February 2013
// Copyright (c) 2026 CCP Games
//


#pragma once
#ifndef TestReferences_h
#define TestReferences_h

#include "BlueExposure.h"

BLUE_CLASS( TestReferences ) : public IRoot
{
public:
	EXPOSE_TO_BLUE();

private:
	IRootPtr m_strongRef;
	BlueWeakRef<TestReferences> m_weakRef;
};

TYPEDEF_BLUECLASS( TestReferences );

#endif // TestReferences_h