////////////////////////////////////////////////////////////////////////////////
//
// Creator:		Snorri Sturluson
// Created:		January 2013
// Copyright (c) 2026 CCP Games
//

#include "TestAttributes.h"

TestAttributes::TestAttributes( IRoot* lockobj ) :
	m_myBool( false ),
	m_myInt( 0 ),
	m_myUInt( 0 ),
	m_myFloat( 0 ),
	m_myDouble( 0 ),
	m_myInt64( 0 ),
	m_myUInt64( 0 ),
	PARENTLOCK( m_myVector )
#if BLUE_WITH_PYTHON
	, PARENTLOCK( m_myDict )
#endif

{
}
