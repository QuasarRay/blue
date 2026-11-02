// Copyright (c) 2026 CCP Games

#include <BlueSmartPy.h>
#include "blns.cpp"
#include "TestBlueSmartPy.h"

void BluePyStrTest::SetUp()
{
	// Initialize a python interpreter for test execution
	PyConfig config;
	PyConfig_InitIsolatedConfig(&config);

	const char* pythonPrefix = std::getenv("PYTHON_PREFIX");
	if (!pythonPrefix)
	{
		std::cerr << "Could not find `PYTHON_PREFIX` environment variable, thus Python cannot be initialized correctly.\n";
		exit( -2 );
	}
	PyConfig_SetBytesString(&config, &config.prefix, pythonPrefix);

	if(const auto status = PyConfig_Read( &config ); PyStatus_Exception( status ))
	{
		PyErr_Print();
		exit( -1 );
	}

	if( const auto status = Py_InitializeFromConfig( &config ); PyStatus_Exception( status ) )
	{
		PyErr_Print();
		exit( -1 );
	}

	if(!Py_IsInitialized())
	{
		std::cerr << "Failed initializing Python interpreter\n";
		exit( -1 );
	}

	PyConfig_Clear(&config );
}

void BluePyStrTest::TearDown()
{
	if( Py_FinalizeEx() < 0 )
	{
		PyErr_Print();
		exit( -1 );
	}
}

TEST_P(BluePyStrTest, StrConversion)
{
    auto param = GetParam();
    EXPECT_STREQ( param.data(), BluePyStr(param.data()).Str() );
}

TEST_P(BluePyStrTest, CStrConversion)
{
    auto param = GetParam();
    EXPECT_STREQ( param.data(), BluePyStr(param.data()).CStr() );
}

INSTANTIATE_TEST_SUITE_P(StrFromStr, BluePyStrTest, testing::ValuesIn(s_BigListOfNaughtyStrings));
