// Copyright (c) 2026 CCP Games

#include "include/BlueUtil.h"
#include "include/IBlueDict.h"
#include "include/IBlueStructureList.h"
#include "include/IBlueClasses.h"
#include "BluePyWrap.h"
#include "BluePythonThunkers.h"
#include <cmath>

static CcpLogChannel_t s_chOS = CCP_LOG_DEFINE_CHANNEL( "OS" );


//--------------------------------------------------------------------
// Debug aid
//--------------------------------------------------------------------
#ifdef _DEBUG
#if 0
bool GetBlueObjectRepr(IRoot* obj, PyObject* pyobj, char* reprbuff, size_t bufsize)
{
	// Get class id
	const Be::Clsid* clsid = obj->ClassType()->mClassId;
	
	// See if there is a __repr__ method on it
	IPythonMethodsPtr meth(obj);

	PyObject* repr = NULL;
	bool handled = false;
	
	if (meth)
	{
		repr = meth->Repr(&handled);
	}

	// See if there is a valid "name" member
	BlueString name("name", obj);
	BlueWString wname("name", obj);

	// Get blue object refcount
	obj = obj->GetRootObject();
	BlueLockData* ld = NULL;
	
	if ((obj->GetFlags() & BLUERT_AUTOVAR) == 0)
	{
		ld = BlueInternalGetLockData(obj);
	}
	
	// Ex.:
	//triui.UIWindow, refcount=5, pyrefs=1, deco=no, name="", rot="", repr=""
	sprintf_s(
		reprbuff, bufsize,
		"%s.%s, refcount=%d, pyrefs=%d, deco=%s, name=\"%s\", wname=\"%S\", repr=\"%s\"",
		clsid->GetModule(), clsid->GetName(),
		obj->GetRefCount(),
		pyobj ? pyobj->ob_refcnt : -1,
		ld && ld->mPythonKlass ? "yes" : "no",
		name?name.c_str():"",
		wname?wname.c_str():L"",
		repr ? PyString_AS_STRING(repr) : ""
		);

	Py_XDECREF(repr);
	return true;
}

#endif
#endif


Be::Var* BeMapMemberOffset(
	IRoot* obj, 
	const Be::VarEntry* var, 
	const Be::ClassInfo* typeinfo,
	ssize_t xtraoffs
	)
{
	obj = obj->GetRootObject();

	return 
		(Be::Var*)((char*)(((uintptr_t)obj - 
		typeinfo->mInterfaceTable->mOffset + xtraoffs)) + (var)->mOffset);
}



//--------------------------------------------------------------------
// Performance timer
//--------------------------------------------------------------------
BeTimer::BeTimer(const char *msg)
{
	mMsg = msg;
	Reset();
}


void BeTimer::Reset()
{
	mStart = CcpGetTimestamp();
}


void BeTimer::LogTime(const char *msg) const
{
	double s = GetSeconds();

	if (s > 1.0)
		CCP_LOG_CH( s_chOS, "%s: %.3f sec.\n", msg ? msg : "ticks", s);
	else
		CCP_LOG_CH( s_chOS, "%s: %.3f ms.\n", msg ? msg : "ticks", s * 1000.0);
}


Be::Time BeTimer::GetTime() const
{
	int64_t elap = GetCycles();
	
	elap *= 100;
	elap /= (GetFreq() / 100000);

	return elap;
}

int64_t BeTimer::GetCycles() const
{
	Be::Time now = CcpGetTimestamp();
	return now - mStart;
}

int64_t BeTimer::GetFreq()
{
	return CcpGetTimestampFrequency();
}

double BeTimer::GetSeconds() const
{
	return (double)GetCycles() / (double)GetFreq();
}




IRoot* BluePtrAssign(IRoot** pp, IRoot* lp)
{
	if (lp != NULL)
		lp->Lock();
	IRoot *pTmp = *pp;
	if (pTmp) {
		*pp = NULL;
		pTmp->Unlock();
	}
	*pp = lp;
	return lp;
}


IRoot* BlueQIPtrAssign(
	IRoot** pp, IRoot* lp, 
	const Be::IID& riid, 
	BLUEQIOPT options
	)
{
	IRoot* pTemp = *pp, *pNew = NULL;
	*pp = NULL;
	if (lp != NULL)
		lp->QueryInterface(riid, (void**)&pNew, options);
	if (pTemp)
		pTemp->Unlock();
	*pp = pNew;
	return *pp;
}

bool BlueCreateInstance(const Be::Clsid& clsid, const Be::IID& riid, void** ppv)
{
	return BeClasses->CreateInstance(clsid, riid, ppv);
}


