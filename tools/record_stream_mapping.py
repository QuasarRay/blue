#!/usr/bin/env python3
"""Maintain the reviewed memory-stream slice of the canonical correspondence."""
import hashlib
import json
import re
from pathlib import Path
from evidence import ROOT, identity


def main():
    path = ROOT / 'migration/correspondence.json'
    data = json.loads(path.read_text())
    entities = {e['id']: e for e in data['entities']}
    current = identity()['sha256']
    receipts = []
    for p in sorted((ROOT / 'migration/receipts').glob('*.json')):
        r = json.loads(p.read_text())
        if r.get('exit_code') == 0 and r.get('implementation', {}).get('sha256') == current:
            receipts.append(p.relative_to(ROOT).as_posix())

    def add(repo, owner, name, kind, signature, source, rust, rust_kind, declaring_owner=None, responsibility='', **extra):
        qualified = f'{owner}::{name}' if owner else name
        key = f'{repo}::{qualified}::{kind}::{hashlib.sha256(signature.encode()).hexdigest()[:12]}'
        text = (ROOT / source).read_text(errors='replace')
        matches = [i for i, line in enumerate(text.splitlines(), 1) if re.search(r'\b' + re.escape(name.lstrip('~')) + r'\b', line)]
        entities[key] = {'id': key, 'original': {'repository': repo, 'qualified_identifier': qualified, 'kind': kind, 'canonical_signature': signature, 'declaring_owner': declaring_owner or owner or None, 'visibility': 'private' if name in {'Grow', 'mData', 'mSize', 'mAllocSize', 'mPosition', 'mLocked'} else 'public', 'role': 'production', 'source_references': [{'path': source, 'revision': data['source_pins'][repo]['revision'], 'candidate_spans': [[i, i] for i in matches], 'span_precision': 'name occurrences; exact declaration/body spans still pending review'}], 'configuration': 'Linux; BLUE_WITH_PYTHON=1'}, 'rust': {'qualified_path': rust, 'kind': rust_kind, 'impl_owner': extra.pop('impl_owner', None), 'state_owner': extra.pop('state_owner', None)}, 'responsibility': responsibility, 'disposition': 'BLUE_SPECIFIC', 'status': 'implemented_in_first_batch_pending_complete_inventory_review', 'receipts': receipts, **extra}

    add('blue', '', 'MemStream', 'class', 'MemStream : IBlueMemStream, ICacheable', 'src-cpp/BlueMemStream.h', 'blue::streams::MemStream', 'struct', responsibility='In-memory random-access stream; one canonical owner of its buffer, size, capacity, cursor and lock state.', original_bases=['IBlueMemStream', 'ICacheable'], rust_interfaces=['IBlueMemStream', 'IBlueStream', 'ICcpStream', 'ICacheable', 'IRoot'], python={'module':'blue','name':'MemStream','adapter':'blue::exposure::PyMemStream','properties':{'size':{'owner':'MemStream::mSize','access':'read'},'pos':{'owner':'MemStream::mPosition','access':'read'}}}, auxiliary_infrastructure=['blue::ObjectHandle<MemStream>', 'Bevy ObjectState<MemStream>'], unresolved=['Complete source extraction and native external-interface review remain pending.'])
    for name, bases, file in [('IBlueStream',['IRoot','ICcpStream'],'include/IBluePersist.h'),('IBlueMemStream',['IBlueStream'],'include/IBluePersist.h'),('ICacheable',['IRoot'],'include/ICacheable.h')]:
        add('blue','',name,'interface',name+' : '+', '.join(bases),file,'blue::streams::'+name,'trait',responsibility='Preserve original interface identity, declaring-method ownership and superinterface relationships.',original_bases=bases,rust_supertraits=bases)
    fields = {'mData':('char*','Arc<RwLock<Vec<u8>>>; native shared buffer ownership'), 'mSize':('size_t','usize; logical stream size'), 'mAllocSize':('size_t','usize; source-visible owned allocation extent'), 'mPosition':('size_t','usize; cursor'), 'mLocked':('bool','bool; data lock state')}
    for name,(signature,rep) in fields.items():
        add('blue','MemStream',name,'field',signature+' '+name,'src-cpp/BlueMemStream.h','blue::streams::MemStream::'+name,'field',responsibility=rep,state_owner='blue::streams::MemStream')
    methods = {
        'SetBuffer':('bool SetBuffer(void *buf, size_t size)','IBlueMemStream'),
        'Read':('ptrdiff_t Read(void* dest, ptrdiff_t count)','ICcpStream'),
        'Write':('ptrdiff_t Write(const void* source, size_t count)','ICcpStream'),
        'Seek':('ptrdiff_t Seek(ptrdiff_t distance, SeekOrigin method)','ICcpStream'),
        'GetPosition':('ptrdiff_t GetPosition()','ICcpStream'),
        'GetSize':('ptrdiff_t GetSize()','ICcpStream'),
        'LockData':('bool LockData(void** data, size_t size)','IBlueStream'),
        'UnlockData':('bool UnlockData()','IBlueStream'),
        'IsMemoryUsageKnown':('bool IsMemoryUsageKnown()','ICacheable'),
        'GetMemoryUsage':('size_t GetMemoryUsage()','ICacheable'),
        'SetSize':('bool SetSize(size_t newsize)',None),
        'CopyFrom':('ssize_t CopyFrom(IBlueStream* source, size_t count)',None),
        'Grow':('bool Grow(size_t reqSize)',None),
    }
    for name,(signature,trait) in methods.items():
        rust = f'blue::streams::<MemStream as {trait}>::{name}' if trait else f'blue::streams::MemStream::{name}'
        add('blue','MemStream',name,'method',signature,'src-cpp/BlueMemStream.h',rust,'trait_method_impl' if trait else 'inherent_method',declaring_owner=trait or 'MemStream',impl_owner=f'MemStream as {trait}' if trait else 'MemStream',responsibility='Retain source behavior; convert raw pointers/allocation failures to native slices, owned storage and Result.', notes=['CopyFrom deliberately retains source result -1 without effects.'] if name=='CopyFrom' else [])
    add('blue','MemStream','MemStream','constructor','MemStream()','src-cpp/BlueMemStream.h','blue::streams::MemStream::new','associated_function',impl_owner='MemStream',responsibility='Initialize empty buffer, size, allocation extent, cursor and lock state.')
    add('blue','MemStream','~MemStream','destructor','~MemStream()','src-cpp/BlueMemStream.h','blue::streams::MemStream [native field destruction]','native_destruction',responsibility='Native Arc/Vec destruction releases owned storage safely; external shared ownership remains valid.', replacement={'disposition':'NATIVE_RUST_MECHANISM','facility':'std::sync::Arc and Vec field destruction','version':'Rust 1.98.1'})
    for owner,names,file in [('IBlueStream',['LockData','UnlockData'],'include/IBluePersist.h'),('IBlueMemStream',['SetBuffer'],'include/IBluePersist.h'),('ICacheable',['IsMemoryUsageKnown','GetMemoryUsage'],'include/ICacheable.h')]:
        for name in names:
            signature=methods[name][0]
            add('blue',owner,name,'interface_method',signature,file,f'blue::streams::{owner}::{name}','trait_method',declaring_owner=owner,responsibility='Original declaring interface contract; implemented by MemStream.')
    data['entities'] = list(entities.values())
    data['initial_batch_limitations'] = [
        'Registry implementation is partial; full IBlueClasses trait/method set, IDs, discovery, copying and deferred deletion remain required.',
        'IRoot and ICcpStream are explicit Carbon-core native boundaries; dependency-source method inventories remain unresolved.',
        'Python Write currently accepts bytes; full generic contiguous-buffer behavior needs PyO3 buffer adapter work.',
        'Non-overridden source properties/identities and complete BlueExposure wrapper semantics remain required.',
        'Candidate spans are provenance hints; full declaration/body and macro expansion extraction remains unresolved.',
    ]
    path.write_text(json.dumps(data,indent=2)+'\n')
    print(f'{len(data["entities"])} reviewed mapping entries; full correspondence remains incomplete')


if __name__ == '__main__': main()
