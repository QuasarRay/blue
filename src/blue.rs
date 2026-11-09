use bevy::prelude::*;

// The templated container classes need special treatment here. Generally
// each exposed class gets its own Python type object, but the templated
// objects all share one type object. This setup here below ensures that
// this type object is known to Python.

pub type GenericList = BlueList<IRoot>;
pub type GenericDict = BlueDict<IRoot>;

type_def_blueclass!(GenericList);
blue_define!(GenericList);

type_def_blueclass!(GenericDict);
blue_define!(GenericDict);

use bevy::prelude::*;

#[allow(non_snake_case)]
pub fn AttachToLogServer() -> bool {
    let mut result = false;

    if IsSocketLoggerConnected() {
        CCP_LOG!("Socket logger is already attached");
        result = true;
    } else if StartSocketLogger() {
        CCP::RegisterLogEcho(
            LogToSocketLogger,
            CCP::LOGTYPE_INFO,
            true,
            CCP::LOG_ECHO_REQUIRES_PRIVILEGE_CHECK,
        );

        CCP_LOG!("Socket logger has been attached");
        result = true;
    } else {
        CCP_LOG!("Failed to attach to socket logger");
        result = false;
    }

    return result;
}

map_function_and_wrap!(
    "AttachToLogServer",
    AttachToLogServer,
    "Attaches to the log server"
);

#[cfg(feature = "blue_with_python")]
mod blue_python {
    use super::*;
    use bevy::prelude::*;

    #[allow(non_snake_case, unused_variables)]
    pub fn PyGetVersionChangelist(
        self_: &PyObject,
        args: &PyObject,
    ) -> &PyObject {
        PyUnicode_FromString(EVECHANGELIST)
    }

    #[allow(non_snake_case, unused_variables)]
    pub fn PyGetVersionBranch(
        self_: &PyObject,
        args: &PyObject,
    ) -> &PyObject {
        PyUnicode_FromString(EVEBRANCH)
    }

    map_function!(
        "GetChangelist",
        PyGetVersionChangelist,
        "Reports the changelist of the blue library"
    );

    map_function!(
        "GetBranch",
        PyGetVersionBranch,
        "Reports the branch of the blue library"
    );
}

//--------------------------------------------------------------------
// AtomicFileRead and Write
// The atomicity is guaratneed by the OS locking thingie, so we can
// release the GIL.
//--------------------------------------------------------------------

use bevy::prelude::*;

#[allow(non_snake_case, unused_variables)]
mod atomic_file_read {
    use super::*;

    pub fn PyAtomicFileRead<'py>(
        self_: &'py PyObject,
        args: &'py PyObject,
    ) -> Option<&'py PyObject> {
        let mut filename: Option<&PyObject> = None;

        if !PyArg_ParseTuple(args, "U", &mut filename) {
            return None;
        }

        let ufn = BluePy(PyUnicode_FromObject(filename.unwrap()));

        if !ufn {
            return None;
        }

        #[cfg(target_os = "windows")]
        {
            let mut h: HANDLE = INVALID_HANDLE_VALUE;
            let mut fileSize: DWORD = 0;
            let mut info: BY_HANDLE_FILE_INFORMATION = Default::default();

            let fileName = PyUnicode_AsWideCharString(ufn.o, None);

            let Some(fileName) = fileName else {
                return None;
            };

            'herr: {
                {
                    let _allow = Ccp::PyAllowThreads::new();

                    for i in 0..10 {
                        h = CreateFileW(
                            fileName,
                            GENERIC_READ,
                            FILE_SHARE_READ,
                            0,
                            OPEN_EXISTING,
                            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_SEQUENTIAL_SCAN,
                            0,
                        );

                        if h == INVALID_HANDLE_VALUE {
                            let code: DWORD = GetLastError();

                            if code == ERROR_SHARING_VIOLATION {
                                Sleep(10);
                                continue;
                            }
                        }

                        break;
                    }

                    if h == INVALID_HANDLE_VALUE {
                        break 'herr;
                    }

                    fileSize = GetFileSize(h, 0);

                    if fileSize == INVALID_FILE_SIZE {
                        break 'herr;
                    }

                    let success: BOOL = GetFileInformationByHandle(
                        h,
                        &mut info,
                    );

                    if success == 0 {
                        break 'herr;
                    }
                }

                {
                    let r = BluePy(PyBytes_FromStringAndSize(
                        None,
                        fileSize as Py_ssize_t,
                    ));

                    if !r {
                        break 'herr;
                    }

                    let mut read: DWORD = 0;

                    {
                        let buffer = PyBytes_AsString(r.o);
                        let _allow = Ccp::PyAllowThreads::new();

                        let success: BOOL = ReadFile(
                            h,
                            buffer,
                            fileSize,
                            &mut read,
                            0,
                        );

                        if success == 0 {
                            break 'herr;
                        }

                        CloseHandle(h);
                        h = INVALID_HANDLE_VALUE;
                    }

                    if read != fileSize {
                        PyErr_SetString(
                            PyExc_RuntimeError,
                            "Read short file",
                        );

                        return None;
                    }

                    PyMem_Free(fileName);

                    return Some(r.Detach());
                }
            }

            // HERR:
            PyMem_Free(fileName);
            PyWin32Error();

            if h != INVALID_HANDLE_VALUE {
                CloseHandle(h);
            }

            return None;
        }

        #[cfg(target_os = "macos")]
        {
            let tmp = PyUnicode_AsWideCharString(ufn.o, None);

            let Some(tmp) = tmp else {
                return None;
            };

            let filenameStr = CW2A::new(tmp);

            PyMem_Free(tmp);

            let f: i32;
            let mut fileSize: i64 = 0;

            {
                let _allow = Ccp::PyAllowThreads::new();

                f = open(
                    &filenameStr,
                    O_RDONLY | O_SHLOCK,
                );

                if f >= 0 {
                    fileSize = lseek(f, 0, SEEK_END);
                    lseek(f, 0, SEEK_SET);
                }
            }

            if f < 0 {
                return PyErr_SetFromErrnoWithFilename(
                    PyExc_OSError,
                    &filenameStr,
                );
            }

            let r = BluePy(PyBytes_FromStringAndSize(
                None,
                fileSize as Py_ssize_t,
            ));

            if !r {
                close(f);
                return None;
            }

            let bytes: ssize_t;

            {
                let _allow = Ccp::PyAllowThreads::new();

                bytes = read(
                    f,
                    PyBytes_AsString(r.o),
                    fileSize,
                );

                close(f);
            }

            if (bytes as i64) < fileSize {
                PyErr_SetString(
                    PyExc_RuntimeError,
                    "Read short file",
                );

                return None;
            }

            return Some(r.Detach());
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        compile_error!("PyAtomicFileRead implementation missing");
    }
}