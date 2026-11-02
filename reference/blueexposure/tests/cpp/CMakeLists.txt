# Copyright (c) 2026 CCP Games

find_package(GTest REQUIRED)
add_executable(BlueExposureCppUnitTests
    blns.cpp
    main.cpp
    TestBlueSmartPy.cpp
    TestBlueSmartPy.h
)
target_precompile_headers(BlueExposureCppUnitTests
    PRIVATE
        <Python.h>
        <gtest/gtest.h>
)
target_link_libraries(
    BlueExposureCppUnitTests
    BlueExposure
    GTest::GTest
    Python3::Python
)

if (CMAKE_IMPORT_LIBRARY_SUFFIX)
    add_custom_command(TARGET BlueExposureCppUnitTests POST_BUILD
            COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_RUNTIME_DLLS:BlueExposureCppUnitTests> $<TARGET_FILE_DIR:BlueExposureCppUnitTests>
            COMMAND_EXPAND_LISTS
            VERBATIM
    )

    gtest_discover_tests(
        BlueExposureCppUnitTests
        PROPERTIES
            ENVIRONMENT "PYTHON_PREFIX=$<TARGET_FILE_DIR:Python3::Interpreter>"
    )
elseif(APPLE)
   # Python distributions on macOS have a different layout than on Windows, so the prefix passed to the test suite is different
    gtest_discover_tests(
            BlueExposureCppUnitTests
            PROPERTIES
            ENVIRONMENT "PYTHON_PREFIX=${VCPKG_IMPORT_PREFIX}"
    )
endif ()
