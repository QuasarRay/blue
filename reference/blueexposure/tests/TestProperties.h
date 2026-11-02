////////////////////////////////////////////////////////////////////////////////
//
// Creator:		Snorri Sturluson
// Created:		January 2013
// Copyright (c) 2026 CCP Games
//

#pragma once
#ifndef TestProperties_h
#define TestProperties_h

#include "BlueExposure.h"
#include "BoolBeResult.h"

// This class is used for testing properties exposed to Python
class TestProperties :
	public IRoot
{
public:
	EXPOSE_TO_BLUE();

	TestProperties( IRoot* lockobj = NULL );

	std::string GetString() const;
	void SetString( const std::string& val );

	std::wstring GetUnicode() const;
	void SetUnicode( const std::wstring& val );

	bool GetBool() const;
	void SetBool( bool val );

	int GetInt() const;
	void SetInt( int val );

	unsigned int GetUInt() const;
	void SetUInt( unsigned int val );

	float GetFloat() const;
	void SetFloat( float val );

	double GetDouble() const;
	void SetDouble( double val );

	int64_t GetInt64() const;
	void SetInt64( int64_t val );

	uint64_t GetUInt64() const;
	void SetUInt64( uint64_t val );

	Be::Result<bool> GetIntWithError( int& val ) const;
	Be::Result<bool> SetIntWithError( int val );

	const std::string GetStringConst() const;
	const std::string& GetStringConstRef() const;

	BlueSharedString GetSharedString() const;
	void SetSharedString( BlueSharedString string );

	BlueSharedStringW GetSharedStringW() const;
	void SetSharedStringW( BlueSharedStringW string );

private:
	bool m_returnError;
	std::string m_myString;
	std::wstring m_myUnicode;
	bool m_myBool;
	int m_myInt;
	unsigned int m_myUInt;
	float m_myFloat;
	double m_myDouble;
	int64_t m_myInt64;
	uint64_t m_myUInt64;
	BlueSharedString m_sharedString;
	BlueSharedStringW m_sharedStringW;
};

TYPEDEF_BLUECLASS( TestProperties );


#endif // TestProperties_h