#if BLUE_WITH_PYTHON
BLUEIMPORT void BlueInitializePyType( PyTypeObject* pyType, const Be::Clsid* clsid, const char* doc, const Be::InterfaceEntry* interfaces, newfunc pyNew )
{
	CCP_ASSERT( pyType );
	CCP_ASSERT( interfaces );
	CCP_ASSERT( clsid );
	CCP_ASSERT( clsid->GetModule() );
	CCP_ASSERT( clsid->GetName() );

	std::string name = clsid->GetModule();
	name += ".";
	name += clsid->GetName();

	bool isPlain = true;

	static Be::IID pythonNumericType( "IPythonNumeric" );
	static Be::IID listType( "IList" );
	static Be::IID blueDictType( "IBlueDict" );
	static Be::IID blueStructureListType( "IBlueStructureList" );


	const Be::InterfaceEntry* p = interfaces;
	while( p->mIID )
	{
		if( p->mIID->IsEqual( pythonNumericType ) )
		{
			BlueWrapper::InitializeNumericTypeObject( pyType, name.c_str() );
			isPlain = false;
			break;
		}

		if( p->mIID->IsEqual( listType ) )
		{
			BlueWrapper::InitializeListTypeObject( pyType, name.c_str() );
			isPlain = false;
			break;
		}

		if( p->mIID->IsEqual( blueDictType ) )
		{
			BlueWrapper::InitializeDictTypeObject( pyType, name.c_str() );
			isPlain = false;
			break;
		}

		if( p->mIID->IsEqual( blueStructureListType ) )
		{
			BlueWrapper::InitializeStructureListTypeObject( pyType, name.c_str() );
			isPlain = false;
			break;
		}

		// Note that we don't support an object exposing more than one of
		// numeric, list and dict protocols simultaneously.

		++p;
	}

	if( isPlain )
	{
		BlueWrapper::InitializeStandardTypeObject( pyType, name.c_str() );
	}

	pyType->tp_new = pyNew;
	pyType->tp_doc = doc;
}

bool AddObjectDirToList( PyObject* obj, PyObject* list )
{
	PyObject* dirResults = PyObject_Dir( obj );
	if( PyErr_Occurred() )
	{
		return false;
	}
	PyObject* iterator = PyObject_GetIter( dirResults );
	PyObject* entry;
	while( entry = PyIter_Next( iterator ) )
	{
		PyList_Append( list, entry );
		Py_DecRef(entry);
	}
	Py_DECREF( iterator );
	Py_DECREF( dirResults );
	return true;
}

static PyObject* PyBlueObject_Dir( PyObject* self, PyObject* args )
{
	PyObject* results = PyList_New( 0 );

	// Call dir() on the type and add that to the list.
	// This will add any members and methods registered
	// on the PyTypeObject via tp_members and tp_methods.
	PyObject* blueTypeResults = PyList_New( 0 );
	PyObject* classType = PyObject_Type( self );
	AddObjectDirToList( classType, blueTypeResults );
	Py_DECREF( classType );

	PyObject* iterator = PyObject_GetIter( blueTypeResults );
	// Filter out attributes that exist on the type, but not on the instance.
	for( PyObject* entry = PyIter_Next( iterator ); entry; entry = PyIter_Next( iterator ) )
	{
		if( PyObject_HasAttr( self, entry ) )
		{
			PyList_Append( results, entry );
		}
		Py_DecRef(entry);
	}
	Py_DECREF( iterator );
	Py_DECREF( blueTypeResults );

	// If the object has a Python deco, add the results of calling
	// dir() on that.
	if( PyObject_HasAttrString( self, "__klass__" ) )
	{
		PyObject* klass = PyObject_GetAttrString( self, "__klass__" );
		AddObjectDirToList( klass, results );
		Py_DECREF( klass );
	}

	// These are special cases that are handled directly in PyGetAttr
	// but are included here to match Python 2.7 behavior
	PyList_Append( results, PyUnicode_FromString( "__bluetype__" ) );
	PyList_Append( results, PyUnicode_FromString( "__typename__" ) );
	PyList_Append( results, PyUnicode_FromString( "__iroot__" ) );

	return results;
};

