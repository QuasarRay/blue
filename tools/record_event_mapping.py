#!/usr/bin/env python3
"""Record the current object/decoration/event slice in the same correspondence.

Every broad source type remains partial until all its responsibilities have a
reviewed counterpart. Original event-test passes do not erase those obligations.
"""
import hashlib
import json
from evidence import ROOT, identity


def main():
    p = ROOT / 'migration/correspondence.json'
    data = json.loads(p.read_text())
    entities = {e['id']: e for e in data['entities']}
    current = identity()['sha256']
    receipts = []
    for receipt in sorted((ROOT / 'migration/receipts').glob('*.json')):
        r = json.loads(receipt.read_text())
        if r.get('exit_code') == 0 and r.get('implementation', {}).get('sha256') == current:
            receipts.append(receipt.relative_to(ROOT).as_posix())
    definitions = [
        ('blueexposure', 'BlueClasses', 'class', 'BlueClasses : IBlueClasses', 'reference/blueexposure/BlueClasses.h', 'blue::BlueClasses', 'struct', 'Initial registered class metadata, construction through PyO3 native constructors, and live instance counts.', ['Full IBlueClasses trait and inherited IRoot relationships, registration IDs, interfaces, copying, thunkers, lock counts, and deferred destruction remain required.']),
        ('blueexposure', 'BlueClasses::PyCreateInstance', 'method', 'static PyObject* PyCreateInstance(PyObject* self, PyObject* args)', 'reference/blueexposure/BlueClasses_Blue.cpp', 'blue::BlueClasses::PyCreateInstance', 'associated_function', 'Resolve registered names and construct the actual Rust object through its native Python constructor.', ['Optional initialization arguments, IInitialize, source construction flags and complete error policy remain required.']),
        ('blueexposure', 'PythonKlass', 'class', 'PythonKlass', 'reference/blueexposure/include/PythonKlass.h', 'blue::PythonKlass', 'struct', 'Own the Blue decorator class and its authoritative instance variable dictionary.', ['Full persistence, copying, custom getattr/setattr/delattr hooks, lookup cache and special callable binding policy remain required.']),
        ('blueexposure', 'PythonKlass::GetAttr', 'method', 'PyObject* GetAttr(const char* name, bool* handled, PyObject* self)', 'reference/blueexposure/PythonKlass.cpp', 'blue::PythonKlass::GetAttr', 'inherent_method', 'Instance variables precede class attributes; bind callable class items to the actual Blue wrapper.', ['GetAttrFinal and special class-bound method semantics remain pending.']),
        ('blueexposure', 'PythonKlass::SetAttr', 'method', 'int SetAttr(bool* handled, PyObject* self, const char* name, PyObject* v, bool force, bool nosetattr)', 'reference/blueexposure/PythonKlass.cpp', 'blue::PythonKlass::SetAttr', 'inherent_method', 'Write decorator variables to one PyO3-owned dictionary.', ['force/nosetattr, custom __setattr__/__delattr__, OnSetAttr and deletion policy remain required.']),
        ('blueexposure', 'PythonKlass::Persist', 'method', 'PyObject* Persist() const', 'reference/blueexposure/PythonKlass.cpp', 'blue::PythonKlass::Persist', 'inherent_method', 'Select declared persistent variables from the same dictionary.', ['Fallback when __persistvars__ is missing and exact absent-variable semantics remain required.']),
        ('blueexposure', 'BlueWrapper', 'class', 'BlueWrapper : BluePythonObject', 'reference/blueexposure/BluePyWrap.h', 'blue::BlueWrapper', 'pyclass_struct', 'Preserve Blue decoration and native wrapper policy while PyO3 owns Python layout/references.', ['Full source wrapper identity, interface casts, protocols, special attributes including __iroot__, source notifications and lifecycle inventory remain required.']),
        ('blueexposure', 'BlueWrapper::PyGetAttr', 'method', 'PyObject* PyGetAttr(const char* name)', 'reference/blueexposure/BluePyWrap.cpp', 'blue::BlueWrapper::PyGetAttr', 'associated_function', 'Consult decoration first, support _parent_ bypass, Blue metadata names, then native descriptors.', ['Full IPythonMethods, reflection lookup, GetAttrFinal, trace and native identity behavior remain required.']),
        ('blueexposure', 'BlueWrapper::PySetAttr', 'method', 'int PySetAttr(const char* name, PyObject* v)', 'reference/blueexposure/BluePyWrap.cpp', 'blue::BlueWrapper::PySetAttr', 'associated_function', 'Attach/rebind the decorator; apply actual native property descriptors and decorator variables outside internal locks.', ['Complete source ordering, deletion, custom hooks, OnSetAttr and interface notification behavior remain required.']),
        ('blue', 'BluePyOS', 'class', 'BluePyOS : IBluePyOS', 'src-cpp/BluePython.h', 'blue::python_runtime::BluePyOS', 'struct', 'Bevy-admitted Python tasks, native task identity, synchronous callbacks, posted callbacks and completion ownership.', ['Complete IBluePyOS/IBluePython hierarchy and method set, lifecycle/shutdown/cancellation, error service, context/timing, Python facilities, and async-loop bridge remain required.']),
        ('blue', 'BluePyOS::CreateTaskletImpl', 'method', 'PyObject* CreateTaskletImpl(PyObject* callable, PyObject* args, PyObject* kw, PyObject* context)', 'src-cpp/BluePython.cpp', 'blue::python_runtime::BluePyOS::CreateTaskletImpl', 'inherent_method', 'Allocate a native stack for synchronous suspension; Bevy admits work and awaits async-channel completion without blocking its async pool.', ['Full task context, cancellation, source scheduling modes and shutdown remain required.']),
        ('blue', 'BluePyOS::SendEvent', 'method', 'bool SendEvent(IRoot* caller, const char* context, const char* eventName, PyObject** pRetval, const char* format, ...)', 'src-cpp/BluePython.cpp', 'blue::python_runtime::BluePyOS::SendEvent', 'inherent_method', 'Invoke the bound callback before returning; preserve true/false success and trap source scheduling during the callback.', ['General DispatchEvent policy, context metadata and native optional result responsibility remain required.']),
        ('blue', 'BluePyOS::PostEvent', 'method', 'bool PostEvent(IRoot* caller, const char* context, const char* eventName, const char* format, ...)', 'src-cpp/BluePython.cpp', 'blue::python_runtime::BluePyOS::PostEvent', 'inherent_method', 'Retain the actual callback and arguments in the queued native task; execute only after Bevy admission.', ['General DispatchEvent policy, context metadata, cancellation and shutdown remain required.']),
        ('blue', 'Synchro', 'class', 'Synchro', 'src-cpp/Synchro.h', 'blue::python_runtime::Synchro', 'struct', 'One authoritative native yield queue shared by the Bevy scheduler and Python adapter.', ['Sleep/timing queues, wakeup, shutdown, statistics, clocks, task-local context and full source relationships remain required.']),
        ('blue', 'Synchro::Yield', 'method', 'PyObject* Yield()', 'src-cpp/Synchro.cpp', 'blue::python_runtime::Synchro::Yield', 'inherent_method', 'Suspend a non-main Python task, detached from Python, until a subsequent Bevy pump; enforce synchronous block traps.', ['Source return/wakeup values and full shutdown/task termination semantics remain required.']),
        ('blue', 'BlueOS', 'class', 'BlueOS : IBlueOS, BeInfo', 'src-cpp/BlueOS.h', 'blue::BlueOS', 'struct', 'Canonical OS pump state, including a concurrent-safe reentrancy guard; real work runs through Bevy schedules.', ['Full IBlueOS/BeInfo graph, clock/timer/tick/IO/lifecycle/error/process/platform responsibilities remain required.']),
        ('blue', 'BlueOS::PumpOS', 'method', 'void PumpOS()', 'src-cpp/BlueOS.cpp', 'blue::BlueOS::PumpOS', 'inherent_method', 'Delegate to the original owning BlueOS pump boundary.', ['Complete PumpOSInternal behavior remains required.']),
        ('blue', 'BlueOS::PumpOSInternal', 'method', 'void PumpOSInternal()', 'src-cpp/BlueOS.cpp', 'blue::BlueOS::PumpOSInternal', 'inherent_method', 'Suppress reentrant/concurrent pumps and execute actual Bevy schedules.', ['Source clocks, tick ordering, IO delivery, resource completion, diagnostics and timing accounting remain required.']),
        ('blue', 'BlueTestEvents', 'class', 'BlueTestEvents : IRoot', 'src-cpp/BlueTestHelpers.h', 'blue::BlueTestEvents', 'struct', 'Original fieldless test helper, with both source methods on the canonical owner and a Bevy-owned lifetime.', ['Source task context labels still need propagation.']),
        ('blue', 'BlueTestEvents::PySendEvent', 'method', 'PyObject* PySendEvent(PyObject* args)', 'src-cpp/BlueTestHelpers.cpp', 'blue::BlueTestEvents::PySendEvent', 'associated_function', 'Convert the original int/double tuple and synchronously dispatch DoSendEvent on the same decorated object.', ['Pass the source context label BlueTestEvents::DoSendEvent.']),
        ('blue', 'BlueTestEvents::PyPostEvent', 'method', 'PyObject* PyPostEvent(PyObject* args)', 'src-cpp/BlueTestHelpers.cpp', 'blue::BlueTestEvents::PyPostEvent', 'associated_function', 'Convert the original int/double tuple and enqueue OnPostEvent on the same decorated object.', ['Pass the source context label BlueTestEvents::OnPostEvent.']),
    ]
    for repo, qualified, kind, signature, source, rust, rust_kind, responsibility, pending in definitions:
        key = f'{repo}::{qualified}::{kind}::{hashlib.sha256(signature.encode()).hexdigest()[:12]}'
        entities[key] = {'id':key, 'original':{'repository':repo,'qualified_identifier':qualified,'kind':kind,'canonical_signature':signature,'declaring_owner':qualified.rsplit('::',1)[0] if '::' in qualified else None,'visibility':'private' if qualified.endswith(('PumpOSInternal','PyGetAttr','PySetAttr')) else 'public','role':'production','source_references':[{'path':source,'revision':data['source_pins'][repo]['revision'],'span_status':'exact declaration/body spans pending'}],'configuration':'Linux; BLUE_WITH_PYTHON=1'},'rust':{'qualified_path':rust,'kind':rust_kind,'impl_owner':rust.rsplit('::',1)[0] if kind=='method' else None},'responsibility':responsibility,'disposition':'BLUE_SPECIFIC','status':'partial','unresolved':pending,'receipts':receipts}
    data['entities'] = list(entities.values())
    data['auxiliary_rust_infrastructure'] = [
        {'path':'blue::python_runtime::BlueTask','role':'Native task identity/completion handle replacing external Stackless representation; not a substitute for missing Blue-specific task context, cancellation or lifecycle behavior.'},
        {'path':'blue::runtime::ObjectHandle<T>','role':'Typed lease into the canonical Bevy-owned object. No duplicate source object state.'},
        {'path':'blue::runtime::ClassRegistration','role':'Initial registration descriptor only; complete Be::ClassRegistration counterpart and related reflection metadata remain pending.'},
    ]
    data['pyo3_replacements'] = {'version':'0.30.1','references':'Py<T>/Bound<T>; Python::attach/ detach; #[pyclass]/#[pymethods]/#[pymodule]; PyVisit; native Python descriptor APIs','retained_blue_policy':'PythonKlass decoration, source names, conversion boundaries, class registration, synchronous/posted event timing and block traps','status':'partial; full BlueExposure source utility inventory and individual replacements remain unresolved'}
    data['initial_batch_limitations'] = [s for s in data.get('initial_batch_limitations',[]) if not s.startswith('Python Write currently')]
    p.write_text(json.dumps(data,indent=2)+'\n')
    print(f'{len(entities)} mapping entries; broad object and service counterparts explicitly partial')


if __name__ == '__main__': main()
