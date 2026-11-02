# Copyright (c) 2026 CCP Games

import unittest
import BlueExposureTest


BLUE_SPECIAL_OBJECT_ATTRIBUTES = "__bluetype__", "__typename__", "__iroot__"


class TestMetaclass(type):
    __persistvars__ = []
    __nonpersistvars__ = []

    def __new__(cls, name, bases, dict):
        return type.__new__(cls, name, bases, dict)

    def __call__(self, *args, **kwargs):
        inst = BlueExposureTest.classes.CreateInstance(self.__cid__)
        inst.__klass__ = self
        return inst


class TestClassWithMetaclass(object, metaclass=TestMetaclass):
    __cid__ = "BlueExposureTest.TestMethods"


class TestDir(unittest.TestCase):
    """
    Test that the dir() function returns
    sensible values for Blue-Exposed things.
    """
    def test_function_in_module_dir(self):
        self.assertIn("FunctionReturningBool", dir(BlueExposureTest))

    def test_enum_in_module_dir(self):
        self.assertIn("TEST_ENUM", dir(BlueExposureTest))

    def test_attributes_in_enum_dir(self):
        self.assertIn("ONE", dir(BlueExposureTest.TEST_ENUM))

    def test_class_in_module_dir(self):
        self.assertIn("TestMethods", dir(BlueExposureTest))

    def test_thunker_in_blue_list_dir(self):
        obj = BlueExposureTest.TestAttributes()
        self.assertIn("CloneTo", dir(obj.myVector))

    def test_method_in_singleton_dir(self):
        self.assertIn("LiveCount", dir(BlueExposureTest.classes))

    def test_method_in_class_type_dir(self):
        self.assertIn("MethodReturningInt", dir(BlueExposureTest.TestMethods))

    def test_method_in_instance_dir(self):
        tm = BlueExposureTest.TestMethods()
        self.assertIn("MethodReturningInt", dir(tm))

    def test_variable_in_instance_dir(self):
        tm = BlueExposureTest.TestMethods()
        self.assertIn("returnError", dir(tm))

    def test_baseclass_method_in_subclass_dir(self):
        class Foo(TestClassWithMetaclass):
            pass
        self.assertIn("MethodReturningInt", dir(Foo()))

    def test_baseclass_variable_in_subclass_dir(self):
        class Foo(TestClassWithMetaclass):
            pass
        self.assertIn("returnError", dir(Foo()))

    def test_subclass_variable_in_subclass_dir(self):
        class Foo(TestClassWithMetaclass):
            bar = 1
        self.assertIn("bar", dir(Foo()))

    def test_subclass_method_in_subclass_dir(self):
        class Foo(TestClassWithMetaclass):
            def DoStuff(self):
                pass
        self.assertIn("DoStuff", dir(Foo()))

    def test_blue_superclass_attribute(self):
        tsa = BlueExposureTest.TestSuperclassAttributes()
        self.assertIn("myInt", dir(tsa))

    def test_blue_superclass_method(self):
        tsm = BlueExposureTest.TestSuperclassMethods()
        self.assertIn("MethodReturningInt", dir(tsm))

    def test_non_python_attributes_on_blue_object(self):
        obj = BlueExposureTest.TestAttributes()
        result = dir(obj)
        for attr in BLUE_SPECIAL_OBJECT_ATTRIBUTES:
            self.assertIn(attr, result)

    def test_non_python_attributes_not_in_module_dir(self):
        result = dir(BlueExposureTest)
        for attr in BLUE_SPECIAL_OBJECT_ATTRIBUTES:
            self.assertNotIn(attr, result)

    def test_type_attributes_not_on_instance_dont_show_up_in_dir(self):
        obj = BlueExposureTest.TestAttributes()
        dir_results = dir(obj)
        for s in dir(type(obj)):
            if not hasattr(obj, s):
                self.assertNotIn(s, dir_results)