BLUEIMPORT void BlueRegisterPyMethodDefs( const Be::ClassInfo* info, std::vector<PyMethodDef>* methods )
{
	CCP_ASSERT( info && info->mTypeObject && info->mInterfaceTable );
	CCP_ASSERT( methods );

	bool hasDirMethod = false;

	for( PyMethodDef def : *methods )
	{
		if( !strcmp( "__dir__", def.ml_name ) )
		{
			hasDirMethod = true;
			break;
		}
	}

	// Grab method definitions from superclasses.
	for(const Be::ClassInfo* type = info->mParentClassInfo; type; type = type->mParentClassInfo)
	{
		for( PyMethodDef* def = type->mTypeObject->tp_methods; def->ml_name; ++def )
		{
			methods->push_back(*def);
		}
	}

	// Add a dir method if one hasn't already been registered.
	if( !hasDirMethod )
	{
		PyMethodDef dirMethodDef = {
			"__dir__",
			(PyCFunction)PyBlueObject_Dir,
			METH_NOARGS,
			"Returns a list of methods and attributes."
		};
		methods->push_back( dirMethodDef );
	}

	// Create a set of relevant IID hashes for quick lookup.
	std::set<unsigned int> interfaceIIDHashes;
	for( const Be::InterfaceEntry* entry = info->mInterfaceTable; entry->mIID; ++entry )
	{
		interfaceIIDHashes.insert( entry->mIID->GetHash() );
	}

	// Loop over Thunker methods and add method definitions for
	// any relevant interfaces.
	for( auto it : BlueRegistration::GetGlobalThunkerRegs() )
	{
		if( interfaceIIDHashes.find( it.second.GetHash() ) == interfaceIIDHashes.end() )
		{
			continue; // This is for an interface which our class does not implement.
		}
		// Loop until we hit the endEntry (ml_name is 0), see definition of THUNKER_END
		for( const BlueMethodDefinition* def = it.first; def->ml_name; def++ )
		{
			methods->push_back( static_cast<PyMethodDef>( *def ) );
		}
	}

	methods->push_back( PyMethodDef{ 0 } ); // Null terminator.
	info->mTypeObject->tp_methods = &( *methods )[0];
}


BLUEIMPORT void BlueRegisterPyMemberDefs( const Be::ClassInfo* info, std::vector<PyMemberDef>* memberDefs )
{
	CCP_ASSERT( info && info->mTypeObject && info->mMemberTable );
	CCP_ASSERT( memberDefs );

	const static std::map<Be::VARTYPE, int> s_blueVarToPythonType = {
		{ Be::VARTYPE::LONG, T_LONG },
		{ Be::VARTYPE::FLOAT, T_FLOAT },
		{ Be::VARTYPE::DOUBLE, T_DOUBLE },
		{ Be::VARTYPE::BOOL, T_BOOL },
		{ Be::VARTYPE::CHARARRAY, T_STRING },
		{ Be::VARTYPE::CSTRING, T_STRING },
		{ Be::VARTYPE::INT64, T_INT },
		{ Be::VARTYPE::PYOBJECTPTR, T_OBJECT_EX },
		{ Be::VARTYPE::BYTE, T_BYTE },
		{ Be::VARTYPE::SHORT, T_SHORT },
		{ Be::VARTYPE::ULONG, T_ULONG },
		{ Be::VARTYPE::UINT64, T_UINT },
	};

	for(const Be::ClassInfo* type = info; type; type = type->mParentClassInfo)
	{
		for (const Be::VarEntry* entry = type->mMemberTable; entry->mName; entry++) {
			PyMemberDef def;
			def.name = entry->mName;
			auto it = s_blueVarToPythonType.find(entry->mType);
			if( it == s_blueVarToPythonType.end() )
			{
				// We don't have a mapping from this type into a Python type,
				// so let's just call it an object. This should be harmless
				// since variable conversion to/from Python seems to be done
				// based on the VARTYPE.
				def.type = T_OBJECT_EX;
			}
			else
			{
				def.type = it->second;
			}

			def.offset = entry->mOffset; // Member offset on the BlueWrapper in bytes.

			if( entry->mEditFlags & Be::EDITFLAGS::WRITE )
			{
				def.flags = 0; // READWRITE
			}
			else
			{
				def.flags = READONLY;
			}
			def.doc = entry->mDescription;
			memberDefs->push_back( def );
		}
	}
	memberDefs->push_back( PyMemberDef{0} );
	info->mTypeObject->tp_members = &( *memberDefs )[0];
}

// Helper function for implementing the object instantiation function
// set to Python type objects
BLUEIMPORT PyObject* BlueCreateInstanceFromPython( const Be::Clsid& clsid, PyObject* args, PyObject* kwds )
{
	IRootPtr obj;

	const Be::ClassRegistration* cr = BeClasses->GetClassRegistration( clsid );
	CCP_ASSERT( cr );

	static Be::IID irootType( "IRoot" );
	if( !cr->mCreateFn( irootType, (void**)&obj ) )
	{
		Py_RETURN_NONE;
	}

	BluePythonObject* ret =  BlueWrapObjectForPython(obj);

	if( PyObject_HasAttrString( ret, "__init__" ) )
	{
		PyObject* initCall = PyObject_GetAttrString( ret, "__init__" );
		PyObject* initResult = PyObject_CallObject( initCall, args );
		Py_XDECREF(initCall);
		if (!initResult)
		{
			Py_DECREF(ret);
			return NULL;
		}
		Py_XDECREF(initResult);
	}

	return ret;
}
#endif


std::vector<std::wstring> GetSplitCommandLine()
{
	std::vector<std::wstring> res;

#ifdef _WIN32
	const wchar_t *line = GetCommandLineW();

	int numArgs;
	LPWSTR *words = CommandLineToArgvW(line, &numArgs);
	for( int i = 0; i < numArgs; ++i )
	{
		res.push_back( words[i] );
	}
	LocalFree( words );
#else
	// TODO: Implement
#endif

	return res;

}
