import argparse
import importlib.util
import json
import platform
import shlex
import subprocess
import time
from pathlib import Path

module_spec=importlib.util.spec_from_file_location('required_costs',Path(__file__).with_name('run-rocksdb-required-costs.py'))
required=importlib.util.module_from_spec(module_spec)
module_spec.loader.exec_module(required)
SIZES=[512,65536,1048576]


def verify_report(report,enabled,size):
    assert report['validation'] and report['reopen_validation'] and report['sampling_complete']
    assert report['value_size']==size and report['clients']==report['initial_keys']==16
    assert report['shared_slots']==8 and report['revision_allocator']=='serialized'
    assert report['warmup_seconds']==1 and report['measurement_seconds']==5
    assert report['calls']==report['latency']['samples']>0
    assert len(report['input_fingerprints'])==16 and report['process_samples']>=2
    assert report['research_metrics']['enabled'] is enabled
    if enabled:
        measured=report['research_metrics']['tenants']['tenant_1']
        assert measured['measurement']['service_request_duration']['count']==report['calls']
        for metric in ['service_request_total','semaphore_wait','blocking_dispatch_wait','blocking_closure_duration',
                       'rocksdb_get_call','get_copy_decode']:
            assert measured['read_stages']['measurement']['get'][metric]['count']==report['calls']
        assert measured['outcomes']['measurement']['user_commits']==0
        assert measured['outcomes']['measurement']['other_errors']==0


def run_case(args,version,size,enabled,repetition,sequence):
    binary=getattr(args,version+'_binary')
    label='on' if enabled else 'off'
    name=f'{sequence:03d}-{size}-{version}-{label}-rep{repetition}'
    path=args.result_root/'runs'/name
    path.mkdir(parents=True,exist_ok=False)
    command=[str(binary),'--value-size',str(size),'--metrics',str(int(enabled)),
        '--data-dir',str(path/'db'),'--report',str(path/'report.json')]
    (path/'command.txt').write_text(shlex.join(command)+'\n')
    before=required.digest(binary)
    started=time.monotonic()
    with (path/'stdout.txt').open('w') as output:
        completed=subprocess.run(command,stdout=output,stderr=subprocess.STDOUT)
    (path/'exit-status.txt').write_text(str(completed.returncode)+'\n')
    provenance={'version':version,'value_size':size,'metrics_enabled':enabled,'repetition':repetition,
        'sequence':sequence,'source_sha':getattr(args,version+'_sha'),'elapsed_seconds':time.monotonic()-started,
        'binary_sha256_before':before,'binary_sha256_after':required.digest(binary)}
    (path/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
    assert completed.returncode==0 and provenance['binary_sha256_after']==before,name
    verify_report(json.loads((path/'report.json').read_text()),enabled,size)
    required.archive_database(path)
    (path/'checked.json').write_text(json.dumps({'passed':True})+'\n')
    print(json.dumps({'run':name,'passed':True}),flush=True)


def main():
    parser=argparse.ArgumentParser()
    for version in ['before','after']:
        parser.add_argument('--'+version+'-binary',required=True,type=Path)
        parser.add_argument('--'+version+'-sha',required=True)
    parser.add_argument('--result-root',required=True,type=Path)
    args=parser.parse_args()
    check=required.checked_output
    assert platform.machine()=='aarch64' and check(['hostname'])=='instance-20260923-1013'
    assert check(['git','rev-parse','HEAD'])==args.after_sha and not check(['git','status','--porcelain'])
    assert check(['findmnt','-n','-o','FSTYPE','-T',str(args.result_root.parent)])=='zfs'
    args.result_root.mkdir(exist_ok=False)
    plan={'before_sha':args.before_sha,'after_sha':args.after_sha,
        'before_binary_sha256':required.digest(args.before_binary),'after_binary_sha256':required.digest(args.after_binary),
        'sizes':SIZES,'repetitions':3,'performance_runs':18,'diagnostic_runs':6,
        'scope':'ordinary Get intermediate encoded Vec removal; same canonical decoder and owned full result; transaction path unchanged'}
    (args.result_root/'plan.json').write_text(json.dumps(plan,indent=2)+'\n')
    commands=[['date','-u'],['hostname'],['uname','-a'],['lscpu'],['free','-h'],['df','-h','/bench/zfs/db'],
        ['findmnt','-T','/bench/zfs/db'],['git','status','--short','--branch'],
        ['rustup','run','1.97.1','rustc','-vV'],['git','-C','/bench/zfs/db/product-build/rocksdb-source','rev-parse','HEAD'],
        ['sha256sum','/bench/zfs/db/product-build/rocksdb-source/librocksdb.a']]
    with (args.result_root/'host-and-source.txt').open('w') as output:
        for command in commands:
            output.write(shlex.join(command)+'\n');output.flush()
            subprocess.run(command,stdout=output,stderr=subprocess.STDOUT,check=True)
    sequence=0
    for repetition in range(1,4):
        for size_index,size in enumerate(SIZES):
            versions=['before','after'] if (size_index+repetition)%2 else ['after','before']
            for version in versions:
                sequence+=1
                run_case(args,version,size,False,repetition,sequence)
    assert sequence==18
    for size in SIZES:
        for version in ['before','after']:
            sequence+=1
            run_case(args,version,size,True,1,sequence)
    assert sequence==24 and not check(['git','status','--porcelain'])
    (args.result_root/'complete.json').write_text(json.dumps({'passed':True,'performance_runs':18,'diagnostic_runs':6})+'\n')


if __name__=='__main__':
    main()
