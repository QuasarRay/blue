// Copyright (c) 2026 CCP Games

#pragma once
#ifndef TestBlueSmartPy_h
#define TestBlueSmartPy_h

class BluePyStrTest : public testing::TestWithParam<std::string_view>
{
protected:
	void SetUp() override;

	void TearDown() override;
};

#endif // TestBlueSmartPy_h